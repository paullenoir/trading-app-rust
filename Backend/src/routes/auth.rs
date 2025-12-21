use actix_web::{post, get, web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};

use crate::models::users::{Entity as Users, Column as UserColumn, ActiveModel as UserActiveModel};
use crate::utils::{password, jwt};
use crate::middleware::AuthUser;  // ← AJOUTÉ

// DTO pour l'inscription
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

// DTO pour la connexion
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// DTO pour changer le mot de passe
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// Réponse après login/register
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
}

// Réponse pour /auth/me
#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: i32,
    pub username: String,
}

/// POST /auth/register - Créer un compte (PUBLIC)
#[post("/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // 1. Vérifier si l'utilisateur existe déjà
    let existing_user = Users::find()
        .filter(UserColumn::Username.eq(&body.username))
        .one(db.get_ref())
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Username already exists"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
        _ => {}
    }

    // 2. Hash le mot de passe
    let password_hash = match password::hash_password(&body.password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to hash password: {}", e)
            }));
        }
    };

    // 3. Créer l'utilisateur
    let new_user = UserActiveModel {
        username: Set(Some(body.username.clone())),
        password_hash: Set(Some(password_hash)),
        wallet: Set(None),
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

    // 4. Générer le JWT
    let token = match jwt::generate_token(user.id, &body.username) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate token: {}", e)
            }));
        }
    };

    // 5. Retourner la réponse
    HttpResponse::Created().json(AuthResponse {
        token,
        user_id: user.id,
        username: body.username.clone(),
    })
}

/// POST /auth/login - Se connecter (PUBLIC)
#[post("/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // 1. Trouver l'utilisateur
    let user = Users::find()
        .filter(UserColumn::Username.eq(&body.username))
        .one(db.get_ref())
        .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid username or password"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // 2. Vérifier le mot de passe
    let password_hash = match user.password_hash {
        Some(ref hash) => hash,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid username or password"
            }));
        }
    };

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
            "error": "Invalid username or password"
        }));
    }

    // 3. Générer le JWT
    let username = user.username.unwrap_or_default();
    let token = match jwt::generate_token(user.id, &username) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate token: {}", e)
            }));
        }
    };

    // 4. Retourner la réponse
    HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user.id,
        username,
    })
}

/// GET /auth/me - Vérifier le token (PROTÉGÉE)
#[get("/me")]
pub async fn me(auth_user: AuthUser) -> HttpResponse {
    HttpResponse::Ok().json(MeResponse {
        user_id: auth_user.user_id,
        username: auth_user.username,
    })
}

/// POST /auth/change-password - Changer son mot de passe (PROTÉGÉE)
#[post("/change-password")]
pub async fn change_password(
    auth_user: AuthUser,
    body: web::Json<ChangePasswordRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // 1. Récupérer l'utilisateur
    let user = match Users::find_by_id(auth_user.user_id)
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

    // 2. Vérifier l'ancien mot de passe
    let current_password_hash = match user.password_hash {
        Some(ref hash) => hash,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "User has no password"
            }));
        }
    };

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

    // 3. Hasher le nouveau mot de passe
    let new_password_hash = match password::hash_password(&body.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to hash password: {}", e)
            }));
        }
    };

    // 4. Mettre à jour le mot de passe dans la BD
    let mut active_model: UserActiveModel = user.into();
    active_model.password_hash = Set(Some(new_password_hash));

    match active_model.update(db.get_ref()).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
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
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(me)
            .service(change_password)
    );
}