pub mod aes;
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose};
use rand::TryRngCore;
use std::{env, fs, io::Write};

pub fn load_key_from_env(var: &str) -> anyhow::Result<[u8; 32]> {
    dotenvy::dotenv().ok();

    let raw = match env::var(var) {
        Ok(val) => val,
        // if not exist, create a new one and write into `.env` file
        Err(_) => {
            let mut key = [0u8; 32];
            rand::rngs::OsRng
                .try_fill_bytes(&mut key)
                .map_err(|e| anyhow!("Failed to generate key: {}", e))?;
            let encoded = general_purpose::STANDARD.encode(key);
            let newline = if cfg!(windows) { "\r\n" } else { "\n" };
            let line = format!("{}=B64:{}{}", var, encoded, newline);

            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(".env")
                .map_err(|e| anyhow!("Failed to open .env: {}", e))?;
            file.write_all(line.as_bytes())
                .map_err(|e| anyhow!("Failed to write to .env: {}", e))?;

            println!("Generated and saved new key to .env");
            return Ok(key);
        }
    };

    // Parse the exist key bytes
    let key_bytes = if let Some(hex) = raw.strip_prefix("HEX:") {
        hex::decode(hex).map_err(|e| anyhow::anyhow!("Invalid hex: {:?}", e))?
    } else if let Some(b64) = raw.strip_prefix("B64:") {
        general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| anyhow::anyhow!("Invalid base64: {:?}", e))?
    } else {
        return Err(anyhow::anyhow!("Missing format prefix (HEX: or B64:)"));
    };

    if key_bytes.len() != 32 {
        println!("The length of key bytes is: {}", key_bytes.len());
        return Err(anyhow::anyhow!("Key must be 32 bytes for AES-256"));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    Ok(key)
}
