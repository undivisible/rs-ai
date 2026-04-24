# rs_ai — Codebase Guide

## Overview

`rs_ai` is a Rust workspace SDK for AI applications. The top-level `rs_ai` crate exposes a fluent builder API (`rs_ai_claude()`, `rs_ai_gemini()`, etc.) over a set of provider crates (`rai_claude`, `rai_gemini`, …) built on the `rai_ai` core traits.

## Workspace Layout

```
rusty_ai/
├── Cargo.toml
├── crates/
│   ├── rs_ai/               # Fluent top-level API
│   ├── rs_ai_cache/         # CacheConfig / CacheTTL
│   ├── rs_ai_langfuse/      # LangFuse observability wrapper
│   ├── rs_ai_portkey/       # Portkey gateway provider
│   │
│   ├── rai_ai/              # Core traits & types
│   ├── rai_claude/          # Anthropic Claude
│   ├── rai_chatgpt/         # OpenAI ChatGPT
│   ├── rai_gemini/          # Google Gemini
│   ├── rai_xai/             # xAI Grok
│   ├── rai_openai_compatible/ # Generic OpenAI-compat + presets
│   ├── rai_ollama/          # Local Ollama
│   ├── rai_cloudflare/      # Cloudflare Workers AI
│   ├── rai_middleware/      # Retry / logging / cache middleware
│   ├── rai_testing/         # MockLanguageModel
│   ├── rai_ui_stream/       # SSE / NDJSON helpers
│   ├── rai_observability/   # OpenTelemetry tracing
│   ├── rai_simple/          # Legacy simple wrappers
│   ├── rai_browser/         # WASM browser runtime
│   ├── rai_gemini_nano/     # On-device Gemini Nano
│   ├── rai_phi_silica/      # Microsoft Phi Silica
│   └── rai_foundationmodels/ # Cross-provider foundation models
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

## Core Crate: `rai_ai`

Defines the traits every provider implements:

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

## Provider Crates

### `rai_claude` — Anthropic

- `ClaudeProvider::new(api_key)` / `.with_base_url(url)`
- Convenience: `.claude_sonnet()`, `.claude_opus()`, `.claude_haiku()`
- `ClaudeModel::set_cache(config)` — applies `cache_control` to requests
- Streaming via SSE; tool use; vision; extended thinking

### `rai_chatgpt` — OpenAI

- `ChatGptProvider::new(api_key)` / `.with_org(org_id)`
- Convenience: `.gpt4o()`, `.gpt4o_mini()`, `.gpt54()`, `.gpt54_mini()`, `.gpt54_nano()`
- Realtime API (`RealtimeSession`) for voice agents
- Structured output, vision, audio I/O

### `rai_gemini` — Google

- `GeminiProvider::new(api_key)`
- Convenience: `.gemini_flash()`, `.gemini_pro()`, `.gemini_flash_lite()`, `.gemini_3_flash()`, `.gemini_31_pro()`
- `.model_with_base_url(id, url)` for proxy/CF gateway
- Live API (`provider.live_session(model_id)`) for low-latency voice/video

### `rai_xai` — xAI

- `XaiProvider::new(api_key)`
- Convenience: `.grok_4()`, `.grok_4_20_reasoning()`
- `XaiModel::set_cache(config)` — sets `x-grok-conv-id` headers and prompt cache keys

### `rai_openai_compatible`

- `OpenAiCompatibleProvider::new(config, id, name)`
- `OpenAiCompatibleConfig` presets: `openai()`, `openrouter()`, `kilo()`, `bedrock()`, `together()`, `vllm(url)`, `ollama_openai(url)`, `lmstudio(url)`, `textgen(url)`

### `rai_ollama` — Local

- `OllamaProvider::new()` / `.with_base_url(url)` (default `http://localhost:11434`)
- `provider.list_models()` — async model discovery

### `rai_cloudflare`

- `CloudflareProvider::new(account_id, api_key)`
- Routes to `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/run/{model}`

## New Crates (`rs_ai_*`)

### `rs_ai_cache`

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

### `rs_ai_langfuse`

Wraps any `LanguageModel` to emit traces to LangFuse without blocking:

```rust
let observable = with_langfuse(model, "pub_key", "secret_key").await?;
```

- `LangfuseModel::with_trace_id()`, `.with_user_id()`, `.with_session_id()`
- Non-blocking via `tokio::spawn` — failed traces log a warning only

### `rs_ai_portkey`

Portkey AI Gateway provider:

```rust
let provider = PortkeyProvider::new("pk_...");
let model = provider.model("gpt-4o");  // routes via Portkey
```

- Capabilities: TextInput, TextOutput, Streaming, ToolCalling

## Middleware: `rai_middleware`

Chain-able middleware over any `LanguageModel`:

- `RetryMiddleware` — exponential backoff on transient errors
- `LoggingMiddleware` — structured request/response logging
- `CacheMiddleware` — in-memory response cache with TTL + key hash

## Testing: `rai_testing`

```rust
use rai_testing::{MockLanguageModel, MockResponse};

let mock = MockLanguageModel::new("test-model")
    .with_text("response 1")
    .with_error(AiError::StreamError { message: "timeout".into() })
    .with_object(serde_json::json!({ "answer": 42 }));
```

FIFO response queue. Records all invocations with timestamps in `mock.calls()`.

## Running Tests

```bash
cargo test                         # all 125 tests
cargo test -p rs_ai_cache          # 19 tests (3 inline + 16 integration)
cargo test -p rai_xai              # 12 cache integration tests
cargo test -p rs_ai_langfuse       # 10 inline tests
cargo test -p rs_ai_portkey        # 5 integration tests
cargo test -p rai_claude           # 8 integration tests
cargo test -p rai_gemini           # 8 integration tests
cargo test -p rai_chatgpt          # 8 integration tests
cargo test -p rai_ollama           # 7 integration tests
```

## Adding a New Provider

1. Create `crates/rai_newprovider/` with `Cargo.toml`, `src/{lib,provider,model,client,error}.rs`
2. Implement `LanguageModel` (and optionally `Provider`) from `rai_ai`
3. Add to workspace `Cargo.toml` `members`
4. Add an entry point function in `crates/rs_ai/src/lib.rs`
5. Add integration tests in `crates/rai_newprovider/tests/`

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
