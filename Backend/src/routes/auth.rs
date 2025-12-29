use actix_web::{post, get, web, HttpResponse};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use crate::models::users::{self, Entity as User};
use crate::utils::{jwt, password};
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[post("/register")]
pub async fn register(
    db: web::Data<DatabaseConnection>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    // Vérifier si username existe déjà
    let existing_user = User::find()
        .filter(users::Column::Username.eq(&body.username))
        .one(db.get_ref())
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Username already exists"
            }));
        }
        Ok(None) => {}
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    }

    // Hasher le mot de passe
    let password_hash = match password::hash_password(&body.password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password hashing error: {}", e)
            }));
        }
    };

    // Créer le user
    let new_user = users::ActiveModel {
        username: Set(body.username.clone()),
        password_hash: Set(password_hash),
        abonnement_id: Set(Some(1)),  // Free par défaut
        ..Default::default()
    };

    let user = match new_user.insert(db.get_ref()).await {
        Ok(user) => user,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create user: {}", e)
            }));
        }
    };

    // Générer JWT
    let token = match jwt::generate_token(user.id, &user.username) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Token generation error: {}", e)
            }));
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    })
}

#[post("/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    // Trouver le user
    let user = match User::find()
        .filter(users::Column::Username.eq(&body.username))
        .one(db.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // Vérifier le mot de passe
    let password_hash = &user.password_hash;

    let is_valid = match password::verify_password(&body.password, password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password verification error: {}", e)
            }));
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }));
    }

    // Générer JWT
    let token = match jwt::generate_token(user.id, &user.username) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Token generation error: {}", e)
            }));
        }
    };

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username.clone(),
        },
    })
}

#[get("/me")]
pub async fn get_current_user(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
) -> HttpResponse {
    let user = match User::find_by_id(auth_user.user_id)
        .one(db.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let username = &user.username;

    HttpResponse::Ok().json(serde_json::json!({
        "id": user.id,
        "username": username,
    }))
}

#[post("/change-password")]
pub async fn change_password(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
    body: web::Json<ChangePasswordRequest>,
) -> HttpResponse {
    // Trouver le user
    let user = match User::find_by_id(auth_user.user_id)
        .one(db.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // Vérifier le mot de passe actuel
    let current_password_hash = &user.password_hash;

    let is_valid = match password::verify_password(&body.current_password, current_password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password verification error: {}", e)
            }));
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Current password is incorrect"
        }));
    }

    // Hasher le nouveau mot de passe
    let new_password_hash = match password::hash_password(&body.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password hashing error: {}", e)
            }));
        }
    };

    // Mettre à jour
    let mut active_model: users::ActiveModel = user.into();
    active_model.password_hash = Set(new_password_hash);

    match active_model.update(db.get_ref()).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Password changed successfully"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update password: {}", e)
            }))
        }
    }
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(get_current_user)
        .service(change_password);
}