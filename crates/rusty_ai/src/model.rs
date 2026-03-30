use async_trait::async_trait;

use crate::capability::CapabilitySet;
use crate::error::{AiError, AiResult};
use crate::prompt::Prompt;
use crate::schema::OutputSchema;
use crate::stream::AiStream;
use crate::structured::{EmbeddingResult, GenerateResult, ObjectResult};
use crate::tool::{ToolChoice, ToolDefinition};
use crate::types::RequestMetadata;

/// Options that control generation behaviour.
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub stop_sequences: Vec<String>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub seed: Option<u64>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub output_schema: Option<OutputSchema>,
    pub metadata: RequestMetadata,
}

impl GenerateOptions {
    /// Set the temperature.
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Set the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// Set top-p (nucleus sampling).
    pub fn with_top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Set top-k sampling.
    pub fn with_top_k(mut self, k: u32) -> Self {
        self.top_k = Some(k);
        self
    }

    /// Set stop sequences.
    pub fn with_stop_sequences(mut self, seqs: Vec<String>) -> Self {
        self.stop_sequences = seqs;
        self
    }

    /// Set frequency penalty.
    pub fn with_frequency_penalty(mut self, p: f64) -> Self {
        self.frequency_penalty = Some(p);
        self
    }

    /// Set presence penalty.
    pub fn with_presence_penalty(mut self, p: f64) -> Self {
        self.presence_penalty = Some(p);
        self
    }

    /// Set the random seed for reproducibility.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Provide tool definitions.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the tool choice strategy.
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set the output schema for structured generation.
    pub fn with_output_schema(mut self, schema: OutputSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Set request metadata.
    pub fn with_metadata(mut self, metadata: RequestMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Describes a provider backend.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub default_base_url: Option<String>,
}

/// The core language model trait that all providers implement.
#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Return the model identifier (e.g. "gpt-4o").
    fn model_id(&self) -> &str;

    /// Return the provider identifier (e.g. "openai").
    fn provider_id(&self) -> &str;

    /// Return the set of capabilities this model supports.
    fn capabilities(&self) -> &CapabilitySet;

    /// Generate a complete response.
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<GenerateResult>;

    /// Stream a response as a series of events.
    async fn stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<AiStream>;

}

/// Generate a structured object from a language model response.
///
/// Adds the JSON schema to the options, calls `generate`, and parses the text result.
pub async fn generate_object<T: serde::de::DeserializeOwned + schemars::JsonSchema>(
    model: &dyn LanguageModel,
    prompt: Prompt,
    options: GenerateOptions,
) -> AiResult<ObjectResult<T>> {
    let mut opts = options;
    opts.output_schema = Some(OutputSchema::from_type::<T>());
    let result = model.generate(prompt, opts).await?;
    let text = result
        .text
        .as_deref()
        .ok_or_else(|| AiError::Serialization("No text in response to parse as object".into()))?;
    let object: T = serde_json::from_str(text)
        .map_err(|e| AiError::Serialization(format!("Failed to parse response as object: {e}")))?;
    Ok(ObjectResult {
        object,
        text: text.to_owned(),
        usage: result.usage,
        metadata: result.metadata,
    })
}

/// A model that produces vector embeddings from text.
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Return the model identifier.
    fn model_id(&self) -> &str;

    /// Return the provider identifier.
    fn provider_id(&self) -> &str;

    /// Return the dimensionality of the embeddings, if known.
    fn dimensions(&self) -> Option<usize>;

    /// Embed a batch of texts into vectors.
    async fn embed(&self, texts: Vec<String>) -> AiResult<EmbeddingResult>;
}

/// Middleware sits between the caller and the model, intercepting generate calls.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a generate request.
    ///
    /// Implementations should call `next.run(prompt, options).await` to
    /// continue the chain, and may inspect / modify the prompt, options,
    /// or result.
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult>;
}

/// A handle to the next element in a middleware chain.
///
/// Calling `run` will invoke either the next middleware or the final model.
pub struct MiddlewareNext<'a> {
    pub middlewares: &'a [Box<dyn Middleware>],
    pub model: &'a dyn LanguageModel,
}

impl<'a> MiddlewareNext<'a> {
    /// Execute the next middleware (or the model if no middleware remains).
    pub async fn run(
        self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        if let Some((first, rest)) = self.middlewares.split_first() {
            let next = MiddlewareNext {
                middlewares: rest,
                model: self.model,
            };
            first.process(prompt, options, next).await
        } else {
            self.model.generate(prompt, options).await
        }
    }
}
