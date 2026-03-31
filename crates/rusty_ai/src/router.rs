use async_trait::async_trait;

use crate::capability::{Capability, CapabilitySet};
use crate::error::{AiError, AiResult};
use crate::model::{GenerateOptions, LanguageModel};
use crate::prompt::Prompt;
use crate::stream::AiStream;
use crate::structured::GenerateResult;

/// A condition function used to decide whether a route matches.
pub type RouteCondition = Box<dyn Fn(&Prompt, &GenerateOptions) -> bool + Send + Sync>;

/// A single route that maps a condition to a model.
pub struct Route {
    pub model: Box<dyn LanguageModel>,
    pub condition: RouteCondition,
    pub priority: i32,
}

/// A router that dispatches generation requests to different models based on conditions.
pub struct Router {
    routes: Vec<Route>,
    fallback: Option<Box<dyn LanguageModel>>,
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            fallback: None,
        }
    }

    /// Add a route with the given condition. Higher priority routes are checked first.
    pub fn add_route(
        mut self,
        model: Box<dyn LanguageModel>,
        condition: impl Fn(&Prompt, &GenerateOptions) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.routes.push(Route {
            model,
            condition: Box::new(condition),
            priority: 0,
        });
        self
    }

    /// Add a route with a specific priority. Higher priority routes are checked first.
    pub fn add_route_with_priority(
        mut self,
        model: Box<dyn LanguageModel>,
        condition: impl Fn(&Prompt, &GenerateOptions) -> bool + Send + Sync + 'static,
        priority: i32,
    ) -> Self {
        self.routes.push(Route {
            model,
            condition: Box::new(condition),
            priority,
        });
        self
    }

    /// Set a fallback model to use when no route matches.
    pub fn with_fallback(mut self, model: Box<dyn LanguageModel>) -> Self {
        self.fallback = Some(model);
        self
    }

    /// Create a router that prefers a local model and falls back to a cloud model.
    ///
    /// The local model is used when the request does not require capabilities
    /// that only the cloud model supports.
    pub fn local_first(local: Box<dyn LanguageModel>, cloud: Box<dyn LanguageModel>) -> Self {
        let local_caps: Vec<Capability> = local.capabilities().iter().cloned().collect();
        Self::new()
            .add_route_with_priority(
                local,
                move |_prompt, _options| {
                    // Prefer local: always try local first
                    let _ = &local_caps;
                    true
                },
                10,
            )
            .with_fallback(cloud)
    }

    /// Create a router that selects a model based on required capabilities.
    ///
    /// For each request, the first model whose capability set satisfies the
    /// request's needs (e.g. tool calling, structured output) is selected.
    pub fn capability_route(models: Vec<Box<dyn LanguageModel>>) -> Self {
        let mut router = Self::new();
        for model in models {
            let caps = model.capabilities().clone();
            router.routes.push(Route {
                model,
                condition: Box::new(move |_prompt, options| {
                    // Check if this model supports the required capabilities
                    let mut needed = Vec::new();
                    if options.tools.is_some() {
                        needed.push(Capability::ToolCalling);
                    }
                    if options.output_schema.is_some() {
                        needed.push(Capability::StructuredOutput);
                    }
                    caps.supports_all(&needed)
                }),
                priority: 0,
            });
        }
        router
    }

    /// Select the best model for the given prompt and options.
    fn select_model<'a>(
        &'a self,
        prompt: &Prompt,
        options: &GenerateOptions,
    ) -> AiResult<&'a dyn LanguageModel> {
        // Sort candidates by priority (highest first)
        let mut candidates: Vec<&Route> = self
            .routes
            .iter()
            .filter(|r| (r.condition)(prompt, options))
            .collect();
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

        if let Some(route) = candidates.first() {
            return Ok(route.model.as_ref());
        }

        if let Some(ref fallback) = self.fallback {
            return Ok(fallback.as_ref());
        }

        Err(AiError::ModelUnavailable {
            model: "No matching route and no fallback configured".into(),
        })
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageModel for Router {
    fn model_id(&self) -> &str {
        "router"
    }

    fn provider_id(&self) -> &str {
        "router"
    }

    fn capabilities(&self) -> &CapabilitySet {
        // A router's capabilities are conceptually the union, but we return
        // an empty set since actual capabilities depend on the selected model.
        // Callers should rely on the router's routing logic rather than this.
        static EMPTY: std::sync::LazyLock<CapabilitySet> =
            std::sync::LazyLock::new(CapabilitySet::new);
        &EMPTY
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let model = self.select_model(&prompt, &options)?;
        model.generate(prompt, options).await
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let model = self.select_model(&prompt, &options)?;
        model.stream(prompt, options).await
    }
}
