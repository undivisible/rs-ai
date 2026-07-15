//! Model router for dispatching requests.
use async_trait::async_trait;
use std::cmp::Reverse;

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
    /// The language model to use when this route matches.
    pub model: Box<dyn LanguageModel>,
    /// Function that decides if the route matches a given request.
    pub condition: RouteCondition,
    /// Higher values are checked first.
    pub priority: i32,
}

/// A router that dispatches generation requests to different models based on conditions.
pub struct Router {
    /// Registered routes.
    routes: Vec<Route>,
    /// Fallback model when no route matches.
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
    /// The local model is selected when its capabilities satisfy the request
    /// (e.g. it supports tool calling when tools are provided). When the local
    /// model cannot satisfy the request, the cloud model is used as a fallback.
    pub fn local_first(local: Box<dyn LanguageModel>, cloud: Box<dyn LanguageModel>) -> Self {
        let local_caps = local.capabilities().clone();
        Self::new()
            .add_route_with_priority(
                local,
                move |_prompt, options| {
                    let mut needed = Vec::new();
                    if options.tools.is_some() {
                        needed.push(Capability::ToolCalling);
                    }
                    if options.output_schema.is_some() {
                        needed.push(Capability::StructuredOutput);
                    }
                    local_caps.supports_all(&needed)
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
        candidates.sort_by_key(|b| Reverse(b.priority));

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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::schema::OutputSchema;
    use crate::structured::GenerateResult;
    use crate::tool::ToolDefinition;
    use crate::types::{FinishReason, ResponseMetadata};
    use crate::usage::Usage;

    // Minimal inline mock that records which model was called.
    struct TrackingModel {
        id: &'static str,
        capabilities: CapabilitySet,
        called: Arc<Mutex<Vec<String>>>,
    }

    impl TrackingModel {
        fn new(
            id: &'static str,
            capabilities: CapabilitySet,
            log: Arc<Mutex<Vec<String>>>,
        ) -> Self {
            Self {
                id,
                capabilities,
                called: log,
            }
        }
    }

    #[async_trait]
    impl LanguageModel for TrackingModel {
        fn model_id(&self) -> &str {
            self.id
        }

        fn provider_id(&self) -> &str {
            "mock"
        }

        fn capabilities(&self) -> &CapabilitySet {
            &self.capabilities
        }

        async fn generate(
            &self,
            _prompt: Prompt,
            _options: GenerateOptions,
        ) -> AiResult<GenerateResult> {
            self.called.lock().unwrap().push(self.id.to_string());
            Ok(GenerateResult {
                text: Some(self.id.to_string()),
                tool_calls: Vec::new(),
                finish_reason: FinishReason::Stop,
                usage: Usage::default(),
                metadata: ResponseMetadata::default(),
                steps: Vec::new(),
                reasoning: None,
            })
        }

        async fn stream(&self, _prompt: Prompt, _options: GenerateOptions) -> AiResult<AiStream> {
            unimplemented!()
        }
    }

    fn local_caps_only() -> CapabilitySet {
        CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
    }

    fn full_caps() -> CapabilitySet {
        CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::ToolCalling)
            .with(Capability::StructuredOutput)
    }

    #[tokio::test]
    async fn local_first_selects_local_for_plain_text() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let local = TrackingModel::new("local", local_caps_only(), log.clone());
        let cloud = TrackingModel::new("cloud", full_caps(), log.clone());

        let router = Router::local_first(Box::new(local), Box::new(cloud));
        let result = router
            .generate(Prompt::from("hello"), GenerateOptions::default())
            .await
            .unwrap();

        assert_eq!(
            result.text.as_deref(),
            Some("local"),
            "plain text should route to local"
        );
        assert_eq!(*log.lock().unwrap(), vec!["local"]);
    }

    #[tokio::test]
    async fn local_first_falls_back_to_cloud_when_tools_needed_and_local_lacks_tool_calling() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let local = TrackingModel::new("local", local_caps_only(), log.clone());
        let cloud = TrackingModel::new("cloud", full_caps(), log.clone());

        let router = Router::local_first(Box::new(local), Box::new(cloud));
        let options = GenerateOptions::default().with_tools(vec![ToolDefinition {
            name: "search".into(),
            description: "web search".into(),
            parameters: serde_json::json!({}),
            examples: None,
        }]);
        let result = router
            .generate(Prompt::from("search for rust"), options)
            .await
            .unwrap();

        assert_eq!(
            result.text.as_deref(),
            Some("cloud"),
            "tool call should route to cloud when local lacks ToolCalling"
        );
        assert_eq!(*log.lock().unwrap(), vec!["cloud"]);
    }

    #[tokio::test]
    async fn local_first_selects_local_when_local_supports_tools() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let local = TrackingModel::new("local", full_caps(), log.clone());
        let cloud = TrackingModel::new("cloud", full_caps(), log.clone());

        let router = Router::local_first(Box::new(local), Box::new(cloud));
        let options = GenerateOptions::default().with_tools(vec![ToolDefinition {
            name: "search".into(),
            description: "web search".into(),
            parameters: serde_json::json!({}),
            examples: None,
        }]);
        let result = router
            .generate(Prompt::from("use tool"), options)
            .await
            .unwrap();

        assert_eq!(
            result.text.as_deref(),
            Some("local"),
            "should prefer local when it supports the required capabilities"
        );
        assert_eq!(*log.lock().unwrap(), vec!["local"]);
    }

    #[tokio::test]
    async fn local_first_falls_back_when_structured_output_needed_and_local_lacks_it() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let local = TrackingModel::new("local", local_caps_only(), log.clone());
        let cloud = TrackingModel::new("cloud", full_caps(), log.clone());

        let router = Router::local_first(Box::new(local), Box::new(cloud));
        let options = GenerateOptions::default().with_output_schema(OutputSchema::from_value(
            serde_json::json!({"type": "object"}),
        ));
        let result = router
            .generate(Prompt::from("give me json"), options)
            .await
            .unwrap();

        assert_eq!(
            result.text.as_deref(),
            Some("cloud"),
            "structured output should route to cloud when local lacks StructuredOutput"
        );
        assert_eq!(*log.lock().unwrap(), vec!["cloud"]);
    }
}
