use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose};

const NONCE_LEN: usize = 12;

pub fn encrypt(plaintext: &str, key_bytes: &[u8]) -> anyhow::Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(encoded: &str, key_bytes: &[u8]) -> anyhow::Result<String> {
    let combined = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| anyhow!("Base64 decode failed: {:?}", e))?;

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;

    Ok(String::from_utf8(plaintext)?)
}
