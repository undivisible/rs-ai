use rusty_ai::{
    AiResult, GenerateOptions, GenerateResult, LanguageModel, Middleware, MiddlewareNext, Prompt,
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

    /// Append a middleware to the chain and return `self` for fluent building.
    ///
    /// Middleware added first will be executed first (outermost).
    pub fn with(mut self, mw: impl Middleware + 'static) -> Self {
        self.middlewares.push(Box::new(mw));
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
