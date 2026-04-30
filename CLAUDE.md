# rs_ai — Codebase Guide

## Overview

`rs_ai` is a Rust workspace SDK for AI applications. The top-level `rs_ai` crate exposes a fluent builder API (`rs_ai_claude()`, `rs_ai_gemini()`, etc.) over consolidated provider and core crates.

## Workspace Layout

```
rs_ai/
├── Cargo.toml
├── crates/
│   ├── rs_ai_core/          # Core traits, types, middleware, cache, UI stream, observability
│   ├── rs_ai_providers/     # Cloud AI providers (feature-gated)
│   ├── rs_ai_local/         # Platform-specific local runtimes (feature-gated)
│   ├── rs_ai_testing/       # MockLanguageModel, MockProvider
│   └── rs_ai/               # Fluent top-level API
└── examples/
    ├── fluent_api/          # rs_ai_claude() / rs_ai_gemini() usage
    ├── basic_text/
    ├── stream_text/
    ├── streaming_tool_use/
    ├── tool_loop/
    ├── agent_with_tools/
    ├── generate_object/
    ├── stream_object/
    ├── multimodal/
    ├── gemini_live/
    ├── realtime_voice/
    ├── router/
    ├── local_apple/
    ├── local_android/
    └── local_windows/
```

## Core Crate: `rs_ai_core`

Defines the traits every provider implements, plus middleware, cache config, UI stream helpers, and observability:

```rust
pub trait LanguageModel: Send + Sync {
    fn model_id(&self) -> &str;
    fn provider_id(&self) -> &str;
    fn capabilities(&self) -> &CapabilitySet;
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult>;
    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream>;
}

pub trait Provider {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>>;
}
```

Key types: `Prompt`, `GenerateResult`, `StreamEvent`, `AiError`, `Usage`, `FinishReason`, `CapabilitySet`, `ToolDefinition`.

Also re-exports:
- `cache::{CacheConfig, CacheTTL}`
- `middleware::{CacheMiddleware, LoggingMiddleware, RetryMiddleware, MiddlewareChain}`
- `ui_stream::{UiStreamEvent, NdjsonEncoder, SseEncoder}`
- `observability::{with_observability, ObservableModel}`

## Providers Crate: `rs_ai_providers`

All cloud providers merged into one crate, gated behind feature flags:

```toml
[features]
default = []
claude = []
chatgpt = []
gemini = []
openai-compatible = []
xai = []
cloudflare = []
ollama = []
portkey = []
langfuse = []
```

Each provider is a module under `rs_ai_providers::`:
- `claude` — Anthropic Claude (`ClaudeProvider`)
- `chatgpt` — OpenAI ChatGPT (`ChatGptProvider`)
- `gemini` — Google Gemini (`GeminiProvider`)
- `openai_compatible` — Generic OpenAI-compatible adapter (`OpenAiCompatibleProvider`)
- `xai` — xAI Grok (`XaiProvider`)
- `cloudflare` — Cloudflare Workers AI (`CloudflareProvider`)
- `ollama` — Local Ollama (`OllamaProvider`)
- `portkey` — Portkey AI Gateway (`PortkeyProvider`)
- `langfuse` — Langfuse observability wrapper (`with_langfuse`)

## Local Crate: `rs_ai_local`

Platform-specific local AI runtimes, gated behind feature flags:

```toml
[features]
default = []
browser = ["dep:wasm-bindgen", "dep:wasm-bindgen-futures", "dep:js-sys", "dep:web-sys"]
gemini-nano = ["dep:jni", "dep:uniffi"]
foundationmodels = ["dep:uniffi"]
phi-silica = []
```

