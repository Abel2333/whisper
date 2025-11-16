use flexi_logger::{Duplicate, FileSpec, Logger};
use rig::embeddings::ToolSchema;
use whisper::{
    agent::{
        cli_chat::CliFrontend,
        model_adaptor::{self, create_agent},
        session::SessionBuilder,
    },
    config,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _logger_handler = Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory("logs")
                .basename("cli_chatbot"),
        )
        .rotate(
            flexi_logger::Criterion::AgeOrSize(flexi_logger::Age::Day, 10_000_000),
            flexi_logger::Naming::TimestampsCustomFormat {
                current_infix: None,
                format: "r%Y-%m-%d",
            },
            flexi_logger::Cleanup::KeepLogFiles(5),
        )
        .append()
        .duplicate_to_stderr(Duplicate::None)
        .format(flexi_logger::opt_format)
        .start()?;

    log::info!("Loading configuration");
    let app_config = config::read_config::load_config()?;

    let agent_settings = model_adaptor::AgentSettings::try_new(
        app_config.models,
        None,
        String::from("You are a helpful assistant."),
        0.7,
    )?;

    log::info!("Building agent instance");
    let agent =
        match create_agent::<ToolSchema, Vec<ToolSchema>>(agent_settings, None::<Vec<ToolSchema>>)
            .await
        {
            Ok(agent) => agent,
            Err(err) => {
                log_error_chain("Failed to create agent", &err);
                return Err(err);
            }
        };

    let session = SessionBuilder::new()
        .agent(agent)
        .multi_turn_depth(4)
        .show_usage()
        .build();

    let mut frontend = CliFrontend::new();
    if let Err(err) = session.run(&mut frontend).await {
        log_error_chain("Session terminated with error", &err);
        return Err(err);
    }

    Ok(())
}

fn log_error_chain(ctx: &str, err: &anyhow::Error) {
    log::error!("{ctx}: {err}");
    for (idx, cause) in err.chain().enumerate().skip(1) {
        log::error!("  cause #{idx}: {cause}");
    }
}
