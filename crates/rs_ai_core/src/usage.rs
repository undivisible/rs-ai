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

impl std::ops::Add for Usage {
    type Output = Usage;

    fn add(self, rhs: Usage) -> Usage {
        Usage {
            prompt_tokens: add_opt(self.prompt_tokens, rhs.prompt_tokens),
            completion_tokens: add_opt(self.completion_tokens, rhs.completion_tokens),
            total_tokens: add_opt(self.total_tokens, rhs.total_tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let u = Usage::default();
        assert!(u.prompt_tokens.is_none());
        assert!(u.completion_tokens.is_none());
        assert!(u.total_tokens.is_none());
    }

    #[test]
    fn test_add() {
        let a = Usage {
            prompt_tokens: Some(10),
            completion_tokens: Some(20),
            total_tokens: Some(30),
        };
        let b = Usage {
            prompt_tokens: Some(5),
            completion_tokens: Some(5),
            total_tokens: Some(10),
        };
        let sum = a + b;
        assert_eq!(sum.prompt_tokens, Some(15));
        assert_eq!(sum.completion_tokens, Some(25));
        assert_eq!(sum.total_tokens, Some(40));
    }

    #[test]
    fn test_add_with_none() {
        let a = Usage {
            prompt_tokens: Some(10),
            completion_tokens: None,
            total_tokens: None,
        };
        let b = Usage {
            prompt_tokens: None,
            completion_tokens: Some(5),
            total_tokens: None,
        };
        let sum = a + b;
        assert_eq!(sum.prompt_tokens, Some(10));
        assert_eq!(sum.completion_tokens, Some(5));
        assert!(sum.total_tokens.is_none());
    }

    #[test]
    fn test_merge() {
        let mut a = Usage {
            prompt_tokens: Some(10),
            completion_tokens: Some(20),
            total_tokens: Some(30),
        };
        a.merge(&Usage {
            prompt_tokens: Some(5),
            completion_tokens: Some(5),
            total_tokens: Some(10),
        });
        assert_eq!(a.prompt_tokens, Some(15));
        assert_eq!(a.completion_tokens, Some(25));
        assert_eq!(a.total_tokens, Some(40));
    }
}
