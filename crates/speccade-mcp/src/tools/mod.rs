pub mod analysis;
pub mod authoring;
pub mod discovery;
pub mod generation;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, tool_router, ServerHandler};

#[derive(Clone)]
pub struct SpeccadeMcp {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SpeccadeMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler]
impl ServerHandler for SpeccadeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "SpecCade asset pipeline tools. Create and edit game assets using \
                 declarative Starlark specs. Use stdlib_reference to see available \
                 functions, list_templates for starter templates, write_spec to \
                 create/edit specs, and generate tools to produce assets."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
