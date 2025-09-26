use std::{collections::HashMap, process::Stdio};

use rmcp::{RoleClient, ServiceExt, service::RunningService, transport::ConfigureCommandExt};

#[derive(Debug, Clone)]
pub enum TransportConfig {
    Sse {
        url: String,
    },
    Streamable {
        url: String,
    },
    Stdio {
        command: String,
        args: Vec<String>,
        envs: HashMap<String, String>,
    },
}

pub async fn start(config: TransportConfig) -> anyhow::Result<RunningService<RoleClient, ()>> {
    let client = match config {
        TransportConfig::Streamable { url } => {
            let transport =
                rmcp::transport::StreamableHttpClientTransport::from_uri(url.to_string());

            ().serve(transport).await?
        }
        TransportConfig::Sse { url } => {
            let transport = rmcp::transport::SseClientTransport::start(url.to_string()).await?;

            ().serve(transport).await?
        }
        TransportConfig::Stdio {
            command,
            args,
            envs,
        } => {
            let transport = rmcp::transport::TokioChildProcess::new(
                tokio::process::Command::new(command).configure(|cmd| {
                    cmd.args(args).envs(envs).stderr(Stdio::null());
                }),
            )?;

            ().serve(transport).await?
        }
    };

    Ok(client)
}
