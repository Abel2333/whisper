use std::collections::HashMap;

use rig::tool::ToolDyn as RigTool;
use rmcp::{model::Tool as McpTool, service::ServerSink};

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
        todo!();
    }
}
