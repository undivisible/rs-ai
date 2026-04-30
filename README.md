# rs_ai — Rust AI SDK

A comprehensive Rust SDK for building AI applications with cloud and local providers, streaming, and a clean async-first API.

## Quick Start

```toml
[dependencies]
rs_ai = "0.2"
```

```rust
use rs_ai::rs_ai_claude;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
| `rs_ai_compatible(base_url)` | Any OpenAI-compatible | `OPENAI_API_KEY` |

## Local Runtimes

Add `rs_ai_local` to use on-device AI:

```toml
[dependencies]
rs_ai_local = { version = "0.2", features = ["gemini-nano"] }
```

### Android (Gemini Nano)

Enable the feature and add initialization to your Activity. They call **our** init function — just one line of Kotlin:

```kotlin
// MainActivity.kt
class MainActivity : Activity() {
    companion object {
        init { System.loadLibrary("rs_ai_local") }
        @JvmStatic external fun init(context: Context)
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        init(this)
    }
}
```

Then use from Rust — **your** code:

```rust
use rs_ai_local::gemini_nano::init_with_context;
use rs_ai_local::gemini_nano::GeminiNanoProvider;

// In your Rust code
let provider = GeminiNanoProvider::new(my_bridge);
let response = provider.model().generate("Hello!").await?;
```

### macOS (Foundation Models)

Enable the feature. Build on macOS with Xcode — `build.rs` auto-compiles the Swift bridge:

```toml
rs_ai_local = { version = "0.2", features = ["foundationmodels"] }
```

Use from Rust:

```rust
use rs_ai_local::foundationmodels::{is_available, respond};

if is_available() {
    let answer = respond("What is Rust?").await?;
}
```

### Windows (Phi Silica)

Enable the feature. Build with .NET SDK — `build.rs` auto-compiles the C# bridge:

```toml
rs_ai_local = { version = "0.2", features = ["phi-silica"] }
```

Use from Rust:

```rust
use rs_ai_local::phi_silica::respond;

let answer = respond("Hello!").await?;
```

## Examples

### Streaming
```rust
use rs_ai::rs_ai_claude;
use futures::StreamExt;

let mut stream = rs_ai_claude().stream("Write a poem").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

## Testing

```bash
cargo test
```

## License

MPL-2.0