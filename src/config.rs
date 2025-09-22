use std::path::Path;

use serde::{Deserialize, Serialize};

pub mod mcp;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mcp: mcp::McpConfig,
    pub provider_api_key: Option<String>,
    pub provider_base_url: Option<String>
}

impl Config {
    pub async fn retrieve(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;

        Ok(config)
    }
}
