pub mod chat;
pub mod mcp;
pub mod config;

use std::env;

use rig::{
    agent::AgentBuilder,
    client::{CompletionClient, EmbeddingsClient},
    embeddings::EmbeddingsBuilder,
    providers::openai,
    vector_store::in_memory_store::InMemoryVectorStore,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use crate::{chat::SessionBuilder, mcp::manager::McpManagerBuilder};

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

    let mcp_manager = McpManagerBuilder::new()
        .add_sse("Weather", "http://localhost:8000/sse")
        .build()
        .await?;

    let tool_set = mcp_manager.get_tool_set().await?;

    let client = match openai::Client::builder(env::var("PROVIDER_API_KEY")?.as_str())
        .base_url(env::var("PROVIDER_BASE_URL")?.as_str())
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

    let chat_model = client
        .completion_model(env::var("MODEL_NAME")?.as_str())
        .completions_api();

    let embed_model = client.embedding_model("text-embedding-v3");
    let embeddings = EmbeddingsBuilder::new(embed_model.clone())
        .documents(tool_set.schemas()?)?
        .build()
        .await?;

    let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |tool| tool.name.clone());

    let index = store.index(embed_model);

    let agent = AgentBuilder::new(chat_model)
        .preamble(
            "You are a helpful assistant.
When answering questions, first write out your reasoning step by step,
then give the final concise answer.  Keep the explanation short but clear.
",
        )
        .temperature(0.6)
        .dynamic_tools(2, index, tool_set)
        .build();

    let conversation = SessionBuilder::new()
        .agent(agent)
        .multi_turn_depth(4)
        .show_usage()
        .build();

    conversation.run().await
}