- `browser` — WASM browser AI (Chrome/Edge built-in AI)
- `gemini_nano` — Android Gemini Nano (JNI, 1-line Kotlin init)
- `foundationmodels` — Apple Foundation Models (macOS)
- `phi_silica` — Windows Phi Silica (C# bridge via `build.rs`)

## Local Runtimes: User Setup

### Android (Gemini Nano)

Add to `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.google.mlkit:genai-prompt:1.0.0-beta2")
    implementation("net.java.dev.jna:jna:5.12.0@aar")
}
```

Add 1 line to your `MainActivity`:

```kotlin
class MainActivity : Activity() {
    companion object {
        init { System.loadLibrary("rs_ai_local") }
        @JvmStatic external fun init(context: Context)
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        init(this)  // Calls our init - that's it!
    }
}
```

Then use from Rust:

```rust
use rs_ai_local::gemini_nano::GeminiNanoProvider;

let provider = GeminiNanoProvider::new(my_bridge);
let model = provider.model();
let response = model.generate("Hello!").await?;
```

### iOS/macOS (Foundation Models)

Enable feature, build on macOS (Xcode 16+, macOS 26+ SDK):

```toml
rs_ai_local = { version = "0.2", features = ["foundationmodels"] }
```

Use from Rust:

```rust
use rs_ai_local::foundationmodels::{is_available, respond};

if is_available() {
    let response = respond("What is Rust?").await?;
}
```

### Windows (Phi Silica)

Enable feature (requires .NET SDK):

```toml
rs_ai_local = { version = "0.2", features = ["phi-silica"] }
```

Use from C# (calls `build.rs`-compiled DLL):

```csharp
using RsAi;

var response = await PhiSilicaBridge.GenerateAsync("Hello!");
Console.WriteLine(response);
```

## Top-Level API: `rs_ai`

### Entry points (all in `crates/rs_ai/src/lib.rs`)

| Function | Provider | Default env var |
|---|---|---|
| `rs_ai_claude()` | Anthropic Claude | `ANTHROPIC_API_KEY` |
| `rs_ai_chatgpt()` | OpenAI ChatGPT | `OPENAI_API_KEY` |
| `rs_ai_gemini()` | Google Gemini | `GOOGLE_API_KEY` |
| `rs_ai_xai()` | xAI Grok | `XAI_API_KEY` |
| `rs_ai_cloudflare(account_id)` | Cloudflare Workers AI | `CLOUDFLARE_API_TOKEN` |
| `rs_ai_compatible(base_url)` | Any OpenAI-compatible | `OPENAI_API_KEY` |

### `ClientBuilder` methods

```
.api_key(key)            Override env-var key
.model(id)               Required — model identifier string
.with_image(url_or_path) Attach image (URL or local file, base64-encoded)
.cf_ai_gateway(gw)       Route through Cloudflare AI Gateway
.with_cache(config)      Apply CacheConfig
.enable_cache()          Enable default ephemeral cache
.with_xai_conv_id(id)    xAI conversation routing
.with_prompt_cache_key(k) xAI/OpenAI prompt cache key
.generate(prompt)        → AiResult<String>
.stream(prompt)          → AiResult<BoxStream<AiResult<String>>>
.speak(text)             → AiResult<Vec<u8>>  (TTS, ChatGPT only)
.transcribe(audio)       → AiResult<String>   (STT, ChatGPT only)
```

### API key fallback (in `build()`)

If `.api_key()` is not called, `build()` reads the provider's env var. If the env var is absent, an empty string is sent (server will return 401 which surfaces as `AiError`).

## Provider Details

### Claude — `rs_ai_providers::claude`

- `ClaudeProvider::new(api_key)` / `.with_base_url(url)`
- Convenience: `.claude_sonnet()`, `.claude_opus()`, `.claude_haiku()`
- `ClaudeModel::set_cache(config)` — applies `cache_control` to requests
- Streaming via SSE; tool use; vision; extended thinking

### ChatGPT — `rs_ai_providers::chatgpt`

- `ChatGptProvider::new(api_key)` / `.with_org(org_id)`
- Convenience: `.gpt4o()`, `.gpt4o_mini()`, `.gpt54()`, `.gpt54_mini()`, `.gpt54_nano()`
- Realtime API (`RealtimeSession`) for voice agents
- Structured output, vision, audio I/O

### Gemini — `rs_ai_providers::gemini`

- `GeminiProvider::new(api_key)`
- Convenience: `.gemini_flash()`, `.gemini_pro()`, `.gemini_flash_lite()`, `.gemini_3_flash()`, `.gemini_31_pro()`
- `.model_with_base_url(id, url)` for proxy/CF gateway
- Live API (`provider.live_session(model_id)`) for low-latency voice/video

### xAI — `rs_ai_providers::xai`

- `XaiProvider::new(api_key)`
- Convenience: `.grok_4()`, `.grok_4_20_reasoning()`
- `XaiModel::set_cache(config)` — sets `x-grok-conv-id` headers and prompt cache keys

### OpenAI Compatible — `rs_ai_providers::openai_compatible`

- `OpenAiCompatibleProvider::new(config, id, name)`
- `OpenAiCompatibleConfig` presets: `openai()`, `openrouter()`, `kilo()`, `bedrock()`, `together()`, `vllm(url)`, `ollama_openai(url)`, `lmstudio(url)`, `textgen(url)`

### Ollama — `rs_ai_providers::ollama`

- `OllamaProvider::new()` / `.with_base_url(url)` (default `http://localhost:11434`)
- `provider.list_models()` — async model discovery

### Cloudflare — `rs_ai_providers::cloudflare`

- `CloudflareProvider::new(account_id, api_key)`
- Routes to `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/run/{model}`

## Cache Config — `rs_ai_core::cache`

```rust
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl: CacheTTL,
    pub claude_ephemeral: bool,
    pub gemini_cache_key: Option<String>,
    pub openai_cache_key: Option<String>,
    pub xai_conv_id: Option<String>,
    pub prompt_cache_key: Option<String>,
}
```

`CacheTTL` variants: `FiveMinutes` (default), `OneHour`, `TwentyFourHours`, `InMemory`, `Custom(u64)`.

## Langfuse — `rs_ai_providers::langfuse`

Wraps any `LanguageModel` to emit traces to LangFuse without blocking:

```rust
let observable = with_langfuse(model, "pub_key", "secret_key").await?;
```

- `LangfuseModel::with_trace_id()`, `.with_user_id()`, `.with_session_id()`
- Non-blocking via `tokio::spawn` — failed traces log a warning only

## Portkey — `rs_ai_providers::portkey`

Portkey AI Gateway provider:

```rust
let provider = PortkeyProvider::new("pk_...");
let model = provider.model("gpt-4o");  // routes via Portkey
```

- Capabilities: TextInput, TextOutput, Streaming, ToolCalling

## Middleware — `rs_ai_core::middleware`

Chain-able middleware over any `LanguageModel`:

- `RetryMiddleware` — exponential backoff on transient errors
- `LoggingMiddleware` — structured request/response logging
- `CacheMiddleware` — in-memory response cache with TTL + key hash

## Testing: `rs_ai_testing`

```rust
use rs_ai_testing::{MockLanguageModel, MockResponse};

let mock = MockLanguageModel::new("test-model")
    .with_text("response 1")
    .with_error(AiError::StreamError { message: "timeout".into() })
    .with_object(serde_json::json!({ "answer": 42 }));
```

FIFO response queue. Records all invocations with timestamps in `mock.calls()`.

## Running Tests

```bash
cargo test                         # all workspace tests
cargo test -p rs_ai_core           # core traits + middleware + cache + UI stream + observability tests
cargo test -p rs_ai_providers      # provider tests (enable features as needed)
cargo test -p rs_ai_local          # local runtime tests
cargo test -p rs_ai_testing        # mock model tests
```

## Adding a New Provider

1. Create a new module in `crates/rs_ai_providers/src/<provider>/` with `mod.rs`, `model.rs`, `provider.rs`, `client.rs`, `error.rs`, etc.
2. Implement `LanguageModel` (and optionally `Provider`) from `rs_ai_core`
3. Add a feature flag in `crates/rs_ai_providers/Cargo.toml`
4. Add `pub mod <provider>;` and `pub use <provider>::*;` behind `#[cfg(feature = "...")]` in `crates/rs_ai_providers/src/lib.rs`
5. Add an entry point function in `crates/rs_ai/src/lib.rs`
6. Add integration tests in `crates/rs_ai_providers/tests/`

## Key Dependencies

- `tokio` — async runtime
- `reqwest` — HTTP client with streaming
- `serde` / `serde_json` — serialisation
- `futures` — `Stream` trait and combinators
- `async-trait` — async in trait definitions
- `thiserror` — error types
- `secrecy` — API key zeroisation
- `pin-project-lite` — safe `Stream` pinning
- `uuid` — trace IDs
- `tracing` — structured logging