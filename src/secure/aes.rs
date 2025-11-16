use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose};
use log::{debug, error};

const NONCE_LEN: usize = 12;

/// Encrypts `plaintext` with the provided AES-256 key bytes and returns a Base64 payload.
/// The payload prepends the randomly generated nonce so `decrypt` can recover it later.
pub fn encrypt(plaintext: &str, key_bytes: &[u8]) -> anyhow::Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|e| {
        error!("AES encryption failed: {e:?}");
        anyhow!("Encryption failed: {:?}", e)
    })?;
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    let encoded = general_purpose::STANDARD.encode(combined);
    debug!("Encrypted {} bytes of plaintext", plaintext.len());
    Ok(encoded)
}

/// Decrypts a value produced by [`encrypt`] using the same AES-256 key bytes.
pub fn decrypt(encoded: &str, key_bytes: &[u8]) -> anyhow::Result<String> {
    let combined = general_purpose::STANDARD.decode(encoded).map_err(|e| {
        error!("Base64 decode failed: {e:?}");
        anyhow!("Base64 decode failed: {:?}", e)
    })?;

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        error!("AES decryption failed: {e:?}");
        anyhow!("Decryption failed: {:?}", e)
    })?;

    let text = String::from_utf8(plaintext)?;
    debug!("Decrypted ciphertext to {} bytes of UTF-8", text.len());
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let key = sample_key();
        let plaintext = "secret message";
        let encoded = encrypt(plaintext, &key).expect("encryption works");
        let decrypted = decrypt(&encoded, &key).expect("decryption works");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn decrypt_rejects_bad_base64() {
        let key = sample_key();
        let err = decrypt("!!!", &key).unwrap_err();
        assert!(err.to_string().contains("Base64"));
    }
}
