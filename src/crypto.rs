use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

/// Generates a random 256-bit encryption key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Encrypts data using ChaCha20-Poly1305 AEAD cipher
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());

    // Generate random nonce (96 bits for ChaCha20)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext (we need it for decryption)
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts data using ChaCha20-Poly1305 AEAD cipher
pub fn decrypt(encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if encrypted_data.len() < 12 {
        anyhow::bail!("Invalid encrypted data: too short");
    }

    let cipher = ChaCha20Poly1305::new(key.into());

    // Extract nonce from the beginning
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Encrypts and encodes data to base64
pub fn encrypt_to_base64(data: &[u8], key: &[u8; 32]) -> Result<String> {
    let encrypted = encrypt(data, key)?;
    Ok(BASE64_STANDARD.encode(encrypted))
}

/// Decodes from base64 and decrypts data
pub fn decrypt_from_base64(encoded: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
    let encrypted = BASE64_STANDARD
        .decode(encoded)
        .context("Failed to decode base64")?;
    decrypt(&encrypted, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = generate_key();
        let data = b"Hello, World! This is sensitive data.";

        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_base64_encryption() {
        let key = generate_key();
        let data = b"Secret environment variable";

        let encrypted = encrypt_to_base64(data, &key).unwrap();
        let decrypted = decrypt_from_base64(&encrypted, &key).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }
}
