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
/// PANIC si JWT_SECRET n'est pas défini (sécurité critique)
fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect(
        "FATAL ERROR: JWT_SECRET must be set in .env file.\n\
         \n\
         The server cannot start without a secure JWT secret.\n\
         \n\
         To fix this:\n\
         1. Create or edit your .env file\n\
         2. Add: JWT_SECRET=your-very-long-random-secret-key-here\n\
         3. Generate a secure key with: openssl rand -base64 64\n\
         \n\
         Example .env:\n\
         DATABASE_URL=postgresql://user:pass@localhost/dbname\n\
         JWT_SECRET=your-secure-random-key-minimum-32-characters-long\n"
    )
}

/// Génère un JWT token pour un utilisateur
/// Expiration: 24 heures par défaut
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
        std::env::set_var("JWT_SECRET", "test-secret-key-for-unit-tests-minimum-32-chars");

        let user_id = 123;
        let username = "testuser";

        let token = generate_token(user_id, username).unwrap();
        let claims = verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);

        std::env::remove_var("JWT_SECRET");
    }

    #[test]
    fn test_invalid_token() {
        std::env::set_var("JWT_SECRET", "test-secret-key-for-unit-tests-minimum-32-chars");

        let result = verify_token("invalid.token.here");
        assert!(result.is_err());

        std::env::remove_var("JWT_SECRET");
    }

    #[test]
    #[should_panic(expected = "JWT_SECRET must be set")]
    fn test_missing_jwt_secret_panics() {
        std::env::remove_var("JWT_SECRET");
        get_jwt_secret();
    }
}