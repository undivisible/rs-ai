//! MCP (Model Context Protocol) tools — Vercel's `experimental_mcpTools()` equivalent.
//!
//! Supports stdio-based MCP servers using JSON-RPC 2.0.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use crate::error::{AiError, AiResult};
use crate::tool::{Tool, ToolDefinition};

/// Configuration for connecting to an MCP tool server via stdio.
pub struct McpConfig {
    /// Command to run (e.g. "npx", "uvx", "node").
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: HashMap<String, String>,
}

impl McpConfig {
    pub fn new(command: impl Into<String>) -> Self {
        Self { command: command.into(), args: Vec::new(), env: HashMap::new() }
    }
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into()); self
    }
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into()); self
    }
}

// ── JSON-RPC types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Deserialize)]
struct ToolsListResult {
    tools: Vec<McpToolDef>,
}

#[derive(Deserialize, Clone)]
#[allow(non_snake_case)]
struct McpToolDef {
    name: String,
    description: Option<String>,
    inputSchema: Option<serde_json::Value>,
}

// ── MCP client ──────────────────────────────────────────────────────────────

/// A connected MCP server that provides tools.
pub struct McpClient {
    /// The child process stdin.
    stdin: std::process::ChildStdin,
    /// The child process stdout reader.
    stdout: BufReader<std::process::ChildStdout>,
    /// Incrementing request ID.
    next_id: u64,
}

impl McpClient {
    /// Connect to an MCP server by spawning the process.
    pub async fn connect(config: &McpConfig) -> AiResult<Self> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| AiError::Transport {
            message: format!("Failed to spawn MCP server '{}': {e}", config.command),
            source: Some(Box::new(e)),
        })?;

        let stdin = child.stdin.take().ok_or_else(|| AiError::Transport {
            message: "Failed to capture MCP server stdin".into(),
            source: None,
        })?;
        let stdout = child.stdout.take().ok_or_else(|| AiError::Transport {
            message: "Failed to capture MCP server stdout".into(),
            source: None,
        })?;

        let mut client = McpClient {
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };

        // Send initialize request
        client.send_request("initialize", Some(json!({
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": { "name": "rs_ai", "version": "0.2.4" }
        }))).await?;

        // Send initialized notification
        client.send_notification("notifications/initialized").map_err(|e| AiError::Transport {
            message: format!("MCP initialized notification failed: {e}"),
            source: Some(Box::new(e)),
        })?;

        Ok(client)
    }

    /// Send a JSON-RPC request and wait for the response.
    async fn send_request(&mut self, method: &str, params: Option<serde_json::Value>) -> AiResult<serde_json::Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| AiError::Serialization(e.to_string()))?;

        // Write to stdin
        writeln!(self.stdin, "{request_json}").map_err(|e| AiError::Transport {
            message: format!("Failed to write to MCP server: {e}"),
            source: Some(Box::new(e)),
        })?;
        self.stdin.flush().ok();

        // Read response line
        let mut line = String::new();
        self.stdout.read_line(&mut line).map_err(|e| AiError::Transport {
            message: format!("Failed to read from MCP server: {e}"),
            source: Some(Box::new(e)),
        })?;

        if line.trim().is_empty() {
            return Err(AiError::Transport {
                message: "Empty response from MCP server".into(),
                source: None,
            });
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| AiError::Transport {
                message: format!("Invalid JSON-RPC response from MCP server: {e}"),
                source: Some(Box::new(e)),
            })?;

        if let Some(err) = response.error {
            return Err(AiError::Transport {
                message: format!("MCP error ({}): {}", err.code, err.message),
                source: None,
            });
        }

        response.result.ok_or_else(|| AiError::Transport {
            message: "MCP response missing result and error".into(),
            source: None,
        })
    }

    /// Send a JSON-RPC notification (no response expected).
    fn send_notification(&mut self, method: &str) -> Result<(), std::io::Error> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
        });
        writeln!(self.stdin, "{}", serde_json::to_string(&notification).unwrap())?;
        self.stdin.flush()
    }

    /// List available tools from the MCP server.
    pub async fn list_tools(&mut self) -> AiResult<Vec<ToolDefinition>> {
        let result = self.send_request("tools/list", None).await?;
        let tools_list: ToolsListResult = serde_json::from_value(result)
            .map_err(|e| AiError::Serialization(format!("Failed to parse MCP tools/list response: {e}")))?;

        Ok(tools_list.tools.into_iter().map(|t| ToolDefinition {
            name: t.name,
            description: t.description.unwrap_or_default(),
            parameters: t.inputSchema.unwrap_or(serde_json::Value::Null),
        }).collect())
    }

    /// Call a tool on the MCP server and return the result text.
    pub async fn call_tool(&mut self, name: &str, args: serde_json::Value) -> AiResult<String> {
        let result = self.send_request("tools/call", Some(json!({
            "name": name,
            "arguments": args,
        }))).await?;

        // Extract text content from MCP tool result
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let text_parts: Vec<String> = content.iter()
                .filter_map(|part| {
                    if part.get("type") == Some(&serde_json::Value::String("text".to_string())) {
                        part.get("text").and_then(|t| t.as_str().map(String::from))
                    } else {
                        None
                    }
                })
                .collect();
            return Ok(text_parts.join("\n"));
        }

        Ok(result.to_string())
    }
}

// ── MCP Tool wrapper ─────────────────────────────────────────────────────────

/// A tool backed by an MCP server tool definition.
pub struct McpTool {
    definition: ToolDefinition,
    // This tool will be executed through the McpClient, but for the Tool trait
    // we need a different approach. Use `mcp_tools()` to load tools,
    // then execute them through `McpClient::call_tool()`.
}

impl McpTool {
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
            capability: "mcp_tool_execution_without_client".into(),
            provider: "mcp".into(),
        })
    }
}

/// Load tools from an MCP server. Connects via stdio, initializes,
/// lists tools, and returns their definitions.
///
/// ```ignore
/// let config = McpConfig::new("npx")
///     .arg("-y")
///     .arg("@modelcontextprotocol/server-filesystem")
///     .arg("/tmp");
/// let mut client = McpClient::connect(&config).await?;
/// let tools = client.list_tools().await?;
/// ```
pub async fn mcp_tools(config: &McpConfig) -> AiResult<Vec<ToolDefinition>> {
    let mut client = McpClient::connect(config).await?;
    client.list_tools().await
}
