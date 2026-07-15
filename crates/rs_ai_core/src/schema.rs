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

/// Structured output strategy for `generate_text` and `stream_text`.
/// Equivalent to Vercel AI SDK's `Output` type.
#[derive(Debug, Clone)]
pub enum Output {
    /// Generate an enum value from a list of options.
    /// Vercel: `Output.enum({ values: [...] })`
    Enum {
        /// Allowed enum values.
        values: Vec<String>,
    },
    /// Generate an array of objects matching the schema.
    /// Vercel: `Output.array(schema)`
    Array(OutputSchema),
    /// Generate a single object matching the schema.
    /// Vercel: `Output.object(schema)`
    Object(OutputSchema),
    /// Generate text without schema constraints.
    /// Vercel: `Output.noSchema()`
    NoSchema,
}

impl Output {
    /// Create an enum output strategy.
    pub fn enum_output(values: Vec<impl Into<String>>) -> Self {
        Output::Enum {
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    /// Create an array output strategy.
    pub fn array(schema: OutputSchema) -> Self {
        Output::Array(schema)
    }

    /// Create an object output strategy.
    pub fn object(schema: OutputSchema) -> Self {
        Output::Object(schema)
    }

    /// Create a no-schema output strategy (free text).
    pub fn no_schema() -> Self {
        Output::NoSchema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_schema_from_value() {
        let schema = OutputSchema::from_value(serde_json::json!({"type": "object"}));
        assert_eq!(schema.schema, serde_json::json!({"type": "object"}));
    }

    #[test]
    fn test_output_enum_variant() {
        let output = Output::enum_output(vec!["red", "green", "blue"]);
        match output {
            Output::Enum { values } => assert_eq!(values, vec!["red", "green", "blue"]),
            _ => panic!("expected Enum"),
        }
    }

    #[test]
    fn test_output_array_variant() {
        let schema = OutputSchema::from_value(serde_json::json!({}));
        match Output::array(schema) {
            Output::Array(_) => {}
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn test_output_object_variant() {
        let schema = OutputSchema::from_value(serde_json::json!({}));
        match Output::object(schema) {
            Output::Object(_) => {}
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn test_output_no_schema() {
        match Output::no_schema() {
            Output::NoSchema => {}
            _ => panic!("expected NoSchema"),
        }
    }
}
