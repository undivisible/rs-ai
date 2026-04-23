# RAI — Rust AI SDK

A comprehensive Rust SDK for building AI applications with 15+ cloud and local providers, real-time voice/video streaming, and a clean async-first API.

## Quick Start

### Simplified API (`rai_simple`)

The `rai_simple` crate provides zero-boilerplate access to any provider:

```rust
use rai_simple::{rai_claude, rai_chatgpt, rai_gemini};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Each function reads the API key from the environment variable
    let claude = rai_claude("claude-sonnet-4-6")?;  // ANTHROPIC_API_KEY
    let gpt    = rai_chatgpt("gpt-4o")?;             // OPENAI_API_KEY
    let gemini = rai_gemini("gemini-2.5-flash")?;    // GOOGLE_API_KEY
    Ok(())
}
```

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ai = rai_simple::rai_claude("claude-sonnet-4-6")?;
    let answer = ai.generate("What is 2 + 2?").await?;
    println!("{}", answer);
    Ok(())
}
```

### Provider API (`rai_claude`, `rai_chatgpt`, `rai_gemini`)

For direct provider access with full control:

```rust
use rai_claude::ClaudeProvider;
use rai_ai::{LanguageModel, Prompt, GenerateOptions};

let provider = ClaudeProvider::new(std::env::var("ANTHROPIC_API_KEY")?);
let model    = provider.claude_sonnet();

let result = model.generate(
    Prompt::Text("What is the capital of France?".into()),
    GenerateOptions::default(),
).await?;

println!("{}", result.text.unwrap());
```

### Streaming

```rust
use futures::StreamExt;
use rai_claude::ClaudeProvider;
use rai_ai::{GenerateOptions, LanguageModel, Prompt, StreamEvent};

let provider = ClaudeProvider::new(std::env::var("ANTHROPIC_API_KEY")?);
let model    = provider.claude_sonnet();

let mut stream = model
    .stream(Prompt::Text("Write a poem about Rust.".into()), GenerateOptions::default())
    .await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::TextDelta { delta }       => print!("{}", delta),
        StreamEvent::MessageEnd { usage, .. }  => println!("\nDone: {:?}", usage),
        _ => {}
    }
}
```

### Streaming Tool Use

```rust
use rai_ai::{GenerateOptions, LanguageModel, Prompt, StreamEvent, ToolChoice, ToolDefinition};
use schemars::schema_for;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct WeatherInput { location: String }

let tools = vec![ToolDefinition {
    name:        "get_weather".into(),
    description: "Get the current weather for a location.".into(),
    parameters:  serde_json::to_value(schema_for!(WeatherInput))?,
}];

let options = GenerateOptions::default()
    .with_tools(tools)
    .with_tool_choice(ToolChoice::Auto);

let mut stream = model
    .stream(Prompt::Text("What's the weather in Tokyo?".into()), options)
    .await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ToolCallStart { call_id, tool_name } =>
            println!("🔧 {} [{}]", tool_name, call_id),
        StreamEvent::ToolCallEnd { call_id: _, arguments } =>
            println!("   args: {}", arguments),
        StreamEvent::MessageEnd { .. } => break,
        _ => {}
    }
}
```

### Gemini Live API (Real-Time Voice/Video)

```rust
use rai_gemini::{
    live_api::{LiveEvent, LiveSession, BidiSetup, LiveGenerationConfig},
    GEMINI_FLASH_LIVE_LATEST,
};

let setup = BidiSetup {
    model: format!("models/{GEMINI_FLASH_LIVE_LATEST}"),
    system_instruction: None,
    generation_config: Some(LiveGenerationConfig {
        response_modalities: Some(vec!["AUDIO".into(), "TEXT".into()]),
        ..Default::default()
    }),
    tools: None,
};

let mut session = LiveSession::connect(
    &std::env::var("GOOGLE_API_KEY")?,
    GEMINI_FLASH_LIVE_LATEST,
    setup,
).await?;

// Wait for setup confirmation, then send text or audio
session.send_text("Hello, how are you?").await?;

while let Some(ev) = session.recv().await {
    match ev? {
        LiveEvent::TextDelta(t)   => print!("{t}"),
        LiveEvent::AudioDelta(pcm) => { /* write to speaker */ }
        LiveEvent::TurnComplete    => break,
        _ => {}
    }
}
```

### OpenAI Realtime API (Voice Agents)

```rust
use rai_chatgpt::{ChatGptProvider, GPT_4O_REALTIME};
use rai_chatgpt::realtime_api::{ServerEvent, SessionConfig, TurnDetection, Voice};

let provider = ChatGptProvider::new(std::env::var("OPENAI_API_KEY")?);
let mut session = provider.realtime_session(GPT_4O_REALTIME).await?;

session.configure(SessionConfig {
    voice:           Some(Voice::Alloy),
    turn_detection:  Some(TurnDetection::server_vad()),
    instructions:    Some("You are a helpful assistant.".into()),
    ..Default::default()
}).await?;

session.send_text("Tell me a fun fact about Rust.").await?;

while let Some(ev) = session.recv().await {
    match ev? {
        ServerEvent::TextDelta { delta, .. }    => print!("{delta}"),
        ServerEvent::ResponseDone { .. }         => break,
        ServerEvent::Error { error }             => {
            eprintln!("Error: {}", error.message);
            break;
        }
        _ => {}
    }
}
```

### OpenAI-Compatible Providers

```rust
use rai_openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleProvider};
use rai_openai_compatible::presets;

