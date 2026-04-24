# rs_ai — Rust AI SDK

A comprehensive Rust SDK for building AI applications with 15+ cloud and local providers, real-time streaming, and a clean async-first API.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
rs_ai = { path = "crates/rs_ai" }
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
use rs_ai_cache::{CacheConfig, CacheTTL};

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

## Provider Crates

For direct provider access with full control:

```rust
use rai_claude::ClaudeProvider;
use rai_ai::LanguageModel;

let provider = ClaudeProvider::new(std::env::var("ANTHROPIC_API_KEY")?);
let model = provider.claude_sonnet();
let result = model.generate(prompt, options).await?;
```

Available provider crates:

- `rai_claude` — Anthropic Claude (streaming, vision, tool use, cache control)
- `rai_chatgpt` — OpenAI ChatGPT (Realtime API, vision, structured output)
- `rai_gemini` — Google Gemini (Live API for voice/video, 2.5 Pro/Flash)
- `rai_xai` — xAI Grok (conversation routing, prompt caching)
- `rai_openai_compatible` — Generic OpenAI-compatible + 9 presets
- `rai_ollama` — Local Ollama models
- `rai_cloudflare` — Cloudflare Workers AI

## Observability & Caching

- `rs_ai_langfuse` — LangFuse tracing wrapper (wraps any `LanguageModel`)
- `rs_ai_portkey` — Portkey AI Gateway (multi-provider routing, analytics)
- `rs_ai_cache` — Unified cache config (`CacheConfig`, `CacheTTL`)

```rust
use rs_ai_langfuse::with_langfuse;

let model = Box::new(provider.claude_sonnet());
let observable = with_langfuse(model, "public_key", "secret_key").await?;
let result = observable.generate(prompt, options).await?;
```

## Workspace Structure

```
crates/
├── rs_ai/                   # Top-level fluent API (rs_ai_claude, rs_ai_gemini, …)
├── rs_ai_cache/             # Unified cache configuration
├── rs_ai_langfuse/          # LangFuse observability integration
├── rs_ai_portkey/           # Portkey AI Gateway provider
│
├── rai_ai/                  # Core traits: LanguageModel, Provider, StreamEvent
├── rai_claude/              # Anthropic Claude provider
├── rai_chatgpt/             # OpenAI ChatGPT provider
├── rai_gemini/              # Google Gemini provider
├── rai_xai/                 # xAI Grok provider
├── rai_openai_compatible/   # Generic OpenAI-compatible provider + presets
├── rai_ollama/              # Local Ollama provider
├── rai_cloudflare/          # Cloudflare Workers AI provider
├── rai_middleware/          # Retry, logging, caching middleware
├── rai_testing/             # MockLanguageModel for unit tests
└── rai_ui_stream/           # SSE/NDJSON streaming helpers
```

## Testing

```bash
cargo test            # 125 tests, all passing
cargo test -p rs_ai_cache
cargo test -p rai_xai
```

Mock provider for unit tests (no network required):

```rust
use rai_testing::MockLanguageModel;

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
