use rig::providers::{deepseek, ollama, openai};
pub mod cli_chat;
pub mod session;

use crate::config::read_config::{self, AppConfig, ModelConfig, ModelType};

fn get_model(model_config: &ModelConfig) {}

fn create_agent(app_config: &AppConfig) -> anyhow::Result<()> {
    for model_config in &app_config.models {
        match model_config.provider.as_str() {
        }
    }

    Ok(())
}
