use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};

use crate::utils::jwt;

/// Structure qui contient les infos de l'utilisateur authentifié
/// Utilisée comme extracteur dans les routes protégées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: i32,
    pub username: String,
}

/// Implémentation de FromRequest pour AuthUser
/// Cela permet à Actix-Web d'extraire automatiquement AuthUser des requêtes
impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // 1. Extraire le header Authorization
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                let response = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Missing Authorization header"
                }));
                return ready(Err(actix_web::error::InternalError::from_response(
                    "",
                    response,
                ).into()));
            }
        };

        // 2. Convertir le header en string
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                let response = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid Authorization header"
                }));
                return ready(Err(actix_web::error::InternalError::from_response(
                    "",
                    response,
                ).into()));
            }
        };

        // 3. Extraire le token (format: "Bearer <token>")
        let token = if auth_str.starts_with("Bearer ") {
            &auth_str[7..]
        } else {
            let response = HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid Authorization format (expected: Bearer <token>)"
            }));
            return ready(Err(actix_web::error::InternalError::from_response(
                "",
                response,
            ).into()));
        };

        // 4. Vérifier le token JWT
        let claims = match jwt::verify_token(token) {
            Ok(claims) => claims,
            Err(e) => {
                let response = HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": format!("Invalid token: {}", e)
                }));
                return ready(Err(actix_web::error::InternalError::from_response(
                    "",
                    response,
                ).into()));
            }
        };

        // 5. Créer et retourner AuthUser
        ready(Ok(AuthUser {
            user_id: claims.sub,
            username: claims.username,
        }))
    }
}