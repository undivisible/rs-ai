# RAI - Rust AI SDK - Codebase Documentation

## Project Overview

RAI (Rust AI) is a comprehensive Rust SDK providing a unified, async-first interface for building AI applications with support for 15+ cloud and local AI providers, with real-time capabilities, streaming tool use, and a simplified API.

**Key Update (2026)**: The project was renamed from `rusty_ai` to `rai` and completely refactored with a simplified API and expanded provider support.

## High-Level Architecture

### Workspace Structure

The project is organized as a Rust workspace with core traits, provider implementations, and examples:

```
rai/
├── crates/
│   ├── rai_ai/                    # Core traits, types, simplified API
│   ├── rai_middleware/            # Middleware layer for requests/responses
│   ├── rai_ui_stream/             # UI streaming utilities
│   ├── rai_testing/               # Testing utilities and mocks
│   
│   # Cloud Providers
│   ├── rai_claude/                # Anthropic Claude (tool streaming, vision)
│   ├── rai_chatgpt/               # OpenAI ChatGPT (Realtime API support)
│   ├── rai_gemini/                # Google Gemini (Live API with audio/video)
│   ├── rai_openai_compatible/     # Generic OpenAI-compatible + 9 presets
│   
│   # Local/Platform Runtimes
│   ├── rai_ollama/                # Ollama - local model runner
│   ├── rai_gemini_nano/           # On-device Gemini Nano
│   ├── rai_phi_silica/            # Microsoft Phi Silica
│   ├── rai_foundationmodels/      # Cross-provider foundation models
│   └── rai_browser/               # Browser JavaScript runtime
│   
├── examples/
│   ├── basic_text/                # Multi-provider text generation
│   ├── stream_text/               # Streaming text responses
│   ├── generate_object/           # Structured output generation
│   ├── stream_object/             # Streaming structured output
│   ├── tool_loop/                 # Tool use and agent loops
│   ├── multimodal/                # Images and media handling
│   ├── local_android/             # Android local execution
│   ├── local_apple/               # Apple platform support
│   ├── local_windows/             # Windows platform support
│   └── router/                    # Multi-provider routing
└── Cargo.toml                     # Workspace manifest
```

## Core Crates

### `rai_ai` (Core)

**Purpose**: Provides foundational traits, types, abstractions, and a simplified API.

**Key Components**:

Core traits:
- `LanguageModel` trait - Core interface for all AI models
- `Provider` trait - Factory for model creation
- `EmbeddingModel`, `TextToSpeechModel`, `SpeechToTextModel` - Specialized model types

Core types:
- `GenerateRequest/GenerateResponse` - Request/response types
- `StreamEvent` - Streaming event types with TextDelta, ToolUse, MessageEnd variants
- `Message`, `Role`, `Content` - Conversation abstractions
- `ToolDefinition`, `ToolSet` - Tool/function calling support
- `FinishReason` - Model completion status

Simplified API (`simple` module):
- `SimpleModel` - Wrapper for easy usage
- `rai_claude(model_id)` - Quick Claude access
- `rai_chatgpt(model_id)` - Quick ChatGPT access
- `rai_gemini(model_id)` - Quick Gemini access
- `rai_compatible()` - Generic OpenAI-compatible access

**Key Functions**:
- `generate_text()` - Simple text generation
- `stream_text()` - Streaming text with events
- `generate_object()` - Type-safe structured output
- `embed()` - Text embedding

**Dependencies**: Minimal - core async/serialization libraries

### Provider Crates

#### `rai_claude` - Anthropic Claude

**Features**:
- Full Claude API support via reqwest
- Streaming tool use with real-time function calling
- Vision capabilities for image analysis
- Tool use with Claude's JSON schema format
- Support for claude-sonnet-4-6, claude-opus-4, claude-haiku

#### `rai_chatgpt` - OpenAI ChatGPT

**Features**:
- OpenAI API compatibility
- Realtime API support (voice agents, audio I/O)
- Vision support with image inputs
- Tool calling with function definitions
- Support for gpt-4o, gpt-4o-mini, gpt-4-turbo

#### `rai_gemini` - Google Gemini

**Features**:
- Gemini 2.5 Flash support
- Gemini Live API for low-latency voice/vision conversations
- 30 HD voices in 24 languages
- Natural speech output with barge-in support
- Multimodal input handling
- Safety settings configuration

#### `rai_openai_compatible` - Generic OpenAI-Compatible Endpoints

**Pre-configured Provider Presets**:
1. **OpenRouter** - 300+ models from all providers
2. **Kilo AI Gateway** - Cost management and routing
3. **Amazon Bedrock** - AWS-managed models
4. **Together AI** - Optimized open-source models
5. **OctoML** - Serverless model hosting
6. **Azure OpenAI** - Microsoft-hosted models
7. **Cloudflare Workers AI** - Edge network inference
8. **vLLM** - Self-hosted high-throughput inference
9. **Ollama** - Self-hosted local model runner

**Features**:
- Drop-in compatible with OpenAI API
- Support for all model types (text, vision, audio)
- Tool/function calling
- Streaming responses

#### Other Providers

