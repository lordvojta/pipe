use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    ChaCha20Poly1305, Nonce,
};

/// Generates a random 256-bit encryption key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Encrypts data using ChaCha20-Poly1305 AEAD cipher
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

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

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}
