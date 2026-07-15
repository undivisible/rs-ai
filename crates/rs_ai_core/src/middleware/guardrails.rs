//! Guardrails middleware — Vercel's experimental_guardrails equivalent.
//!
//! Provides content filtering hooks that can block or modify
//! model inputs and outputs based on configurable rules.

use async_trait::async_trait;

use crate::error::{AiError, AiResult};
use crate::{Middleware, MiddlewareNext};
use crate::model::GenerateOptions;
use crate::prompt::Prompt;
use crate::structured::GenerateResult;

/// Content filter action.
#[derive(Debug, Clone)]
pub enum FilterAction {
    /// Allow the content through.
    Allow,
    /// Block with a message.
    Block(String),
    /// Replace with safe content.
    Replace(String),
}

/// A content filter function.
pub type ContentFilter = Box<dyn Fn(&str) -> FilterAction + Send + Sync>;

/// Guardrails configuration.
#[derive(Default)]
pub struct GuardrailConfig {
    /// Filter applied to input prompts.
    pub input_filter: Option<ContentFilter>,
    /// Filter applied to output text.
    pub output_filter: Option<ContentFilter>,
}

impl std::fmt::Debug for GuardrailConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuardrailConfig")
            .field("input_filter", &self.input_filter.is_some())
            .field("output_filter", &self.output_filter.is_some())
            .finish()
    }
}

/// Middleware that applies guardrails to model inputs and outputs.
///
/// ```ignore
/// use rs_ai_core::middleware::{
///     guardrails::{GuardrailConfig, FilterAction},
///     GuardrailMiddleware,
/// };
///
/// let guardrail = GuardrailMiddleware::new(GuardrailConfig {
///     input_filter: Some(Box::new(|text| {
///         if text.contains("dangerous") {
///             FilterAction::Block("Content blocked".into())
///         } else {
///             FilterAction::Allow
///         }
///     })),
///     output_filter: None,
/// });
/// ```
pub struct GuardrailMiddleware {
    config: GuardrailConfig,
}

impl GuardrailMiddleware {
    pub fn new(config: GuardrailConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Middleware for GuardrailMiddleware {
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult> {
        // Check input filter
        if let Some(ref filter) = self.config.input_filter {
            let text = match &prompt {
                Prompt::Text(t) => t.clone(),
                _ => format!("{prompt:?}"),
            };
            match filter(&text) {
                FilterAction::Allow => {}
                FilterAction::Block(msg) => {
                    return Err(AiError::ProviderError {
                        provider: "guardrail".into(),
                        status: Some(403),
                        message: msg,
                    });
                }
                FilterAction::Replace(_safe) => {
                    // Replace logic can be added later if needed.
                }
            }
        }

        let mut result = next.run(prompt, options).await?;

        // Check output filter
        if let Some(ref filter) = self.config.output_filter {
            if let Some(ref text) = result.text {
                match filter(text) {
                    FilterAction::Allow => {}
                    FilterAction::Block(msg) => {
                        return Err(AiError::ProviderError {
                            provider: "guardrail".into(),
                            status: Some(403),
                            message: msg,
                        });
                    }
                    FilterAction::Replace(safe) => {
                        result.text = Some(safe);
                    }
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        middleware::MiddlewareChain,
        model::LanguageModel,
        prompt::Prompt,
        structured::GenerateResult,
        types::{FinishReason, ResponseMetadata},
        Usage,
    };
    use async_trait::async_trait;

    struct TestModel;

    #[async_trait]
    impl LanguageModel for TestModel {
        fn model_id(&self) -> &str {
            "test"
        }
        fn provider_id(&self) -> &str {
            "test"
        }
        fn capabilities(&self) -> &crate::capability::CapabilitySet {
            unimplemented!()
        }
        async fn generate(
            &self,
            _p: Prompt,
            _o: GenerateOptions,
        ) -> AiResult<GenerateResult> {
            Ok(GenerateResult {
                text: Some("safe output".into()),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
                usage: Usage::default(),
                steps: vec![],
                reasoning: None,
                metadata: ResponseMetadata::default(),
            })
        }
        async fn stream(
            &self,
            _p: Prompt,
            _o: GenerateOptions,
        ) -> AiResult<crate::stream::AiStream> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn blocks_forbidden_input() {
        let guardrail = GuardrailMiddleware::new(GuardrailConfig {
            input_filter: Some(Box::new(|t| {
                if t.contains("bad") {
                    FilterAction::Block("blocked".into())
                } else {
                    FilterAction::Allow
                }
            })),
            output_filter: None,
        });
        let model = TestModel;
        let chain = MiddlewareChain::new(model).with(guardrail);
        let result = chain
            .generate(Prompt::Text("bad stuff".into()), GenerateOptions::default())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn allows_safe_input() {
        let guardrail = GuardrailMiddleware::new(GuardrailConfig {
            input_filter: Some(Box::new(|t| {
                if t.contains("bad") {
                    FilterAction::Block("blocked".into())
                } else {
                    FilterAction::Allow
                }
            })),
            output_filter: None,
        });
        let model = TestModel;
        let chain = MiddlewareChain::new(model).with(guardrail);
        let result = chain
            .generate(Prompt::Text("good stuff".into()), GenerateOptions::default())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().text.as_deref(), Some("safe output"));
    }
}