- **`rai_ollama`** - Direct Ollama integration with local model management
- **`rai_gemini_nano`** - On-device execution for mobile/edge
- **`rai_phi_silica`** - Microsoft Phi models for efficient inference
- **`rai_foundationmodels`** - Cross-provider foundation model support
- **`rai_browser`** - Browser JavaScript runtime for in-browser inference

### Utility Crates

#### `rai_middleware`

**Purpose**: Middleware layer for processing requests and responses

**Use Cases**:
- Request logging and tracing
- Response filtering and transformation
- Cross-cutting concerns (caching, retry logic, etc.)
- Rate limiting

#### `rai_ui_stream`

**Purpose**: UI-optimized streaming utilities

**Features**:
- Buffer management for streaming
- Event batching for UI updates
- Progress tracking and status reporting
- Token counting

#### `rai_testing`

**Purpose**: Testing utilities and mock providers

**Provides**:
- Mock implementations for testing
- Test fixtures
- Assertion helpers
- Fake streaming responses

## Design Patterns

### 1. Simplified API Pattern

The `simple` module provides convenient factory functions:

```rust
let ai = rai_claude("claude-sonnet-4-6").await?;
let result = ai.generate("What is 2+2?").await?;
```

These wrap the traditional provider pattern for common use cases.

### 2. Provider Pattern

Each service has a Provider that:
- Implements the core `LanguageModel` trait
- Provides factory methods for model variants
- Handles authentication and configuration
- Implements the specific service's API

```rust
let provider = ClaudeProvider::new(api_key);
let model = provider.claude_sonnet();
generate_text(&model, "prompt").await?;
```

### 3. Trait-Based Abstraction

Core functionality is defined through traits:

```rust
pub trait LanguageModel: Send + Sync {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    async fn stream(&self, request: GenerateRequest) -> Result<BoxStream<StreamEvent>>;
}
```

This enables:
- Provider interchangeability
- Type-safe model operations
- Compile-time trait bounds

### 4. Async/Streaming Architecture

All I/O is async-first:
- Built on `tokio` runtime
- Streaming responses via `futures::Stream`
- Non-blocking throughout

### 5. Real-Time Capabilities

Three new real-time patterns:

**Streaming Tool Use**:
```rust
// Tools are streamed in real-time, not batched
while let Some(StreamEvent::ToolUse { tool_name, input }) = stream.next().await {
    let result = execute_tool(&tool_name, input).await?;
    // Continue streaming with result
}
```

**Gemini Live API**:
```rust
// WebSocket connection for low-latency voice/video
let live_session = gemini_model.create_live_session().await?;
live_session.send_audio(audio_bytes).await?;
let response_audio = live_session.receive_audio().await?;
```

**OpenAI Realtime API**:
```rust
// WebSocket for voice agents with function calling
let session = chatgpt_model.create_realtime_session().await?;
session.send_audio(audio).await?;
let audio_response = session.receive_audio().await?;
```

### 6. Type-Safe Structured Output

Uses `schemars` for JSON schema generation:

```rust
#[derive(Deserialize, schemars::JsonSchema)]
struct BookReview {
    title: String,
    rating: u8,
}

let review: BookReview = generate_object(&model, "Review this book").await?;
```

## Common Workflows

### Simple Text Generation (New Simplified API)

```
User Code
    ↓
rai_claude(model_id)
    ↓
SimpleModel wrapper
    ↓
generate() call
    ↓
Claude API request
    ↓
Return text result
```

### Traditional Text Generation (Advanced API)

```
User Code
    ↓
Provider::new()
    ↓
provider.model_variant()
    ↓
generate_text() helper
    ↓
Model::generate() trait impl
    ↓
HTTP request to API
    ↓
Return text
```

### Streaming with Tool Use

```
User Code
    ↓
stream_text() or stream_object()
    ↓
Stream<StreamEvent> created
    ↓
Each API chunk → StreamEvent
    ↓
TextDelta events for text
    ↓
ToolUse events for function calls
    ↓
Tool executed, result sent back
    ↓
Continue streaming until MessageEnd
```

### Real-Time Voice (Gemini Live / ChatGPT Realtime)

```
User Code
    ↓
create_live_session() or create_realtime_session()
    ↓
WebSocket connection established
    ↓
send_audio(bytes) / send_text(text)
    ↓
receive_audio() / receive_text()
    ↓
Real-time streaming of responses
    ↓
Tool calls in voice stream (Realtime only)
```

## Key Dependencies

### Async Runtime
- `tokio` - Full feature set for async execution
- `futures` - Stream and async utilities
- `async-trait` - Async trait support

### HTTP & Streaming
- `reqwest` - HTTP client with streaming
- `reqwest-eventsource` - Server-Sent Events parsing
- `eventsource-stream` - Event stream handling
- `tokio-tungstenite` - WebSocket for real-time APIs

### Serialization
- `serde` - Core serialization framework
- `serde_json` - JSON support
- `schemars` - JSON Schema generation for typed outputs

### Observability
- `tracing` - Logging and diagnostics
- `uuid` - Request ID generation
- `chrono` - Timestamp handling

