use std::collections::HashMap;

use super::tool_adaptor;
use super::transport::{TransportConfig, start as start_transport};
use rig::tool::ToolSet;
use rmcp::{RoleClient, service::RunningService};

/// Manage all Mcp Clients
pub struct McpManager {
    pub clients: HashMap<String, RunningService<RoleClient, ()>>,
}

impl McpManager {
    pub async fn get_tool_set(&self) -> anyhow::Result<ToolSet> {
        let mut tool_set = ToolSet::default();
        let mut task = tokio::task::JoinSet::<anyhow::Result<_>>::new();

        for client in self.clients.values() {
            let server = client.peer().clone();
            task.spawn(tool_adaptor::get_tool_set(server));
        }

        let results = task.join_all().await;
        for result in results {
            match result {
                Err(e) => {
                    tracing::error!(error=%e, "Failed to get tool set");
                }
                Ok(tools) => {
                    tool_set.add_tools(tools);
                }
            }
        }

        Ok(tool_set)
    }
}

/// Build McpManager
pub struct McpManagerBuilder {
    server: Vec<(String, TransportConfig)>,
}

impl Default for McpManagerBuilder {
    fn default() -> Self {
        Self::new().add_sse("default-mcp", "http://localhost:8080/mcp")
    }
}

impl McpManagerBuilder {
    pub fn new() -> Self {
        Self { server: Vec::new() }
    }

    pub fn add_sse(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.server
            .push((name.into(), TransportConfig::Sse { url: url.into() }));

        self
    }

    pub fn add_streamable(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.server
            .push((name.into(), TransportConfig::Streamable { url: url.into() }));

        self
    }

    pub fn add_stdio(
        mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        envs: HashMap<String, String>,
    ) -> Self {
        self.server.push((
            name.into(),
            TransportConfig::Stdio {
                command: command.into(),
                args,
                envs,
            },
        ));

        self
    }

    pub async fn build(self) -> anyhow::Result<McpManager> {
        let mut clients = HashMap::new();
        let mut task_set = tokio::task::JoinSet::<anyhow::Result<_>>::new();

        for server in &self.server {
            let (server_name, transport_config) = server.clone();
            task_set.spawn(async move {
                let client = start_transport(transport_config).await?;
                anyhow::Result::Ok((server_name.clone(), client))
            });
        }

        let start_up_result = task_set.join_all().await;
        for result in start_up_result {
            match result {
                Ok((name, client)) => {
                    clients.insert(name, client);
                }
                Err(e) => {
                    eprintln!("Failed to start server: {e:?}");
                }
            }
        }
        Ok(McpManager { clients })
    }
}
