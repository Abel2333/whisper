pub mod chat;
pub mod mcp_adaptor;

use std::env;

use rig::{client::CompletionClient, providers::deepseek};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use crate::chat::SessionBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment file
    dotenvy::dotenv().ok();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        format!("{}.log", env!("CARGO_CRATE_NAME")),
    );
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(file_appender)
        .with_file(false)
        .with_ansi(false)
        .init();

    let _client = match deepseek::Client::builder(
        env::var("MODEL_API_KEY")
            .expect("`MODEL_API_KEY` not set")
            .as_str(),
    )
    .base_url(
        env::var("MODEL_BASE_URL")
            .expect("`MODEL_BASE_URL` not set")
            .as_str(),
    )
    .build()
    {
        Ok(c) => {
            tracing::info!("Client initialized successfully");
            c
        }
        Err(e) => {
            tracing::error!("Failed to build Client: {}", e);
            return Err(anyhow::anyhow!("Client build error: {}", e));
        }
    };

    let agent = _client
        .agent("qwen/qwen3-coder-30b")
        .preamble("Be precise and concise.")
        .temperature(0.5)
        .build();

    let conversation = SessionBuilder::new().agent(agent).show_usage().build();

    conversation.run().await
}
