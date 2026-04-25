//! Output schema for structured generation.
use serde::{Deserialize, Serialize};

/// A JSON schema describing the expected output format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    /// The schema name (used for labelling in prompts).
    pub name: String,
    /// The JSON Schema object.
    pub schema: serde_json::Value,
}

impl OutputSchema {
    /// Derive an `OutputSchema` from a type that implements `JsonSchema`.
    pub fn from_type<T: schemars::JsonSchema>() -> Self {
        let schema = schemars::schema_for!(T);
        Self {
            name: T::schema_name().to_string(),
            schema: serde_json::to_value(schema).unwrap_or_default(),
        }
    }

    /// Create an output schema from a raw JSON value.
    pub fn from_value(value: serde_json::Value) -> Self {
        Self {
            name: String::new(),
            schema: value,
        }
    }

    /// Return a reference to the underlying JSON Schema value.
    pub fn as_value(&self) -> &serde_json::Value {
        &self.schema
    }
}