// OpenRouter — 300+ models
let config   = presets::openrouter::config(std::env::var("OPENROUTER_KEY")?);
let provider = OpenAiCompatibleProvider::new(config, "openrouter", "OpenRouter");
let model    = provider.language_model(presets::openrouter::CLAUDE_SONNET);

// Amazon Bedrock
let config   = presets::bedrock::config(std::env::var("AWS_ACCESS_KEY_ID")?)
    .with_header("X-AWS-Region", "us-west-2");
let provider = OpenAiCompatibleProvider::new(config, "bedrock", "Amazon Bedrock");

// Local Ollama
let config   = presets::ollama::config(None); // defaults to localhost:11434
let provider = OpenAiCompatibleProvider::new(config, "ollama", "Ollama");
let model    = provider.language_model("llama3.2");
```

## Supported Providers

| Crate | Provider | Notes |
|-------|----------|-------|
| `rai_claude` | Anthropic Claude | Native API, tool streaming, vision |
| `rai_chatgpt` | OpenAI ChatGPT | OpenAI-compat + Realtime API (voice) |
| `rai_gemini` | Google Gemini | Native API + Live API (voice/video) |
| `rai_openai_compatible` | Generic OpenAI-compat | 9 presets (see below) |
| `rai_ollama` | Ollama | Direct Ollama API with model management |
| `rai_gemini_nano` | Gemini Nano | On-device (Android, Chrome) |
| `rai_phi_silica` | Phi Silica | On-device (Windows Arm) |
| `rai_foundationmodels` | Foundation Models | Cross-provider |
| `rai_browser` | Browser Prompt API | WASM, Chrome 138+ |

### OpenAI-Compatible Presets

`rai_openai_compatible::presets` includes:
- `openrouter` — 300+ models via OpenRouter
- `kilo` — Kilo AI Gateway (cost management)
- `bedrock` — Amazon Bedrock
- `together` — Together AI (open-source models)
- `octoml` — OctoML serverless
- `azure` — Azure OpenAI
- `cloudflare` — Cloudflare Workers AI
- `vllm` — Self-hosted vLLM
- `ollama` — Self-hosted Ollama (OpenAI-compat mode)

## Workspace Layout

```
rai/
├── crates/
│   ├── rai_ai/                  # Core traits, types, streaming
│   ├── rai_simple/              # Simplified factory functions
│   ├── rai_middleware/          # Request/response middleware
│   ├── rai_ui_stream/           # UI streaming utilities
│   ├── rai_testing/             # Test utilities and mocks
│   ├── rai_claude/              # Anthropic Claude
│   ├── rai_chatgpt/             # OpenAI ChatGPT + Realtime API
│   ├── rai_gemini/              # Google Gemini + Live API
│   ├── rai_openai_compatible/   # Generic OpenAI-compat (9 presets)
│   ├── rai_ollama/              # Ollama direct API
│   ├── rai_gemini_nano/         # On-device Gemini Nano
│   ├── rai_phi_silica/          # Microsoft Phi Silica
│   ├── rai_foundationmodels/    # Foundation models
│   └── rai_browser/             # Browser WASM runtime
├── examples/
│   ├── basic_text/              # Multi-provider text generation
│   ├── stream_text/             # Streaming text responses
│   ├── generate_object/         # Structured output
│   ├── stream_object/           # Streaming structured output
│   ├── tool_loop/               # Basic tool use
│   ├── streaming_tool_use/      # Real-time streaming tool calls
│   ├── agent_with_tools/        # Agent loop with tool calling
│   ├── multimodal/              # Images and media
│   ├── realtime_voice/          # OpenAI Realtime API voice agent
│   ├── gemini_live/             # Gemini Live API voice/text session
│   ├── router/                  # Multi-provider routing
│   └── local_*/                 # Android, Apple, Windows local models
└── Cargo.toml
```

## Environment Variables

```bash
# Cloud providers
ANTHROPIC_API_KEY=sk-ant-...   # rai_claude
OPENAI_API_KEY=sk-...          # rai_chatgpt
GOOGLE_API_KEY=...             # rai_gemini

# OpenAI-compatible routers
OPENROUTER_KEY=sk-or-...
KILO_API_KEY=...
TOGETHER_API_KEY=...

# AWS (Bedrock)
AWS_REGION=us-west-2
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
```

## Development

```bash
# Build
cargo build

# Check (fast, no codegen)
cargo check --workspace

# Test
cargo test --workspace

# Run an example
ANTHROPIC_API_KEY=... cargo run --example agent_with_tools
GOOGLE_API_KEY=...    cargo run --example gemini_live
OPENAI_API_KEY=...    cargo run --example realtime_voice
```

## Key Dependencies

- `tokio` — async runtime
- `futures` — Stream utilities
- `reqwest` — HTTP client
- `tokio-tungstenite` — WebSocket (Realtime / Live APIs)
- `serde` / `serde_json` — serialization
- `schemars` — JSON schema generation for tool definitions
- `secrecy` — secure API key handling
- `thiserror` — ergonomic error types

## References

- [Anthropic Claude API](https://docs.anthropic.com/en/docs/build-with-claude/tool-use)
- [Gemini Live API](https://ai.google.dev/gemini-api/docs/live-api)
- [OpenAI Realtime API](https://platform.openai.com/docs/api-reference/realtime)
- [OpenRouter](https://openrouter.ai/docs/api/api-reference)
- [Amazon Bedrock](https://docs.aws.amazon.com/bedrock/)
