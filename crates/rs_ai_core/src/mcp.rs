//! MCP (Model Context Protocol) tools — Vercel's `experimental_mcpTools()` equivalent.
//!
//! Allows loading tools from MCP servers that communicate via JSON-RPC 2.0.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{AiError, AiResult};
use crate::tool::{Tool, ToolDefinition};

/// Configuration for connecting to an MCP tool server.
pub struct McpConfig {
    /// Command to run (e.g. "npx", "uvx", "node").
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: HashMap<String, String>,
}

impl McpConfig {
    /// Create a new stdio-based MCP config.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// An MCP tool loaded from a remote server (stdio or HTTP).
///
/// Due to the complexity of the JSON-RPC lifecycle, this provides a
/// simplified interface. For full MCP support, use a dedicated MCP client.
pub struct McpTool {
    definition: ToolDefinition,
}

impl McpTool {
    /// Create a new MCP tool from a tool definition.
    pub fn new(definition: ToolDefinition) -> Self {
        Self { definition }
    }
}

#[async_trait]
impl Tool for McpTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String, AiError> {
        Err(AiError::UnsupportedCapability {
            capability: "mcp_tool_execution".into(),
            provider: "mcp".into(),
        })
    }
}

/// Load tools from an MCP server and convert them to [`ToolDefinition`]s.
/// Analogous to Vercel's `experimental_mcpTools()`.
///
/// This is a placeholder that will be expanded with full JSON-RPC/stdio support.
pub async fn mcp_tools(_config: &McpConfig) -> AiResult<Vec<ToolDefinition>> {
    // For now, return an empty list with a descriptive error.
    // Full MCP client implementation requires:
    // 1. Spawning the process with stdio
    // 2. Sending JSON-RPC initialize
    // 3. Sending tools/list
    // 4. Parsing the JSON-RPC response
    // 5. Converting tool schemas to ToolDefinition format
    Err(AiError::UnsupportedCapability {
        capability: "mcp_tools_full".into(),
        provider: "mcp".into(),
    })
}
