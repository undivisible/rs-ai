# RAI - Rust AI SDK

A comprehensive Rust SDK for building AI applications with support for multiple cloud and local AI providers, with a simplified async-first API.

## Overview

RAI (Rust AI) is a unified framework that provides a consistent, easy-to-use interface for working with various AI providers including:

- **Cloud Providers**: Claude (Anthropic), ChatGPT (OpenAI), Gemini (Google)
- **Cloud-Compatible Routers**: OpenRouter, Amazon Bedrock, Kilo, Together AI, OctoML
- **Local Providers**: Ollama, vLLM, LM Studio, text-generation-webui
- **Platform Runtimes**: Gemini Nano, Phi Silica, Foundation Models, Browser Runtime
- **Real-time Features**: Gemini Live API, OpenAI Realtime API, tool streaming

## Quick Start

### Simple API

```rust
use rai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Claude
    let claude = rai_claude("claude-sonnet-4-6").await?;
    let result = claude.generate("What is 2+2?").await?;
    println!("Claude: {}", result);

    // ChatGPT
    let gpt = rai_chatgpt("gpt-4o").await?;
    let result = gpt.generate("Hello, world!").await?;
    println!("ChatGPT: {}", result);

    // Gemini
    let gemini = rai_gemini("gemini-2.0-flash").await?;
    let result = gemini.generate("Tell me a joke").await?;
    println!("Gemini: {}", result);

    Ok(())
}
```

### OpenAI-Compatible Providers

```rust
use rai::*;
use rai_openai_compatible::presets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenRouter
    let config = presets::openrouter::config(
        std::env::var("OPENROUTER_KEY")?
    );
    let provider = rai_openai_compatible::OpenAiCompatibleProvider::new(config);
    let model = provider.model(presets::openrouter::CLAUDE_SONNET);
    
    let result = generate_text(&model, "What is Rust?").await?;
    println!("Response: {}", result);

    // Kilo Gateway
    let config = presets::kilo::config(
        std::env::var("KILO_API_KEY")?
    );
    let provider = rai_openai_compatible::OpenAiCompatibleProvider::new(config);
    let model = provider.model(presets::kilo::GPT4O);
    
    let result = generate_text(&model, "Explain quantum computing").await?;
    println!("Response: {}", result);

    // Local Ollama
    let config = presets::ollama::config(None); // Uses localhost:11434
    let provider = rai_openai_compatible::OpenAiCompatibleProvider::new(config);
    let model = provider.model("llama2");
    
    let result = generate_text(&model, "Hello").await?;
    println!("Response: {}", result);

    Ok(())
}
```

### Streaming Responses

