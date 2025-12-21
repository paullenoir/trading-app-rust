use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,        // user_id
    pub username: String,
    pub exp: i64,        // expiration timestamp
}

/// Récupère la clé secrète JWT depuis les variables d'environnement
fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("⚠️  WARNING: JWT_SECRET not found in .env, using default (INSECURE)");
        "default-insecure-key-change-this".to_string()
    })
}

/// Génère un JWT token pour un utilisateur
pub fn generate_token(user_id: i32, username: &str) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or("Failed to calculate expiration")?
        .timestamp();

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration,
    };

    let secret = get_jwt_secret();

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
        .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Vérifie et décode un JWT token
pub fn verify_token(token: &str) -> Result<Claims, String> {
    let secret = get_jwt_secret();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let user_id = 123;
        let username = "testuser";

        let token = generate_token(user_id, username).unwrap();
        let claims = verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_token("invalid.token.here");
        assert!(result.is_err());
    }
}