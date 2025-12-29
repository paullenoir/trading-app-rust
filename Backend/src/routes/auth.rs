// ============================================================================
// ROUTES : AUTHENTIFICATION
// ============================================================================
//
// Description:
//   Routes d'authentification pour l'application de trading.
//
// Routes disponibles:
//   - POST /api/auth/register : Créer un compte (1-1)
//   - POST /api/auth/login : Se connecter
//   - GET /api/auth/me : Vérifier son token JWT (protégée)
//   - POST /api/auth/change-password : Changer mot de passe (protégée)
//   - POST /api/auth/forgot-password : Demander reset password (2-1)
//   - POST /api/auth/reset-password : Réinitialiser mot de passe avec token (2-2)
//   - GET /api/auth/verify-email : Vérifier l'email avec token (apres register 1-2)
//   - POST /api/auth/google : Authentification Google OAuth
//
// Dépendances:
//   - actix_web : Framework web
//   - sea_orm : ORM PostgreSQL
//   - serde : Sérialisation JSON
//   - chrono : Gestion dates
//   - uuid : Génération tokens
//   - reqwest : Appels HTTP vers Google API
//
// ============================================================================

use actix_web::{post, get, web, HttpResponse};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::models::users::{self, Entity as User};
use crate::models::password_reset_tokens::{self, Entity as PasswordResetToken};
use crate::models::email_verification_tokens::{self, Entity as EmailVerificationToken};
use crate::utils::{jwt, password};
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
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
    pub email: String,
    pub email_verified: bool,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GoogleAuthRequest {
    pub id_token: String,
}

#[derive(Deserialize)]
pub struct GoogleTokenInfo {
    pub sub: String,        // Google ID unique
    pub email: String,
    pub name: Option<String>,
    pub email_verified: Option<String>,
}

// ============================================================================
// REGISTER
// ============================================================================
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

    // Vérifier si email existe déjà
    let existing_email = User::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(db.get_ref())
        .await;

    match existing_email {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Email already exists"
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
        password_hash: Set(Some(password_hash)),
        email: Set(body.email.clone()),
        google_id: Set(None),
        email_verified: Set(false),
        abonnement_id: Set(Some(1)),
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

    // Générer le token de vérification email
    let verification_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    let new_verification_token = email_verification_tokens::ActiveModel {
        user_id: Set(user.id),
        token: Set(verification_token.clone()),
        expires_at: Set(expires_at.naive_utc()),
        used: Set(false),
        ..Default::default()
    };

    // Insérer le token en BD
    if let Err(e) = new_verification_token.insert(db.get_ref()).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create verification token: {}", e)
        }));
    }

    // TODO: Envoyer l'email de vérification avec le lien
    // https://votreapp.com/verify-email?token={verification_token}

    // Générer JWT
    let token = match jwt::generate_token(user.id, &user.username) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Token generation error: {}", e)
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            email_verified: user.email_verified,
        },
        "verification_token": verification_token  // ← À SUPPRIMER EN PRODUCTION
    }))
}

// ============================================================================
// LOGIN
// ============================================================================
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

    // Vérifier que le user a un password_hash (pas OAuth Google)
    let password_hash = match &user.password_hash {
        Some(hash) => hash,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "This account uses Google OAuth. Please login with Google."
            }));
        }
    };

    // Vérifier le mot de passe
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
            email: user.email.clone(),
            email_verified: user.email_verified,
        },
    })
}

// ============================================================================
// ME
// ============================================================================
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

    HttpResponse::Ok().json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "email_verified": user.email_verified,
    }))
}

// ============================================================================
// CHANGE PASSWORD
// ============================================================================
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

    // Vérifier que le user a un password_hash (pas OAuth Google)
    let current_password_hash = match &user.password_hash {
        Some(hash) => hash,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "This account uses Google OAuth. Cannot change password."
            }));
        }
    };

    // Vérifier le mot de passe actuel
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
    active_model.password_hash = Set(Some(new_password_hash));

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

// ============================================================================
// FORGOT PASSWORD
// ============================================================================
#[post("/forgot-password")]
pub async fn forgot_password(
    db: web::Data<DatabaseConnection>,
    body: web::Json<ForgotPasswordRequest>,
) -> HttpResponse {
    // Vérifier que l'email existe
    let user = match User::find()
        .filter(users::Column::Email.eq(&body.email))
        .one(db.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Email not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // Générer un token UUID v4
    let token = Uuid::new_v4().to_string();

    // Calculer la date d'expiration (maintenant + 1 heure)
    let expires_at = Utc::now() + Duration::hours(1);

    // Créer le token de reset
    let new_token = password_reset_tokens::ActiveModel {
        user_id: Set(user.id),
        token: Set(token.clone()),
        expires_at: Set(expires_at.naive_utc()),
        used: Set(false),
        ..Default::default()
    };

    // Insérer en BD
    match new_token.insert(db.get_ref()).await {
        Ok(_) => {
            // TODO: Envoyer l'email ici avec le lien
            // EN PRODUCTION: Ne pas renvoyer le token dans la réponse !
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Password reset email sent. Check your inbox.",
                "token": token  // ← À SUPPRIMER EN PRODUCTION
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create reset token: {}", e)
            }))
        }
    }
}

