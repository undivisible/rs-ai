# RAI SDK Testing Endpoints & Mocking Guide

## 1. Mock Provider (Built-in Testing)

The `rai_testing` crate provides comprehensive mocking without external dependencies:

```rust
use rai_testing::{MockLanguageModel, MockResponse};
use rai_ai::LanguageModel;

// Create a mock model with predefined responses
let mock = MockLanguageModel::new("test-model")
    .with_text("Hello, world!")
    .with_text("Second response")
    .with_error(AiError::ProviderError {
        provider: "test".into(),
        status: Some(500),
        message: "Server error".into(),
    });

// Use it like any other model
let result = mock.generate(prompt, options).await?;
```

**Benefits:**
- No external dependencies
- FIFO response queue
- Records all invocations with timestamps
- Supports text, tool calls, errors, and JSON objects
- Perfect for unit/integration tests

## 2. Local Development Endpoints

### Ollama (Local LLM)
- **Default:** `http://localhost:11434`
- **Setup:** [Download Ollama](https://ollama.ai)
- **Usage:**
```rust
let provider = OllamaProvider::new()
    .with_base_url("http://localhost:11434");
let model = provider.model("llama2"); // or mistral, neural-chat, etc.
```

### OpenAI-Compatible Presets

#### vLLM (High-throughput local inference)
- **Default:** `http://localhost:8000/v1`
- **Setup:** `pip install vllm && vllm serve mistral-7b`

#### LM Studio (GUI-based local)
- **Default:** `http://localhost:1234/v1`
- **Setup:** Download from lmstudio.ai

#### Text-generation-webui
- **Default:** `http://localhost:5000/v1`
- **Setup:** [GitHub repo](https://github.com/oobabooga/text-generation-webui)

#### Using with RAI:
```rust
use rai_openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleProvider};

let config = OpenAiCompatibleConfig::vllm("http://localhost:8000/v1");
let provider = OpenAiCompatibleProvider::new(config, "vllm", "vLLM");
let model = provider.model("mistral-7b-instruct")?;
```

## 3. Cloud Provider Test/Sandbox Endpoints

### Anthropic (Claude)
- **Production:** `https://api.anthropic.com`
- **Requires:** Real API key from console.anthropic.com
- **Testing:** Use mock models, no sandbox environment
```rust
let provider = ClaudeProvider::new("sk-ant-...");
// Custom URL for proxy testing:
let provider = ClaudeProvider::new("sk-ant-...").with_base_url("http://proxy:8080");
```

### Google Gemini
- **Production:** `https://generativelanguage.googleapis.com`
- **Requires:** Real API key from Google Cloud Console
- **Testing:** Mock models recommended
```rust
let provider = GeminiProvider::new("AIza...");
// Custom URL support:
let model = provider.model_with_base_url("gemini-2.5-flash", "http://proxy:8080");
```

### OpenAI (ChatGPT)
- **Production:** `https://api.openai.com/v1`
- **Requires:** Real API key
- **Testing:** Mock models, no sandbox
```rust
let provider = ChatGptProvider::new("sk-...");
// Organization support:
let provider = ChatGptProvider::new("sk-...").with_org("org-...");
```

### xAI (Grok)
- **Production:** `https://api.x.ai/v1`
- **Requires:** Real API key
```rust
let provider = XaiProvider::new("xai-...");
```

### Portkey (Multi-provider Gateway)
- **Production:** `https://api.portkey.ai/v1`
- **Requires:** Real Portkey API key
- **Features:** Routes to any provider
```rust
let provider = PortkeyProvider::new("pk_...");
// Custom URL:
let provider = PortkeyProvider::new("pk_...").with_base_url("https://custom.gateway/v1");
```

## 4. Recommended Testing Strategy

### Unit Tests (No External Calls)
```rust
#[tokio::test]
async fn test_with_mock() {
    let mock = MockLanguageModel::new("test")
        .with_text("Expected response");
    
    let result = mock.generate(prompt, options).await.unwrap();
    assert_eq!(result.text, Some("Expected response".to_string()));
}
```

### Integration Tests (Local Endpoints)
```rust
#[tokio::test]
#[ignore] // Skip unless running with local services
async fn test_with_ollama() {
    let provider = OllamaProvider::new();
    let model = provider.model("mistral");
    
    let result = model.generate(prompt, options).await;
    assert!(result.is_ok());
}
```

### End-to-End Tests (Cloud Providers)
```rust
#[tokio::test]
#[ignore] // Skip unless INTEGRATION_TEST=1
async fn test_with_claude() {
    let key = std::env::var("ANTHROPIC_API_KEY").expect("missing key");
    let provider = ClaudeProvider::new(key);
    let model = provider.claude_sonnet();
    
    let result = model.generate(prompt, options).await;
    assert!(result.is_ok());
}
```

## 5. Environment-Based Testing

Create a test configuration:

```rust
// tests/common/mod.rs
pub fn create_test_model() -> Box<dyn LanguageModel> {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        Box::new(ClaudeProvider::new(key).claude_sonnet())
    } else {
        Box::new(MockLanguageModel::new("mock").with_text("test"))
    }
}
```

Run tests:
```bash
# Unit tests only (fast)
cargo test --lib

# Integration tests with local services
cargo test --test '*' -- --ignored

# Full E2E tests (requires API keys)
ANTHROPIC_API_KEY=sk-... cargo test --test '*'
```

## 6. Custom Base URL Pattern

All cloud providers support custom base URLs for testing proxies:

```rust
// Test through a proxy
let provider = ClaudeProvider::new(api_key)
    .with_base_url("http://localhost:8080/anthropic");

// Test with custom gateway
let portkey = PortkeyProvider::new(api_key)
    .with_base_url("http://gateway.internal/v1");

// Custom API endpoint
let gemini = GeminiProvider::new(api_key);
let model = gemini.model_with_base_url(
    "gemini-2.5-flash",
    "http://api.internal:8080"
);
```

## 7. Current Test Coverage

- **44 inline unit tests** (mocks + validation)
- **81 integration tests** (provider APIs)
- **125 total tests** (all passing)

To run full test suite:
```bash
cargo test --lib --test '*'
```

## Recommended Next Steps

1. **Add mock-based tests** for streaming and tool calling
2. **Add Ollama integration tests** (requires local Ollama instance)
3. **Add VCR/cassette recording** for API responses (once stable)
4. **Add load testing** with mock models for high-concurrency scenarios
