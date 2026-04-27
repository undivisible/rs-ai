# rs_ai — Rust AI SDK

A comprehensive Rust SDK for building AI applications with 15+ cloud and local providers, real-time streaming, and a clean async-first API.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
rs_ai = "0.1.0"
```

```rust
use rs_ai::rs_ai_claude;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads ANTHROPIC_API_KEY from environment automatically
    let answer = rs_ai_claude()
        .model("claude-sonnet-4-6")
        .generate("What is 2+2?")
        .await?;

    println!("{}", answer);
    Ok(())
}
```

## Providers

| Function | Provider | Env Var |
|---|---|---|
| `rs_ai_claude()` | Anthropic Claude | `ANTHROPIC_API_KEY` |
| `rs_ai_chatgpt()` | OpenAI ChatGPT | `OPENAI_API_KEY` |
| `rs_ai_gemini()` | Google Gemini | `GOOGLE_API_KEY` |
| `rs_ai_xai()` | xAI Grok | `XAI_API_KEY` |
| `rs_ai_cloudflare(account_id)` | Cloudflare Workers AI | `CLOUDFLARE_API_TOKEN` |
| `rs_ai_compatible(base_url)` | Any OpenAI-compatible API | `OPENAI_API_KEY` |

API keys are read from environment variables automatically. Pass `.api_key("...")` to override.

## Examples

### Text Generation

```rust
use rs_ai::rs_ai_gemini;

let response = rs_ai_gemini()
    .model("gemini-2.5-flash")
    .generate("Explain Rust lifetimes")
    .await?;
```

### Streaming

```rust
use rs_ai::rs_ai_claude;
use futures::StreamExt;

let mut stream = rs_ai_claude()
    .model("claude-sonnet-4-6")
    .stream("Write a poem about Rust")
    .await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### Vision (Image Input)

```rust
use rs_ai::rs_ai_claude;

let response = rs_ai_claude()
    .model("claude-sonnet-4-6")
    .with_image("https://example.com/photo.jpg")  // or local file path
    .generate("Describe this image")
    .await?;
```

### Cloudflare AI Gateway

```rust
use rs_ai::rs_ai_claude;

let response = rs_ai_claude()
    .model("claude-sonnet-4-6")
    .cf_ai_gateway("account-id/gateway-id")
    .generate("Hello!")
    .await?;
```

### Prompt Caching

```rust
use rs_ai::rs_ai_claude;
use rs_ai_core::{CacheConfig, CacheTTL};

let response = rs_ai_claude()
    .model("claude-sonnet-4-6")
    .with_cache(CacheConfig::new().enable_anthropic_ephemeral())
    .generate("Summarise this document...")
    .await?;
```

### Explicit API Key

```rust
use rs_ai::rs_ai_chatgpt;

let response = rs_ai_chatgpt()
    .api_key("sk-...")   // overrides OPENAI_API_KEY
    .model("gpt-4o")
    .generate("Hello!")
    .await?;
```

## Lower-Level Provider Access

For direct provider access with full control, use `rs_ai_providers`:

```rust
use rs_ai_providers::claude::ClaudeProvider;
use rs_ai_core::LanguageModel;

let provider = ClaudeProvider::new(std::env::var("ANTHROPIC_API_KEY")?);
let model = provider.claude_sonnet();
let result = model.generate(prompt, options).await?;
```

Available provider modules (each gated by a Cargo feature):

- `rs_ai_providers::claude` — Anthropic Claude (streaming, vision, tool use, cache control)
- `rs_ai_providers::chatgpt` — OpenAI ChatGPT (Realtime API, vision, structured output)
- `rs_ai_providers::gemini` — Google Gemini (Live API for voice/video, 2.5 Pro/Flash)
- `rs_ai_providers::xai` — xAI Grok (conversation routing, prompt caching)
- `rs_ai_providers::openai_compatible` — Generic OpenAI-compatible + 9 presets
- `rs_ai_providers::ollama` — Local Ollama models
- `rs_ai_providers::cloudflare` — Cloudflare Workers AI
- `rs_ai_providers::portkey` — Portkey AI Gateway
- `rs_ai_providers::langfuse` — Langfuse observability wrapper

## Local Runtimes

Platform-specific on-device AI runtimes are available via `rs_ai_local` (feature-gated):

- `browser` — WASM browser AI (Chrome/Edge built-in AI)
- `gemini-nano` — Android Gemini Nano (Prompt API)
- `foundationmodels` — Apple Foundation Models (Swift FFI)
- `phi-silica` — Windows Phi Silica (C# bridge)

## Workspace Structure

```
crates/
├── rs_ai/               # Top-level fluent API (rs_ai_claude, rs_ai_gemini, ...)
├── rs_ai_core/          # Core traits, types, middleware, cache, UI stream, observability
├── rs_ai_providers/     # Cloud AI providers (feature-gated)
├── rs_ai_local/         # Platform-specific local runtimes (feature-gated)
└── rs_ai_testing/       # MockLanguageModel for unit tests
```

## Testing

```bash
cargo test            # all workspace tests
cargo test -p rs_ai_core
cargo test -p rs_ai_providers --all-features
cargo test -p rs_ai_local --all-features
cargo test -p rs_ai_testing
```

Mock provider for unit tests (no network required):

```rust
use rs_ai_testing::MockLanguageModel;

let mock = MockLanguageModel::new("test-model")
    .with_text("Hello from mock!");

let result = mock.generate(prompt, options).await?;
assert_eq!(result.text.as_deref(), Some("Hello from mock!"));
```

## Environment Variables

```bash
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
GOOGLE_API_KEY=AIza...
XAI_API_KEY=xai-...
CLOUDFLARE_API_TOKEN=...
```

## License

MPL-2.0
