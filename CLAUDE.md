# Rusty AI SDK - Codebase Documentation

## Project Overview

Rusty AI is a comprehensive Rust SDK providing a unified, async-first interface for building AI applications with support for multiple cloud and local AI providers.

## High-Level Architecture

### Workspace Structure

The project is organized as a Rust workspace with the following main components:

```
rusty_ai/
├── crates/
│   ├── rusty_ai/                    # Core traits and abstractions
│   ├── rusty_middleware/            # Middleware layer
│   ├── rusty_ui_stream/             # UI streaming utilities
│   ├── rusty_testing/               # Testing utilities
│   ├── rusty_claude/                # Anthropic Claude provider
│   ├── rusty_chatgpt/               # OpenAI ChatGPT provider
│   ├── rusty_gemini/                # Google Gemini provider
│   ├── rusty_openai_compatible/     # Generic OpenAI-compatible endpoints
│   ├── rusty_ollama/                # Ollama local model support
│   ├── rusty_gemini_nano/           # On-device Gemini Nano
│   ├── rusty_phi_silica/            # Microsoft Phi Silica models
│   ├── rusty_foundationmodels/      # Foundation models
│   └── rusty_browser/               # Browser-based runtime
├── examples/
│   ├── basic_text/                  # Multi-provider text generation
│   ├── stream_text/                 # Streaming text responses
│   ├── generate_object/             # Structured output generation
│   ├── stream_object/               # Streaming structured output
│   ├── tool_loop/                   # Tool use and agent loops
│   ├── multimodal/                  # Images and media handling
│   ├── local_android/               # Android local execution
│   ├── local_apple/                 # Apple platform support
│   ├── local_windows/               # Windows platform support
│   └── router/                      # Multi-provider routing
└── Cargo.toml                       # Workspace manifest
```

## Core Crates

### `rusty_ai` (Core)

**Purpose**: Provides foundational traits, types, and abstractions that all providers implement.

**Key Components**:
- `Model` trait - Abstract interface for AI models
- `Provider` trait - Factory and configuration for model creation
- `GenerateRequest/GenerateResponse` - Request/response types
- `StreamEvent` - Streaming event types
- `Message`, `Content`, `Role` - Conversation abstractions
- `Tool` - Tool/function calling support

**Key Functions**:
- `generate_text()` - Simple text generation
- `stream_text()` - Streaming text with events
- `generate_object()` - Type-safe structured output
- `stream_object()` - Streaming structured outputs

**Dependencies**: Minimal - only core async/serialization libraries

### Provider Crates

#### `rusty_claude`

**Purpose**: Anthropic Claude API integration

**Implements**: `ClaudeProvider` with model variants (claude-sonnet, claude-opus, claude-haiku)

**Key Features**:
- Full Claude API support via reqwest
- Vision capabilities for image analysis
- Tool use with Claude's JSON schema format

#### `rusty_chatgpt`

**Purpose**: OpenAI ChatGPT integration

**Implements**: `ChatGptProvider` with model variants (gpt-4o, gpt-4o-mini, gpt-4-turbo)

**Key Features**:
- OpenAI API compatibility
- Vision support
- Tool calling with JSON schemas

#### `rusty_gemini`

**Purpose**: Google Gemini API integration

**Implements**: `GeminiProvider` with model variants

**Key Features**:
- Gemini API support
- Multimodal input handling
- Safety settings configuration

#### Other Providers

- **`rusty_openai_compatible`** - Generic interface for OpenAI-compatible endpoints (Ollama, local servers, etc.)
- **`rusty_ollama`** - Direct Ollama integration with local model management
- **`rusty_gemini_nano`** - On-device execution for mobile/edge
- **`rusty_phi_silica`** - Microsoft Phi models for efficient inference
- **`rusty_foundationmodels`** - Cross-provider foundation model support
- **`rusty_browser`** - Browser JavaScript runtime for in-browser inference

### Utility Crates

#### `rusty_middleware`

**Purpose**: Middleware layer for processing requests and responses

**Use Cases**:
- Request logging and tracing
- Response filtering and transformation
- Cross-cutting concerns (caching, retry logic, etc.)

#### `rusty_ui_stream`

**Purpose**: UI-optimized streaming utilities

**Features**:
- Buffer management for streaming
- Event batching for UI updates
- Progress tracking and status reporting

#### `rusty_testing`

**Purpose**: Testing utilities and mock providers

**Provides**:
- Mock implementations for testing
- Test fixtures
- Assertion helpers

## Design Patterns

### 1. Provider Pattern

