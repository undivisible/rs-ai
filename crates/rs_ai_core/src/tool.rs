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

/// Tool execution options passed to tool handlers (Vercel's `ToolExecutionOptions`).
#[derive(Debug, Clone, Default)]
pub struct ToolExecutionOptions {
    /// Unique identifier for this tool call.
    pub call_id: String,
    /// Arbitrary context data passed from the caller.
    pub context: Option<serde_json::Value>,
}

/// Context injected into tools at execution time (Vercel's `toolsContext`).
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Per-tool context data.
    pub data: HashMap<String, serde_json::Value>,
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

    /// Execute a tool call request with default options.
    pub async fn execute(&self, call: &ToolCallRequest) -> AiResult<ToolCallResult> {
        self.execute_with_options(call, &ToolExecutionOptions::default())
            .await
    }

    /// Execute a tool call request with execution options and return the result.
    pub async fn execute_with_options(
        &self,
        call: &ToolCallRequest,
        _options: &ToolExecutionOptions,
    ) -> AiResult<ToolCallResult> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct AddTool;

    #[async_trait]
    impl Tool for AddTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "add".into(),
                description: "Add two numbers".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    }
                }),
            }
        }
        async fn execute(&self, args: serde_json::Value) -> Result<String, AiError> {
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Ok((a + b).to_string())
        }
    }

    #[test]
    fn test_tool_definition_creation() {
        let def = ToolDefinition {
            name: "test".into(),
            description: "desc".into(),
            parameters: serde_json::json!({}),
        };
        assert_eq!(def.name, "test");
    }

    #[test]
    fn test_tool_call_request() {
        let req = ToolCallRequest {
            id: "call_1".into(),
            name: "test".into(),
            arguments: serde_json::json!({}),
        };
        assert_eq!(req.id, "call_1");
    }

    #[test]
    fn test_tool_call_result() {
        let res = ToolCallResult {
            call_id: "call_1".into(),
            content: "ok".into(),
            is_error: false,
        };
        assert!(!res.is_error);
        assert_eq!(res.content, "ok");
    }

    #[test]
    fn test_tool_execution_options_default() {
        let opts = ToolExecutionOptions::default();
        assert!(opts.call_id.is_empty());
        assert!(opts.context.is_none());
    }

    #[test]
    fn test_tool_context() {
        let mut ctx = ToolContext::default();
        ctx.data.insert("key".into(), serde_json::json!("value"));
        assert_eq!(ctx.data["key"], serde_json::json!("value"));
    }

    #[test]
    fn test_tool_choice_variants() {
        match ToolChoice::Auto {
            ToolChoice::Auto => {}
            _ => panic!("expected Auto"),
        }
        match ToolChoice::None {
            ToolChoice::None => {}
            _ => panic!("expected None"),
        }
        match ToolChoice::Required {
            ToolChoice::Required => {}
            _ => panic!("expected Required"),
        }
        if let ToolChoice::Specific(name) = ToolChoice::Specific("func".into()) {
            assert_eq!(name, "func");
        } else {
            panic!("expected Specific");
        }
    }

    #[test]
    fn test_tool_set_new_and_get() {
        let mut set = ToolSet::new();
        set.add(AddTool);
        assert!(set.get("add").is_some());
        assert_eq!(set.get("add").unwrap().definition().name, "add");
        assert!(set.get("missing").is_none());
    }

    #[tokio::test]
    async fn test_tool_set_execute() {
        let mut set = ToolSet::new();
        set.add(AddTool);
        let req = ToolCallRequest {
            id: "c1".into(),
            name: "add".into(),
            arguments: serde_json::json!({"a": 2, "b": 3}),
        };
        let result = set.execute(&req).await.unwrap();
        assert_eq!(result.content, "5");
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_tool_set_execute_missing() {
        let set = ToolSet::new();
        let req = ToolCallRequest {
            id: "c1".into(),
            name: "nonexistent".into(),
            arguments: serde_json::json!({}),
        };
        let result = set.execute(&req).await;
        assert!(result.is_err());
    }
}