// ============================================================================
// RESET PASSWORD
// ============================================================================
#[post("/reset-password")]
pub async fn reset_password(
    db: web::Data<DatabaseConnection>,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    // Trouver le token dans la BD
    let reset_token = match PasswordResetToken::find()
        .filter(password_reset_tokens::Column::Token.eq(&body.token))
        .one(db.get_ref())
        .await
    {
        Ok(Some(token)) => token,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid or expired token"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // Vérifier que le token n'a pas déjà été utilisé
    if reset_token.used {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Token has already been used"
        }));
    }

    // Vérifier que le token n'est pas expiré
    let now = Utc::now().naive_utc();
    if reset_token.expires_at < now {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Token has expired"
        }));
    }

    // Trouver l'utilisateur
    let user = match User::find_by_id(reset_token.user_id)
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

    // Hasher le nouveau mot de passe
    let new_password_hash = match password::hash_password(&body.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Password hashing error: {}", e)
            }));
        }
    };

    // Mettre à jour le mot de passe de l'utilisateur
    let mut user_active_model: users::ActiveModel = user.into();
    user_active_model.password_hash = Set(Some(new_password_hash));

    if let Err(e) = user_active_model.update(db.get_ref()).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update password: {}", e)
        }));
    }

    // Marquer le token comme utilisé
    let mut token_active_model: password_reset_tokens::ActiveModel = reset_token.into();
    token_active_model.used = Set(true);

    if let Err(e) = token_active_model.update(db.get_ref()).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to mark token as used: {}", e)
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset successful. You can now login with your new password."
    }))
}

// ============================================================================
// VERIFY EMAIL
// ============================================================================
#[get("/verify-email")]
pub async fn verify_email(
    db: web::Data<DatabaseConnection>,
    query: web::Query<VerifyEmailQuery>,
) -> HttpResponse {
    // Trouver le token dans la BD
    let verification_token = match EmailVerificationToken::find()
        .filter(email_verification_tokens::Column::Token.eq(&query.token))
        .one(db.get_ref())
        .await
    {
        Ok(Some(token)) => token,
        Ok(None) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid or expired verification token"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    // Vérifier que le token n'a pas déjà été utilisé
    if verification_token.used {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Token has already been used"
        }));
    }

    // Vérifier que le token n'est pas expiré
    let now = Utc::now().naive_utc();
    if verification_token.expires_at < now {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Token has expired"
        }));
    }

    // Trouver l'utilisateur
    let user = match User::find_by_id(verification_token.user_id)
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

    // Mettre à jour email_verified = true
    let mut user_active_model: users::ActiveModel = user.into();
    user_active_model.email_verified = Set(true);

    if let Err(e) = user_active_model.update(db.get_ref()).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to verify email: {}", e)
        }));
    }

    // Marquer le token comme utilisé
    let mut token_active_model: email_verification_tokens::ActiveModel = verification_token.into();
    token_active_model.used = Set(true);

    if let Err(e) = token_active_model.update(db.get_ref()).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to mark token as used: {}", e)
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Email verified successfully. Your account is now active."
    }))
}

// ============================================================================
// GOOGLE OAUTH
// ============================================================================
#[post("/google")]
pub async fn google_auth(
    db: web::Data<DatabaseConnection>,
    body: web::Json<GoogleAuthRequest>,
) -> HttpResponse {
    // Vérifier le token Google auprès de l'API Google
    let google_token_url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        body.id_token
    );

    let client = reqwest::Client::new();
    let google_response = match client.get(&google_token_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to verify Google token: {}", e)
            }));
        }
    };

    // Vérifier que la réponse de Google est OK
    if !google_response.status().is_success() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid Google token"
        }));
    }

    // Parser les infos du user depuis Google
    let google_info: GoogleTokenInfo = match google_response.json().await {
        Ok(info) => info,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to parse Google response: {}", e)
            }));
        }
    };

    // Chercher si un user existe déjà avec ce google_id
    let existing_user = User::find()
        .filter(users::Column::GoogleId.eq(&google_info.sub))
        .one(db.get_ref())
        .await;

    match existing_user {
        Ok(Some(user)) => {
            // CAS A: User existe déjà → Login
            let token = match jwt::generate_token(user.id, &user.username) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Token generation error: {}", e)
                    }));
                }
            };

            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": UserInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    email_verified: user.email_verified,
                },
                "is_new_user": false
            }))
        }
        Ok(None) => {
            // CAS B: User n'existe pas → Créer le compte automatiquement

            // Vérifier si l'email existe déjà (avec un autre compte)
            let existing_email = User::find()
                .filter(users::Column::Email.eq(&google_info.email))
                .one(db.get_ref())
                .await;

            match existing_email {
                Ok(Some(_)) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Email already exists with a password account. Please login with your password."
                    }));
                }
                Ok(None) => {}
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Database error: {}", e)
                    }));
                }
            }

            // Générer un username depuis l'email (ex: john@gmail.com → john)
            let username = google_info.email.split('@').next().unwrap_or("user").to_string();

            // Vérifier si le username existe déjà et ajouter un suffixe si nécessaire
            let final_username = match User::find()
                .filter(users::Column::Username.eq(&username))
                .one(db.get_ref())
                .await
            {
                Ok(Some(_)) => format!("{}_{}", username, &google_info.sub[0..6]),
                Ok(None) => username,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Database error: {}", e)
                    }));
                }
            };

            // Créer le nouveau user
            let new_user = users::ActiveModel {
                username: Set(final_username),
                password_hash: Set(None),  // Pas de mot de passe pour Google OAuth
                email: Set(google_info.email.clone()),
                google_id: Set(Some(google_info.sub.clone())),
                email_verified: Set(true),  // Google a déjà vérifié l'email
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

            HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "user": UserInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    email_verified: user.email_verified,
                },
                "is_new_user": true
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// ============================================================================
// CONFIGURATION DES ROUTES
// ============================================================================
pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(get_current_user)
            .service(change_password)
            .service(forgot_password)
            .service(reset_password)
            .service(verify_email)
            .service(google_auth)
    );
}