```rust
use futures::StreamExt;
use rai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = rai_claude("claude-sonnet-4-6").await?;
    let mut stream = model.model().stream(
        Prompt::Text("Write a poem about Rust".into()),
        GenerateOptions::default()
    ).await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => print!("{}", delta),
            StreamEvent::MessageEnd { usage, .. } => {
                println!("\nTokens: {:?}", usage);
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Tool Use (Function Calling)

```rust
use rai::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Calculator {
    operation: String,
    a: f64,
    b: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = rai_claude("claude-sonnet-4-6").await?;
    
    let tools = vec![
        ToolDefinition {
            name: "calculator".to_string(),
            description: "Basic math operations".to_string(),
            input_schema: schemars::schema_for!(Calculator),
        }
    ];

    // Model will intelligently call tools and continue
    let result = generate_text(&model.model(), "What is 15 + 27?").await?;
    println!("Result: {}", result);

    Ok(())
}
```

### Structured Output (JSON Generation)

```rust
use rai::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct BookReview {
    title: String,
    rating: u8,
    summary: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = rai_claude("claude-sonnet-4-6").await?;

    let review: BookReview = generate_object(
        model.model(),
        "Review 'The Rust Book'"
    ).await?;
    
    println!("Rating: {}/10", review.rating);
    println!("Summary: {}", review.summary);

    Ok(())
}
```

## Available Provider Presets

### Cloud-Compatible Routers
- **OpenRouter** - 300+ models from all major providers
- **Kilo Gateway** - Unified routing and cost management
- **Together AI** - Optimized open-source models
- **OctoML** - Serverless model hosting
- **Azure OpenAI** - Microsoft-hosted OpenAI models
- **Amazon Bedrock** - AWS-managed AI models

### Local / Self-Hosted
- **Ollama** - Simple local model runner
- **vLLM** - High-throughput LLM inference
- **LM Studio** - GUI for local models
- **text-generation-webui** - Full-featured local inference
- **Cloudflare Workers AI** - Edge network inference

## Features

### Unified API
- Consistent interface across all 15+ providers
- Single implementation for text, streaming, structured output, and tools

### Real-Time Capabilities
- **Gemini Live API** - Low-latency voice/vision conversations
- **OpenAI Realtime API** - Voice agents with function calling
- **Streaming Tool Use** - Real-time function calling across all providers

### Type Safety
- Compile-time safety through Rust's type system
- Automatic JSON schema generation for structured outputs

### Async/Streaming First
- Non-blocking async-await throughout
- Streaming responses for large outputs
- Efficient memory usage

### Easy Local Development
- Docker compose with popular models
- Ollama integration for M1/M2/M3 Macs and Windows
- Zero config with sensible defaults

## Project Structure

```
rai/
├── crates/
│   ├── rai_ai/                    # Core traits and simplified API
│   ├── rai_middleware/            # Request/response middleware
│   ├── rai_ui_stream/             # UI streaming utilities
│   ├── rai_testing/               # Test utilities and mocks
│   ├── rai_claude/                # Anthropic Claude integration
│   ├── rai_chatgpt/               # OpenAI ChatGPT integration
│   ├── rai_gemini/                # Google Gemini integration
│   ├── rai_openai_compatible/     # Generic OpenAI-compatible endpoints
│   ├── rai_ollama/                # Ollama local models
│   ├── rai_gemini_nano/           # On-device Gemini
│   ├── rai_phi_silica/            # Microsoft Phi Silica
│   ├── rai_foundationmodels/      # Foundation models
│   └── rai_browser/               # Browser runtime
├── examples/
│   ├── basic_text/                # Simple text generation
│   ├── stream_text/               # Streaming responses
│   ├── generate_object/           # Structured JSON output
│   ├── stream_object/             # Streaming structured data
│   ├── tool_loop/                 # Tool use and agents
│   ├── multimodal/                # Images and media
│   ├── local_android/             # Android deployment
│   ├── local_apple/               # Apple Silicon support
│   ├── local_windows/             # Windows local models
│   └── router/                    # Multi-provider routing
└── Cargo.toml
```

## Configuration

Each provider requires environment variables:

```bash
# Cloud Providers
ANTHROPIC_API_KEY=...         # Claude
OPENAI_API_KEY=...            # ChatGPT
GOOGLE_API_KEY=...            # Gemini

# Compatible Routers
OPENROUTER_KEY=...            # OpenRouter
KILO_API_KEY=...              # Kilo
TOGETHER_API_KEY=...          # Together

# AWS
AWS_REGION=...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
ANTHROPIC_API_KEY=sk-... cargo run --example basic_text
OPENROUTER_KEY=sk-... cargo run --example router
```

## Performance

- **Async Throughout** - Non-blocking I/O via tokio
- **Streaming Support** - Handle large outputs without loading in memory
- **Connection Pooling** - HTTP connection reuse via reqwest
- **Lazy Initialization** - Models and providers created on-demand
- **Zero-Copy** - Efficient reference handling

## Security

- **API Key Handling** - Secure key management via `secrecy` crate
- **HTTPS Only** - All provider connections encrypted
- **No Secret Logging** - Keys excluded from traces
- **Safe Deserialization** - Validated JSON parsing
- **Input Validation** - Provider-specific safety checks

## Recent Additions (2026)

- ✨ Simplified single-function API (`rai_claude`, `rai_chatgpt`, etc.)
- ✨ Comprehensive OpenAI-compatible presets (OpenRouter, Bedrock, Kilo, Ollama, etc.)
- ✨ Gemini 2.5 Flash with Live API support
- ✨ OpenAI Realtime API for voice agents
- ✨ Streaming tool use across all providers
- ✨ Automated test suite
- ✨ Renamed from `rusty_ai` to `rai` for clarity

## Sources

For more information on the features used:

- [Claude API Tool Use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use)
- [Claude API Streaming](https://platform.claude.com/docs/en/build-with-claude/streaming)
- [Gemini Live API](https://ai.google.dev/gemini-api/docs/live-api)
- [OpenAI Realtime API](https://developers.openai.com/api/docs/guides/realtime)
- [OpenRouter Documentation](https://openrouter.ai/docs/api/api-reference)
- [Amazon Bedrock Claude](https://platform.claude.com/docs/en/build-with-claude/claude-in-amazon-bedrock)
- [Kilo AI Gateway](https://kilo.ai/docs/gateway/api-reference)

## License

MPL-2.0 (Mozilla Public License 2.0)

## Contributing

Contributions welcome! Please submit pull requests and open issues for bugs and feature requests.

## Support

For issues and questions:
https://github.com/undivisible/rusty_ai
