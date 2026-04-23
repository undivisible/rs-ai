# RAI Implementation Plan - Next Phase

## Overview

This document outlines the implementation roadmap for completing the RAI SDK with real-time APIs, WASM browser support, simplified API integration, comprehensive examples, and advanced features.

---

## Phase 1: Real-Time APIs & WebSocket Support

### 1.1 Add WebSocket Streaming to Providers

#### `rai_claude` - Streaming Tool Use
```rust
pub trait LanguageModel {
    // IMPLEMENT: Real-time tool use streaming
    async fn stream_with_tools(
        &self,
        request: GenerateRequest,
        tools: ToolSet,
    ) -> Result<Stream<StreamEvent>>;
}
```

**Implementation Steps:**
1. Create `crates/rai_claude/src/streaming.rs` for tool streaming logic
2. Implement event parser for tool use events from stream
3. Add `ToolUseEvent` variant to `StreamEvent` enum
4. Create integration test: `test_streaming_tool_use()`
5. Create example: `examples/streaming_tool_use/`

**Files to Create/Modify:**
- `crates/rai_claude/src/streaming.rs` (NEW)
- `crates/rai_ai/src/stream.rs` (MODIFY - add ToolUseEvent)
- `examples/streaming_tool_use/` (NEW)

---

#### `rai_gemini` - Live API WebSocket

```rust
pub struct GeminiLiveSession {
    ws: WebSocket,
    model_id: String,
}

impl GeminiLiveSession {
    pub async fn send_audio(&self, audio: Vec<u8>) -> Result<()> { }
    pub async fn receive_audio(&self) -> Result<Vec<u8>> { }
    pub async fn send_text(&self, text: &str) -> Result<()> { }
}
```

**Implementation Steps:**
1. Add `tokio-tungstenite` dependency (now added to Cargo.toml)
2. Create `crates/rai_gemini/src/live_api.rs`
3. Implement WebSocket connection management
4. Handle audio frame encoding/decoding
5. Create example: `examples/gemini_live_audio/`

**Files to Create/Modify:**
- `crates/rai_gemini/src/live_api.rs` (NEW)
- `examples/gemini_live_audio/` (NEW)
- `examples/gemini_live_video/` (NEW)

---

#### `rai_chatgpt` - Realtime API

```rust
pub struct RealtimeSession {
    ws: WebSocket,
    audio_config: AudioConfig,
}

impl RealtimeSession {
    pub async fn send_audio(&self, audio: Vec<u8>) -> Result<()> { }
    pub async fn receive_audio(&self) -> Result<Vec<u8>> { }
    pub async fn send_function_result(&self, tool_id: &str, result: String) -> Result<()> { }
}
```

**Implementation Steps:**
1. Create `crates/rai_chatgpt/src/realtime_api.rs`
2. Implement WebSocket connection for audio streaming
3. Handle function calling over realtime API
4. Add audio codec support (required by OpenAI API)
5. Create example: `examples/chatgpt_voice_agent/`

**Files to Create/Modify:**
- `crates/rai_chatgpt/src/realtime_api.rs` (NEW)
- `examples/chatgpt_voice_agent/` (NEW)
- `examples/chatgpt_voice_with_tools/` (NEW)

---

## Phase 2: Browser Runtime & WASM Support

### 2.1 Implement Browser Prompt API

**Current Status:** Trait definition only, no WASM bindings

```rust
// File: crates/rai_browser/src/wasm_bindings.rs (NEW)
#[wasm_bindgen]
pub struct PromptApi {
    model: web_sys::LanguageModel,
}

impl PromptApi {
    #[wasm_bindgen]
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        // Call window.LanguageModel.complete()
    }
}
```

**Implementation Steps:**
1. Add WASM dependencies to Cargo.toml (DONE ✅)
2. Create `crates/rai_browser/src/wasm_bindings.rs` with:
   - Chrome Prompt API bindings
   - Browser capability detection
   - Error handling for unsupported browsers
3. Implement `BrowserAiBridge` trait for WASM target
4. Create example: `examples/browser_prompt_api/`
5. Add browser integration test (requires Chrome 138+)

**Files to Create/Modify:**
- `crates/rai_browser/src/wasm_bindings.rs` (NEW)
- `crates/rai_browser/src/lib.rs` (MODIFY - feature gates)
- `examples/browser_prompt_api/` (NEW)

**Browser Compatibility Matrix:**
```
✅ Chrome 138+ (all platforms)
✅ Edge (Copilot+ devices)
❌ Firefox (pending)
❌ Safari (pending)
⚠️ Mobile (limited)
```

---

## Phase 3: Simplified API Integration

### 3.1 Connect `rai_claude()` to Provider

**Current:** Stub implementation, needs actual provider integration

```rust
// CURRENT (stub)
pub async fn rai_claude(_model_id: &str) -> AiResult<SimpleModel> {
    Err(AiError::ProviderError { ... })
}

// NEW (actual implementation)
pub async fn rai_claude(model_id: &str) -> AiResult<SimpleModel> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| AiError::AuthError { ... })?;
    
    let provider = rai_claude::ClaudeProvider::new(api_key);
    let model = provider.language_model(model_id)?;
    
    Ok(SimpleModel::new(Box::new(model)))
}
```

**Implementation Steps:**
1. Update `rai_ai/src/simple.rs` for all providers:
   - `rai_claude()` 
   - `rai_chatgpt()`
   - `rai_gemini()`
   - `rai_compatible()`
