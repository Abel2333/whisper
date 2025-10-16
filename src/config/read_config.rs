use crate::mcp::transport::TransportConfig;
use config::{Config, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Deserializer, Serialize};
use crate::secure::{self, load_key_from_env};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub models: Vec<ModelConfig>,
    pub mcp_servers: Option<Vec<TransportConfig>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    pub base_url: String,
    pub api_key: String,
    #[serde(deserialize_with = "to_lowercase")]
    pub provider: String,
    pub model_name: String,
    pub model_type: ModelType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Embedding,
    Completion,
    Chat,
}

fn to_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.to_lowercase())
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    dotenv().ok();

    let builder = Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(Environment::with_prefix("WHISPER").separator("__"));

    let config = builder.build()?;

    let key_bytes = load_key_from_env("ENCRYPT_KEY").expect("Get key bytes error!");
    let mut config = config.try_deserialize::<AppConfig>()?;
    let mut valid_models = Vec::new();

    for mut model in config.models {
        // Decrypt api_key
        match secure::aes::decrypt(&model.api_key, &key_bytes) {
            Ok(decrypted) => {
                model.api_key = decrypted;
                valid_models.push(model);
            }
            // if decrypt error, skip this model
            Err(e) => {
                eprintln!(
                    "Failed to decrypt api_key for model '{}': {}. Skip it.",
                    model.model_name, e
                )
            }
        }
    }

    config.models = valid_models;

    Ok(config)
}
