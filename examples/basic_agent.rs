// pub mod chat;
// pub mod config;
// pub mod mcp;
use whisper::config;

use rig::{agent::AgentBuilder, client::CompletionClient, completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment file

    println!("Read config...");
    let app_config = config::read_config::load_config()?;
    println!("Get {app_config:#?}");

    let client = openai::Client::builder(&app_config.models[0].api_key)
        .base_url(&app_config.models[0].base_url)
        .build()?;

    let chat_model = client
        .completion_model(&app_config.models[0].model_name)
        .completions_api();

    let agent = AgentBuilder::new(chat_model)
        .preamble(
            "You are a helpful assistant.
When answering questions, first write out your reasoning step by step,
then give the final concise answer.  Keep the explanation short but clear.
",
        )
        .temperature(0.6)
        .build();

    let response = agent.prompt("Hello").await.expect("Failed with agent");
    println!("Agent: {response}");

    Ok(())
}
