# Rusty AI SDK

A comprehensive Rust SDK for building AI applications with support for multiple cloud and local AI providers.

## Overview

Rusty AI is a unified, async-first Rust framework that provides a consistent interface for working with various AI providers including:

- **Cloud Providers**: Claude (Anthropic), ChatGPT (OpenAI), Gemini (Google)
- **Local Providers**: Ollama, Phi Silica, Gemini Nano, Foundation Models
- **Browser Runtime**: In-browser AI execution
- **OpenAI Compatible**: Support for OpenAI-compatible endpoints

## Project Structure

### Core Crates

- **`rusty_ai`** - Core traits, types, and abstractions providing the foundation for the SDK
- **`rusty_middleware`** - Middleware layer for request/response processing
- **`rusty_ui_stream`** - UI streaming utilities for real-time updates
- **`rusty_testing`** - Testing utilities and fixtures

### Provider Crates

- **`rusty_claude`** - Anthropic Claude provider
- **`rusty_chatgpt`** - OpenAI ChatGPT provider
- **`rusty_gemini`** - Google Gemini provider
- **`rusty_openai_compatible`** - Generic OpenAI-compatible endpoints

### Local/Platform Runtime Crates

- **`rusty_ollama`** - Ollama local model support
- **`rusty_gemini_nano`** - Google Gemini Nano for on-device AI
- **`rusty_phi_silica`** - Microsoft Phi Silica models
- **`rusty_foundationmodels`** - Foundation models support
- **`rusty_browser`** - Browser-based runtime for web applications

## Key Features

- **Unified API**: Consistent interface across all AI providers
- **Async/Streaming**: Built on async-await with streaming support for real-time responses
- **Type Safe**: Leverages Rust's type system for safety and ergonomics
- **Modular Architecture**: Mix and match providers based on your needs
- **Router Support**: Route requests between multiple providers
- **Multi-modal**: Support for text, images, and other modalities
- **Tool Integration**: Built-in support for tool use and structured outputs

## Quick Start

### Prerequisites

- Rust 1.85 or later
- Appropriate API keys for your chosen providers

### Basic Text Generation

```rust
use rusty_ai::*;
use rusty_claude::ClaudeProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ClaudeProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );
    let model = provider.claude_sonnet();

    let result = generate_text(
        &model, 
        "What is Rust programming language?"
    ).await?;
    
    println!("Response: {}", result);
    Ok(())
}
```

### Streaming Responses

```rust
use futures::StreamExt;
use rusty_ai::*;
use rusty_claude::ClaudeProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ClaudeProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );
    let model = provider.claude_sonnet();

    let mut stream = stream_text(&model, "Write a poem about Rust").await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => print!("{}", delta),
            StreamEvent::MessageEnd { usage, .. } => {
                println!("\n\nTokens used: {:?}", usage);
            }
            _ => {}
        }
    }
    Ok(())
}
```

### Structured Output (Object Generation)

```rust
use serde::Deserialize;
use rusty_ai::*;
use rusty_claude::ClaudeProvider;

#[derive(Deserialize)]
struct BookReview {
    title: String,
    rating: u8,
    summary: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ClaudeProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );
    let model = provider.claude_sonnet();

    let review: BookReview = generate_object(
        &model,
        "Review 'The Rust Book'"
    ).await?;
    
    println!("Rating: {}/10", review.rating);
    Ok(())
}
```

### Multi-Provider Router

```rust
use rusty_ai::*;
use rusty_claude::ClaudeProvider;
use rusty_chatgpt::ChatGptProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let claude = ClaudeProvider::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );
    let chatgpt = ChatGptProvider::new(
        std::env::var("OPENAI_API_KEY")?
    );

    // Use different providers for different queries
    let claude_result = generate_text(
        &claude.claude_sonnet(), 
        "Query 1"
    ).await?;
    
    let gpt_result = generate_text(
        &chatgpt.gpt4o_mini(), 
        "Query 2"
    ).await?;
    
    Ok(())
}
```

## Examples

The `examples/` directory contains complete working examples:

- **`basic_text`** - Simple text generation with multiple providers
- **`stream_text`** - Real-time streaming of text responses
- **`generate_object`** - Generating structured data (JSON)
- **`stream_object`** - Streaming structured objects
- **`tool_loop`** - Using tools/function calling in agent loops
- **`multimodal`** - Working with images and other modalities
- **`local_android`**, **`local_apple`**, **`local_windows`** - Platform-specific local model execution
- **`router`** - Routing between multiple providers

## Configuration

Each provider typically requires environment variables:

- `ANTHROPIC_API_KEY` - For Claude
- `OPENAI_API_KEY` - For ChatGPT
- `GOOGLE_API_KEY` - For Gemini
- Provider-specific settings for local models

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
ANTHROPIC_API_KEY=your_key cargo run --example basic_text
```

## Architecture

The SDK is built on several architectural principles:

1. **Trait-based Abstraction**: Core traits like `Model` and `Provider` enable provider flexibility
2. **Async Throughout**: All operations are non-blocking using async/await
3. **Streaming First**: Built-in support for streaming responses to handle large outputs
4. **Type Safety**: Leverages Rust's type system for compile-time safety
5. **Composition**: Middleware and tools compose together for flexible request handling

## Dependencies

Key dependencies include:

- **tokio** - Async runtime with full feature set
- **reqwest** - HTTP client with streaming support
- **serde/serde_json** - Serialization framework
- **schemars** - JSON Schema generation for typed outputs
- **thiserror** - Error handling
- **tracing** - Observability and logging

## License

MPL-2.0 (Mozilla Public License 2.0)

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Support

For issues, questions, or contributions, visit:
https://github.com/undivisible/rusty_ai
