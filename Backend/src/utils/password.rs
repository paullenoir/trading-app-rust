use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;
use rand::Rng;
use base64::{Engine, engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD}};

type HmacSha256 = Hmac<Sha256>;

const ITERATIONS: u32 = 260000;
const KEY_LENGTH: usize = 32;

/// Hash un mot de passe au format Werkzeug (compatible Python)
/// Utilise PBKDF2-HMAC-SHA256 avec 260000 itérations et un salt de 16 bytes
pub fn hash_password(password: &str) -> Result<String, String> {
    // Générer un salt aléatoire de 16 bytes
    let mut salt = [0u8; 16];
    rand::thread_rng().fill(&mut salt);

    // Calculer le hash PBKDF2
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2::<HmacSha256>(password.as_bytes(), &salt, ITERATIONS, &mut key)
        .expect("PBKDF2 hash generation failed");

    // Encoder en base64 URL-safe sans padding (format Werkzeug moderne)
    let salt_b64 = URL_SAFE_NO_PAD.encode(salt);
    let hash_b64 = URL_SAFE_NO_PAD.encode(key);

    // Format: pbkdf2:sha256:iterations$salt$hash
    Ok(format!("pbkdf2:sha256:{}${}${}", ITERATIONS, salt_b64, hash_b64))
}

/// Vérifie un mot de passe contre un hash Werkzeug
/// Supporte les formats: base64 (nouveau) et hex (ancien Python)
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    // Parser le format: pbkdf2:sha256:iterations$salt$hash
    let parts: Vec<&str> = stored_hash.split('$').collect();
    if parts.len() != 3 {
        return Err("Invalid hash format".to_string());
    }

    let header_and_iterations = parts[0];
    let salt_str = parts[1];
    let hash_str = parts[2];

    // Extraire les itérations du header
    let header_parts: Vec<&str> = header_and_iterations.split(':').collect();
    if header_parts.len() != 3 {
        return Err("Invalid header".to_string());
    }

    let iterations = header_parts[2]
        .parse::<u32>()
        .map_err(|_| "Invalid iterations".to_string())?;

    // Décoder salt et hash (supporte plusieurs formats pour compatibilité)
    let salt = decode_flexible(salt_str)?;
    let expected_hash = decode_flexible(hash_str)?;

    // Calculer le hash avec le même salt et iterations
    let mut computed = vec![0u8; expected_hash.len()];
    pbkdf2::<HmacSha256>(password.as_bytes(), &salt, iterations, &mut computed)
        .expect("PBKDF2 hash verification failed");

    // Comparer les hashs (constant-time pour éviter timing attacks)
    Ok(computed == expected_hash)
}

/// Décode une chaîne encodée en base64 ou hexadécimal
/// Essaie plusieurs formats pour assurer la compatibilité avec Python Werkzeug
fn decode_flexible(input: &str) -> Result<Vec<u8>, String> {
    // Format hexadécimal (64 caractères = 32 bytes)
    if input.len() == 64 && input.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex::decode(input)
            .map_err(|e| format!("Hex decode failed: {}", e));
    }

    // Ajouter padding si nécessaire pour base64
    let padded = add_base64_padding(input);

    // Essayer différents formats base64
    if let Ok(decoded) = STANDARD.decode(&padded) {
        return Ok(decoded);
    }
    if let Ok(decoded) = URL_SAFE.decode(&padded) {
        return Ok(decoded);
    }
    if let Ok(decoded) = URL_SAFE_NO_PAD.decode(input) {
        return Ok(decoded);
    }
    if let Ok(decoded) = STANDARD_NO_PAD.decode(input) {
        return Ok(decoded);
    }

    // Dernier recours: hexadécimal
    hex::decode(input)
        .map_err(|_| "Failed to decode".to_string())
}

/// Ajoute le padding '=' manquant pour base64
fn add_base64_padding(input: &str) -> String {
    let padding_needed = (4 - (input.len() % 4)) % 4;
    format!("{}{}", input, "=".repeat(padding_needed))
}