### Utilities
- `thiserror` - Error handling macros
- `secrecy` - Secure API key handling
- `base64` - Encoding/decoding
- `mime` - MIME type handling
- `url` - URL parsing

## Configuration & Setup

### Environment Variables

Each provider uses standard environment variables:

```bash
# Cloud Providers
ANTHROPIC_API_KEY=sk-...      # Claude
OPENAI_API_KEY=sk-...         # ChatGPT
GOOGLE_API_KEY=...            # Gemini

# Compatible Routers
OPENROUTER_KEY=sk-...         # OpenRouter
KILO_API_KEY=...              # Kilo
TOGETHER_API_KEY=...          # Together

# AWS (Bedrock)
AWS_REGION=us-west-2
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
```

### Cargo Workspace Configuration

```toml
[workspace]
members = ["crates/*", "examples/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "MPL-2.0"
```

All crates inherit package metadata through workspace.package.

## Testing Strategy

The project includes:
- **Unit tests** within each provider crate
- **Integration tests** in examples/
- **Preset tests** in rai_openai_compatible validating all 9 presets
- **Mock implementations** in `rai_testing` for offline testing
- **Real API tests** gated by environment variables

### Running Tests

```bash
# All tests
cargo test

# Provider-specific tests
cargo test -p rai_claude
cargo test -p rai_openai_compatible

# Integration tests only
cargo test --test '*'
```

## Development Guidelines

### Adding a New Provider

1. Create a new crate: `crates/rai_new_provider/`
2. Implement the `LanguageModel` trait from `rai_ai`
3. Create a `Provider` struct with factory methods
4. Implement authentication and API client
5. Add streaming support via StreamEvent
6. Add tool use support via ToolDefinition
7. Create tests in `tests/` directory
8. Add an example in `examples/` if significant
9. Update workspace Cargo.toml

### Code Structure (Per Provider)

```
src/
├── lib.rs           # Public API exports
├── provider.rs      # Provider struct and factory methods
├── model.rs         # Model implementation of LanguageModel trait
├── client.rs        # HTTP client and API calls
├── stream.rs        # Streaming implementation
├── tool.rs          # Tool/function calling
├── convert.rs       # Type conversions
├── error.rs         # Error types
└── types.rs         # Request/response structures
```

### Error Handling

Uses `thiserror` for ergonomic errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Streaming error: {0}")]
    StreamError(String),
}
```

### Adding Tests

For each provider, create `tests/` directory with:
- `integration_tests.rs` - Full workflow tests
- `streaming_tests.rs` - Streaming response tests
- `tool_tests.rs` - Tool/function calling tests

## Performance Considerations

1. **Async Throughout** - All I/O is non-blocking via tokio
2. **Streaming Support** - Large responses don't load entirely in memory
3. **Connection Pooling** - Reqwest handles HTTP connection reuse
4. **Zero-Copy Where Possible** - Uses references and borrowing
5. **Lazy Initialization** - Providers and models created on-demand
6. **Real-Time Efficiency** - WebSocket connections for low-latency APIs

## Security

1. **API Key Handling** - Uses `secrecy` crate for sensitive data
2. **HTTPS Only** - All provider APIs use HTTPS
3. **WSS for Real-Time** - WebSocket connections are encrypted
4. **No Logging of Secrets** - Keys not included in traces/logs
5. **Safe Deserialization** - Serde with validation
6. **Input Validation** - Provider-specific safety settings

## Recent Features (2026)

### New Simplified API
- Single-function access to each provider
- Zero boilerplate for common use cases
- Automatic environment variable handling

### Real-Time Capabilities
- **Gemini Live API** - Voice/video conversations with function calling
- **OpenAI Realtime API** - Voice agents with low-latency streaming
- **Streaming Tool Use** - Tools streamed in real-time, not batched

### Expanded Provider Support
- 9 OpenAI-compatible presets (OpenRouter, Bedrock, Kilo, Together, OctoML, Azure, Cloudflare, vLLM, Ollama)
- Bedrock-specific configuration
- Local model runners (vLLM, Ollama, LM Studio, text-generation-webui)

### Improved Testing
- Comprehensive preset validation tests
- Integration test suite for each provider
- Mock implementations for offline testing

## Future Extensions

Potential areas for expansion:
- Response caching with TTL
- Automatic retry with exponential backoff
- Rate limiting per provider
- Batch API support for multiple requests
- Vector database integration
- Prompt optimization and compression
- Cost tracking and optimization
- Multi-provider load balancing

## References

- Claude API: https://docs.anthropic.com/en/docs/build-with-claude/tool-use
- OpenAI Realtime API: https://developers.openai.com/api/docs/guides/realtime
- Gemini Live API: https://ai.google.dev/gemini-api/docs/live-api
- OpenRouter: https://openrouter.ai/docs/api/api-reference
- Amazon Bedrock: https://platform.claude.com/docs/en/build-with-claude/claude-in-amazon-bedrock
- Kilo Gateway: https://kilo.ai/docs/gateway/api-reference
- Tokio: https://tokio.rs/
- Serde: https://serde.rs/
- Schemars: https://github.com/GREsau/schemars