Each cloud/local AI service has a corresponding `Provider` crate that:
- Implements the core `Model` trait
- Provides factory methods for creating model instances
- Handles authentication and configuration
- Implements the specific service's API

```rust
pub struct ClaudeProvider { /* ... */ }
impl ClaudeProvider {
    pub fn new(api_key: String) -> Self { /* ... */ }
    pub fn claude_sonnet(&self) -> impl Model { /* ... */ }
}
```

### 2. Trait-Based Abstraction

Core functionality is defined through traits that all providers implement:

```rust
pub trait Model: Send + Sync {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    async fn stream(&self, request: GenerateRequest) -> Result<BoxStream<StreamEvent>>;
}
```

This enables:
- Provider interchangeability
- Type-safe model operations
- Compile-time trait bounds

### 3. Async/Streaming Architecture

All I/O operations are async-first:
- Built on `tokio` runtime
- Streaming responses use `futures::Stream`
- Non-blocking request handling throughout

### 4. Type-Safe Structured Output

Uses `schemars` for generating JSON schemas from Rust types:
- Compile-time type safety
- Automatic schema generation for API calls
- Type-safe deserialization of responses

## Common Workflows

### Text Generation Flow

```
User Code
    ↓
Provider::generate_text()
    ↓
Model trait implementation
    ↓
HTTP request to API
    ↓
Stream/await response
    ↓
Return text/structured output
```

### Streaming Flow

```
User Code
    ↓
Provider::stream_text()
    ↓
Stream<StreamEvent> created
    ↓
Each API chunk → StreamEvent
    ↓
Consumer iterates events
```

### Tool Use Flow

```
User Code + Tools
    ↓
Model generates tool_use event
    ↓
Tool resolver executes requested tool
    ↓
Tool result added to conversation
    ↓
Model continues generation
    ↓
Final response
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

### Serialization
- `serde` - Core serialization framework
- `serde_json` - JSON support
- `schemars` - JSON Schema generation

### Observability
- `tracing` - Logging and diagnostics
- `uuid` - Request ID generation
- `chrono` - Timestamp handling

### Utilities
- `thiserror` - Error handling macros
- `secrecy` - Secure API key handling
- `base64` - Encoding/decoding
- `mime` - MIME type handling

## Configuration & Setup

### Environment Variables

Each provider uses standard environment variables:

```bash
ANTHROPIC_API_KEY=...    # Claude
OPENAI_API_KEY=...       # ChatGPT
GOOGLE_API_KEY=...       # Gemini
```

### Cargo Workspace Configuration

```toml
[workspace]
members = [
    "crates/*",
    "examples/*"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "MPL-2.0"
```

All crates inherit package metadata through workspace.package.

## Testing Strategy

The project uses:
- Unit tests within provider crates
- Integration tests in examples
- Mock implementations in `rusty_testing` for offline testing
- Real API tests with environment variable gating

## Development Guidelines

### Adding a New Provider

1. Create a new crate: `crates/rusty_new_provider/`
2. Implement the `Model` trait from `rusty_ai`
3. Create a `Provider` struct with factory methods
4. Implement authentication and API client
5. Add an example in `examples/` if significant differentiation
6. Update workspace Cargo.toml

### Code Structure

Each provider crate typically has:
```
src/
├── lib.rs           # Public API exports
├── provider.rs      # Provider struct and implementation
├── client.rs        # HTTP client
├── models.rs        # Request/response types
└── error.rs         # Error types
```

### Error Handling

Uses `thiserror` for ergonomic error definitions:
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}
```

## Performance Considerations

1. **Async Throughout** - All I/O is non-blocking
2. **Streaming Support** - Large responses don't load entirely in memory
3. **Connection Pooling** - Reqwest handles HTTP connection reuse
4. **Zero-Copy Where Possible** - Uses references and borrowing
5. **Lazy Initialization** - Providers created on-demand

## Security

1. **API Key Handling** - Uses `secrecy` crate for sensitive data
2. **HTTPS Only** - All provider APIs use HTTPS
3. **No Logging of Secrets** - Keys not included in traces/logs
4. **Safe Deserialization** - Serde with validation
5. **Input Validation** - Provider-specific safety settings

## Future Extensions

Potential areas for expansion:
- Caching layer for responses
- Rate limiting and backoff strategies
- Batch API support for multiple requests
- Vector database integration
- Prompt caching for long contexts
- Additional provider integrations

## References

- Tokio: https://tokio.rs/
- Reqwest: https://github.com/seanmonstar/reqwest
- Serde: https://serde.rs/
- Schemars: https://github.com/GREsau/schemars
