pub mod aes;
use anyhow::anyhow;
use base64::{Engine, engine::general_purpose};
use log::{error, info, warn};
use rand::TryRngCore;
use std::{env, io::Write};

/// Loads or interactively generates a 32-byte AES-256 key from the environment.
pub fn load_key_from_env(var: &str) -> anyhow::Result<[u8; 32]> {
    dotenvy::dotenv().ok();

    let raw = match env::var(var) {
        Ok(val) => val,
        // if not exist, create a new one and write into `.env` file
        Err(_) => {
            warn!("Environment variable '{var}' missing; prompting user to generate key");
            print!("Missing environment variable '{var}'. Generate a new AES-256 key now? [Y/n]:");
            std::io::stdout().flush()?;

            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            answer = answer.trim().to_lowercase();

            if answer == "y" {
                let mut key = [0u8; 32];
                rand::rngs::OsRng
                    .try_fill_bytes(&mut key)
                    .map_err(|e| anyhow!("Failed to generate key: {}", e))?;
                let encoded = general_purpose::STANDARD.encode(key);

                println!("\nYour encryption key has been generated: `B64:{encoded}'");
                println!("Please store it securely\n");
                info!("Generated new AES key for '{var}' via user confirmation");

                return Ok(key);
            } else {
                warn!("User declined to generate AES key for '{var}'");
                return Err(anyhow!("User declined to generate key"));
            }
        }
    };

    let key = decode_key(&raw, var)?;
    info!("Loaded AES key for '{var}' from environment");

    Ok(key)
}

fn decode_key(raw: &str, var: &str) -> anyhow::Result<[u8; 32]> {
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
        error!("Invalid key length for '{var}': {} bytes", key_bytes.len());
        return Err(anyhow::anyhow!("Key must be 32 bytes for AES-256"));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_key_supports_base64_prefix() {
        let key = [1u8; 32];
        let encoded = general_purpose::STANDARD.encode(key);
        let loaded = decode_key(&format!("B64:{encoded}"), "TEST").unwrap();
        assert_eq!(loaded, key);
    }

    #[test]
    fn load_key_rejects_wrong_length() {
        let err = decode_key("B64:AAAA", "TEST").unwrap_err();
        assert!(err.to_string().contains("32 bytes"));
    }
}
