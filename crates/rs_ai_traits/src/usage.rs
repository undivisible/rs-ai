//! Token usage tracking.
use serde::{Deserialize, Serialize};

/// Token usage information returned by providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt.
    pub prompt_tokens: Option<u64>,
    /// Tokens in the completion.
    pub completion_tokens: Option<u64>,
    /// Total tokens consumed.
    pub total_tokens: Option<u64>,
}

impl Usage {
    /// Merge another `Usage` into this one by adding token counts.
    pub fn merge(&mut self, other: &Usage) {
        self.prompt_tokens = add_opt(self.prompt_tokens, other.prompt_tokens);
        self.completion_tokens = add_opt(self.completion_tokens, other.completion_tokens);
        self.total_tokens = add_opt(self.total_tokens, other.total_tokens);
    }
}

fn add_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}
