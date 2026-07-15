use async_trait::async_trait;

use crate::{
    AiResult, CapabilitySet, GenerateOptions, GenerateResult, LanguageModel, Middleware,
    MiddlewareNext, Prompt,
};

/// A chain of middleware wrapping a language model.
///
/// Middlewares are executed in the order they are added (first added = outermost).
/// Each middleware may inspect or modify the prompt, options, or result before
/// and after calling the next element in the chain.
///
/// # Example
///
/// ```ignore
/// let chain = MiddlewareChain::new(my_model)
///     .with(LoggingMiddleware::new())
///     .with(RetryMiddleware::new(RetryConfig::default()));
///
/// let result = chain.generate(prompt, options).await?;
/// ```
pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
    model: Box<dyn LanguageModel>,
}

impl MiddlewareChain {
    /// Create a new chain wrapping the given language model.
    pub fn new(model: impl LanguageModel + 'static) -> Self {
        Self {
            middlewares: Vec::new(),
            model: Box::new(model),
        }
    }

    /// Create a new chain from an already-boxed model.
    pub fn from_box(model: Box<dyn LanguageModel>) -> Self {
        Self {
            middlewares: Vec::new(),
            model,
        }
    }

    /// Create a new chain from middlewares and a boxed model.
    pub fn from_parts(
        middlewares: Vec<Box<dyn Middleware>>,
        model: Box<dyn LanguageModel>,
    ) -> Self {
        Self { middlewares, model }
    }

    /// Append a middleware to the chain and return `self` for fluent building.
    ///
    /// Middleware added first will be executed first (outermost).
    pub fn with(mut self, mw: impl Middleware + 'static) -> Self {
        self.middlewares.push(Box::new(mw));
        self
    }

    /// Append a boxed middleware to the chain.
    pub fn with_boxed(mut self, mw: Box<dyn Middleware>) -> Self {
        self.middlewares.push(mw);
        self
    }

    /// Execute the middleware chain, producing a [`GenerateResult`].
    pub async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        let next = MiddlewareNext {
            middlewares: &self.middlewares,
            model: self.model.as_ref(),
        };
        next.run(prompt, options).await
    }
}

#[async_trait]
impl LanguageModel for MiddlewareChain {
    fn model_id(&self) -> &str {
        self.model.model_id()
    }

    fn provider_id(&self) -> &str {
        self.model.provider_id()
    }

    fn capabilities(&self) -> &CapabilitySet {
        self.model.capabilities()
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        MiddlewareChain::generate(self, prompt, options).await
    }

    /// Streaming delegate to the inner model directly.
    /// Middleware chain currently only intercepts `generate` calls.
    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<crate::AiStream> {
        self.model.stream(prompt, options).await
    }
}

/// Wrap a language model with one or more middleware layers.
/// Equivalent to Vercel's `wrapLanguageModel()`.
pub fn wrap_language_model(
    model: Box<dyn LanguageModel>,
    middlewares: Vec<Box<dyn Middleware>>,
) -> Box<dyn LanguageModel> {
    if middlewares.is_empty() {
        return model;
    }
    Box::new(MiddlewareChain::from_parts(middlewares, model))
}
