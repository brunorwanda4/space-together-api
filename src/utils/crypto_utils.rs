use crate::errors::AppError;
use rsa::{
    pkcs8::{EncodePublicKey, LineEnding},
    RsaPublicKey, RsaPrivateKey,
};
use rand::rngs::OsRng;

/// Generate a new RSA-2048 public key
/// Note: This generates only the public key for server-side fallback.
/// For proper E2E encryption, clients should generate their own key pairs.
pub fn generate_rsa_public_key() -> Result<String, AppError> {
    
    // Use OsRng for cryptographically secure random number generation
    let mut rng = OsRng;
    
    // Generate a 2048-bit RSA key pair
    let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| AppError {
        message: format!("Failed to generate RSA key: {}", e),
    })?;
    
    // Extract public key
    let public_key = RsaPublicKey::from(&private_key);
    
    // Convert to PEM format
    let pem = public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| AppError {
            message: format!("Failed to encode public key to PEM: {}", e),
        })?;
    
    Ok(pem)
}
