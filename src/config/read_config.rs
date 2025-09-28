use crate::mcp::transport::TransportConfig;
use config::{Config, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub models: Vec<ModelConfig>,
    pub mcp_servers: Option<Vec<TransportConfig>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    pub base_url: String,
    pub api_key: String,
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

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    dotenv().ok();

    let builder = Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(Environment::with_prefix("WHISPER").separator("__"));

    let config = builder.build()?;
    config.try_deserialize::<AppConfig>()
}
