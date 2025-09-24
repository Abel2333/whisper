use std::vec;

use rig::tool::{ToolDyn as RigTool, ToolEmbeddingDyn, ToolSet};
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Tool as McpTool},
    serde_json,
    service::ServerSink,
};

pub struct McpToolAdaptor {
    tool: McpTool,
    server: ServerSink,
}

impl RigTool for McpToolAdaptor {
    fn name(&self) -> String {
        self.tool.name.to_string()
    }

    fn definition(
        &self,
        _prompt: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>>
    {
        Box::pin(std::future::ready(rig::completion::ToolDefinition {
            name: self.name(),
            description: self
                .tool
                .description
                .as_deref()
                .unwrap_or_default()
                .to_string(),
            parameters: self.tool.schema_as_json_value(),
        }))
    }

    fn call(
        &self,
        args: String,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<String, rig::tool::ToolError>> + Send + '_>>
    {
        let server = self.server.clone();
        Box::pin(async move {
            let call_mcp_tool_result = server
                .call_tool(CallToolRequestParam {
                    name: self.tool.name.clone(),
                    arguments: serde_json::from_str(&args)
                        .map_err(rig::tool::ToolError::JsonError)?,
                })
                .await
                .inspect(|result| tracing::info!(?result))
                .inspect_err(|error| tracing::error!(%error))
                .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;

            Ok(convert_mcp_call_tool_result_to_string(call_mcp_tool_result))
        })
    }
}

impl ToolEmbeddingDyn for McpToolAdaptor {
    fn context(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self.tool.clone())
    }

    fn embedding_docs(&self) -> Vec<String> {
        vec![
            self.tool
                .description
                .as_deref()
                .unwrap_or_default()
                .to_string(),
        ]
    }
}

pub fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
    serde_json::to_string(&result).unwrap()
}

pub async fn get_tool_set(server: ServerSink) -> anyhow::Result<ToolSet> {
    let tools = server.list_all_tools().await?;
    let mut tool_builder = ToolSet::builder();

    for tool in tools {
        tracing::info!("get tool: {}", tool.name);
        let adaptor = McpToolAdaptor {
            tool: tool.clone(),
            server: server.clone(),
        };
        tool_builder = tool_builder.dynamic_tool(adaptor);
    }
    let tool_set = tool_builder.build();
    Ok(tool_set)
}
