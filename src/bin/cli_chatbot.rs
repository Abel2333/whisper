use rig::{
    agent::AgentBuilder, client::completion::CompletionModelHandle, embeddings::EmbeddingsBuilder,
};
use whisper::{agent::model_adaptor::load_models, config, mcp::manager::McpManagerBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = config::read_config::load_config()?;

    let (comp_models, embed_models) = load_models(&app_config)?;

    if let Some(mcp_servers) = &app_config.mcp_servers {
        let mcp_manager = McpManagerBuilder::new()
            .load_config(&mcp_servers)
            .build()
            .await?;

        let tool_set = mcp_manager.get_tool_set().await?;

        let embed_model = &embed_models[0];
        let comp_handle = CompletionModelHandle {
            inner: comp_models[0].clone(),
        };
    }

    Ok(())
}
