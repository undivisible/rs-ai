# rs_ai Examples

Working examples demonstrating `rs_ai`'s capabilities. Each example is a workspace
member crate, so run it with `cargo run -p <package>` from the repo root.

> `cargo run --example <name>` does **not** work here — these are standalone member
> packages, not `[[example]]` targets of a single crate.

## Examples

### 1. [basic_text](basic_text/)
Generate text from ChatGPT and Claude with the low-level provider API.

```bash
OPENAI_API_KEY=sk-... ANTHROPIC_API_KEY=sk-ant-... cargo run -p example_basic_text
```

**Demonstrates:**
- `ChatGptProvider` / `ClaudeProvider` construction
- `generate_text()` over a `LanguageModel`
- One prompt against multiple providers

### 2. [stream_text](stream_text/)
Stream tokens as they are produced.

```bash
OPENAI_API_KEY=sk-... cargo run -p example_stream_text
```

**Demonstrates:**
- `AiStream` + `StreamExt::next()`
- Incremental text deltas

### 3. [fluent_api](fluent_api/)
Top-level fluent builder API — the ergonomic entry point.

```bash
ANTHROPIC_API_KEY=sk-ant-... OPENAI_API_KEY=sk-... GOOGLE_API_KEY=... \
  cargo run -p fluent_api
```

**Demonstrates:**
- `claude()`, `chatgpt()`, `gemini()`, `compatible()` entry points
- `.api_key(...)`, `.model(...)`, `.generate(...)` chaining
- OpenAI-compatible endpoints (OpenRouter, vLLM, etc.)

### 4. [generate_object](generate_object/)
Typed structured output from a JSON schema.

```bash
OPENAI_API_KEY=sk-... cargo run -p example_generate_object
```

**Demonstrates:**
- `generate_object()` with a `schemars::JsonSchema` type
- `ObjectResult<T>` deserialization

### 5. [stream_object](stream_object/)
Stream a structured object incrementally.

```bash
OPENAI_API_KEY=sk-... cargo run -p example_stream_object
```

**Demonstrates:**
- Streaming partial object state
- `JsonSchema`-driven schemas

### 6. [tool_loop](tool_loop/)
Agent loop with automatic tool execution.

```bash
OPENAI_API_KEY=sk-... cargo run -p example_tool_loop
```

**Demonstrates:**
- Implementing `Tool` with `async_trait`
- `ToolSet` registration
- `agent_loop()` with `max_steps`

### 7. [agent_with_tools](agent_with_tools/)
Claude tool use driven by streaming events.

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run -p example_agent_with_tools
```

**Demonstrates:**
- `ToolDefinition` + `schemars` parameter schemas
- `StreamEvent` tool-call handling
- Streaming agent loop

### 8. [streaming_tool_use](streaming_tool_use/)
Function-call arguments streamed as they are generated.

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run -p example_streaming_tool_use
```

**Demonstrates:**
- Incremental tool-call argument deltas
- Acting on a tool before the full response completes

### 9. [multimodal](multimodal/)
Image + text input (vision).

```bash
OPENAI_API_KEY=sk-... cargo run -p example_multimodal
```

**Demonstrates:**
- `Prompt` with image content
- Vision-capable model calls

### 10. [router](router/)
Model routing with mock providers — runs offline, no keys.

```bash
cargo run -p example_router
```

**Demonstrates:**
- `rs_ai_testing` mocks
- Routing prompts across models

### 11. [realtime_voice](realtime_voice/)
OpenAI Realtime API voice agent with function calling.

```bash
OPENAI_API_KEY=sk-... cargo run -p example_realtime_voice
```

**Demonstrates:**
- `RealtimeSession` over WebSocket
- `send_text` / `send_audio` / `recv` event loop
- `ServerEvent` and session configuration

### 12. [gemini_live](gemini_live/)
Gemini Live bidirectional voice/text conversation.

```bash
GOOGLE_API_KEY=... cargo run -p example_gemini_live
```

**Demonstrates:**
- Gemini Live session setup
- Streaming realtime events

### 13. [local_browser](local_browser/)
WASM browser AI (Chrome/Edge built-in models).

```bash
cargo build -p example_local_browser --target wasm32-unknown-unknown
```

**Demonstrates:**
- `browser` feature (`BrowserAiProvider`)
- `BrowserBridge` implementations

### 14. [local_apple](local_apple/)
Apple Foundation Models (macOS) usage pattern.

```bash
cargo run -p example_local_apple
```

**Demonstrates:**
- `foundationmodels` feature
- Mock bridge mirroring the real Swift/Obj-C interop

### 15. [local_android](local_android/)
Android Gemini Nano usage pattern.

```bash
cargo run -p example_local_android
```

**Demonstrates:**
- `gemini-nano` feature (`GeminiNanoProvider`)
- `MockNanoBridge` vs. `JniGeminiNanoBridge`

### 16. [local_windows](local_windows/)
Windows Phi Silica / WinML usage pattern.

```bash
cargo run -p example_local_windows
```

**Demonstrates:**
- `phi-silica` feature (`PhiSilicaProvider`)
- Mock bridge and Windows ML loader

## Quick Start

```bash
# Offline example (no API key)
cargo run -p example_router

# Streaming with an API key
OPENAI_API_KEY=sk-... cargo run -p example_stream_text

# Fluent API across providers
OPENAI_API_KEY=sk-... cargo run -p fluent_api
```

## Requirements

| Examples | Env var | Notes |
|---|---|---|
| basic_text | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` | Both required |
| fluent_api | `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY` | Per section |
| stream_text, generate_object, stream_object, tool_loop, multimodal, realtime_voice | `OPENAI_API_KEY` | |
| agent_with_tools, streaming_tool_use | `ANTHROPIC_API_KEY` | |
| gemini_live | `GOOGLE_API_KEY` | |
| router, local_* | — | Offline / mock bridges |

## Next Steps

1. Read the [main README](../README.md) for the full API surface.
2. See [AGENTS.md](../AGENTS.md) for workspace layout and provider internals.
3. Check the provider crates in [`crates/`](../crates/) for implementation details.
