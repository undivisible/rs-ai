//! Tool definitions and tool calling types.
use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AiError;
use crate::error::AiResult;

/// JSON-Schema based definition of a tool that a model can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool.
    pub name: String,
    /// Description of what the tool does.
    pub description: String,
    /// JSON Schema describing the parameters object.
    pub parameters: serde_json::Value,
}

/// A request from the model to invoke a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments to pass to the tool, as a JSON object.
    pub arguments: serde_json::Value,
}

/// The result of executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Identifier of the corresponding tool call.
    pub call_id: String,
    /// Result content from the tool.
    pub content: String,
    /// Whether the tool returned an error.
    pub is_error: bool,
}

/// How the model should choose which tool to call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    /// Let the model decide whether to call a tool.
    Auto,
    /// Do not call any tools.
    None,
    /// The model must call a tool.
    Required,
    /// The model must call the named tool.
    Specific(String),
}

/// A tool that can be called by a model.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Return the JSON-schema definition for this tool.
    fn definition(&self) -> ToolDefinition;
    /// Execute the tool with the given JSON arguments.
    async fn execute(&self, args: serde_json::Value) -> Result<String, AiError>;
}

/// A named collection of tools.
pub struct ToolSet {
    /// Map of tool names to tool implementations.
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolSet {
    /// Create an empty tool set.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool. Returns `&mut Self` for chaining.
    pub fn add(&mut self, tool: impl Tool + 'static) -> &mut Self {
        let def = tool.definition();
        self.tools.insert(def.name.clone(), Box::new(tool));
        self
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// Return definitions for every registered tool.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool call request and return the result.
    pub async fn execute(&self, call: &ToolCallRequest) -> AiResult<ToolCallResult> {
        let tool = self
            .tools
            .get(&call.name)
            .ok_or_else(|| AiError::ToolError {
                tool_name: call.name.clone(),
                message: "Tool not found".into(),
            })?;

        match tool.execute(call.arguments.clone()).await {
            Ok(content) => Ok(ToolCallResult {
                call_id: call.id.clone(),
                content,
                is_error: false,
            }),
            Err(e) => Ok(ToolCallResult {
                call_id: call.id.clone(),
                content: e.to_string(),
                is_error: true,
            }),
        }
    }
}

impl Default for ToolSet {
    fn default() -> Self {
        Self::new()
    }
}