2. Add feature flags for optional provider dependencies
3. Create integration tests for simplified API
4. Create examples showing simplified API usage

**Files to Modify:**
- `crates/rai_ai/src/simple.rs` (MODIFY)
- `crates/rai_ai/tests/simple_api.rs` (MODIFY - expand tests)

**Feature Flag Structure:**
```toml
# In Cargo.toml
[features]
default = ["claude", "chatgpt", "gemini"]
claude = ["rai_claude"]
chatgpt = ["rai_chatgpt"]
gemini = ["rai_gemini"]
all-providers = ["claude", "chatgpt", "gemini", "ollama"]
```

---

## Phase 4: Agent & Tool Use Examples

### 4.1 Create Tool Use Agent Loop Example

```rust
// examples/tool_use_agent/src/main.rs
async fn run_agent() {
    let model = rai_claude("claude-sonnet-4-6").await?;
    let tools = vec![
        ToolDefinition { name: "get_weather", ... },
        ToolDefinition { name: "search_web", ... },
    ];
    
    let mut messages = vec![];
    
    loop {
        let response = model.stream(&prompt, options).await?;
        
        match response {
            StreamEvent::ToolUse { tool_name, input } => {
                let result = execute_tool(&tool_name, &input).await?;
                messages.push((tool_name, result));
            }
            StreamEvent::MessageEnd { .. } => break,
            _ => {}
        }
    }
}
```

**Implementation Steps:**
1. Create `examples/agent_with_tools/` crate
2. Implement tool execution loop
3. Add memory/conversation history
4. Add tool error handling and retries
5. Create documentation on agent patterns

**Files to Create:**
- `examples/agent_with_tools/` (NEW - complete example)
- Documentation: `AGENT_PATTERNS.md` (NEW)

---

### 4.2 Multi-Provider Agent Example

Show agent using different providers for different tasks:

```rust
// examples/multi_provider_agent/src/main.rs
// Use Claude for planning
// Use ChatGPT for coding
// Use Gemini for research
```

**Files to Create:**
- `examples/multi_provider_agent/` (NEW)

---

## Phase 5: Documentation & Guides

### 5.1 Create Architecture Guides

**Files to Create:**
- `ARCHITECTURE.md` - Overall design decisions
- `AGENT_PATTERNS.md` - How to build agents
- `REAL_TIME_GUIDE.md` - Gemini Live & Realtime APIs
- `BROWSER_GUIDE.md` - Using Prompt API in WASM
- `STREAMING_GUIDE.md` - Handling streaming responses
- `TOOL_USE_GUIDE.md` - Function calling patterns

### 5.2 Create Comparison Matrices

- Provider feature matrix (vision, tool use, streaming, etc.)
- Real-time API comparison
- Local vs cloud provider comparison

### 5.3 Create Migration Guide

For users upgrading from `rusty_ai` to `rai`:
- Renamed crates
- New simplified API
- Breaking changes
- Migration examples

---

## Phase 6: Testing & Quality

### 6.1 Add Mock Testing Infrastructure

```rust
// crates/rai_testing/src/lib.rs
pub struct MockModel { ... }
impl LanguageModel for MockModel { ... }

// Usage in tests:
#[tokio::test]
async fn test_agent_with_mock() {
    let mock = MockModel::new()
        .with_response("Hello")
        .with_tool_call("search", "query");
    
    // Test agent logic without API calls
}
```

**Implementation Steps:**
1. Expand `rai_testing` with comprehensive mocks
2. Add `wiremock` for HTTP mocking (now in Cargo.toml)
3. Create property-based tests with `proptest`
4. Add streaming response mocks

**Files to Modify:**
- `crates/rai_testing/src/lib.rs` (EXPAND)
- Add property tests in each provider crate

### 6.2 Add Integration Tests

Create real integration tests against actual APIs (when available):

```
tests/
├── claude_integration.rs
├── chatgpt_integration.rs
├── gemini_integration.rs
└── ollama_integration.rs
```

---

## Implementation Timeline

| Phase | Priority | Est. Time | Status |
|-------|----------|-----------|--------|
| Phase 1: Real-Time APIs | 🔴 HIGH | 2 weeks | Not Started |
| Phase 2: Browser WASM | 🔴 HIGH | 1 week | Not Started |
| Phase 3: Simplified API | 🟡 MEDIUM | 3 days | Not Started |
| Phase 4: Agent Examples | 🟡 MEDIUM | 1 week | Not Started |
| Phase 5: Documentation | 🟡 MEDIUM | 1 week | Not Started |
| Phase 6: Testing | 🟡 MEDIUM | 1 week | Not Started |

**Total Estimated: 6-7 weeks for full completion**

---

## Success Criteria

- [ ] All real-time APIs (WebSocket) working
- [ ] Browser Prompt API fully functional
- [ ] Simplified API connected to providers
- [ ] 10+ examples covering different use cases
- [ ] 100+ integration tests
- [ ] Comprehensive documentation
- [ ] 90%+ code coverage
- [ ] Zero unsafe code (unless justified)
- [ ] All clippy warnings resolved
- [ ] Performance benchmarks established

---

## Rollout Strategy

1. **Week 1-2:** Real-time APIs (WebSocket support)
2. **Week 3:** Browser WASM + Simplified API integration
3. **Week 4-5:** Examples + agent patterns
4. **Week 6-7:** Documentation + testing polish

Each phase gets PR review before moving to next.

