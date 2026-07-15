//! WASM implementation of [`BrowserAiBridge`] using Chrome's Prompt API.
//!
//! Compiled only when `features = ["browser"]` and `target_arch = "wasm32"`.
//!
//! **Browser support (2026):**
//! - ✓ Chrome 138+ — `window.LanguageModel` (Gemini Nano on-device)
//! - ✓ Edge (Copilot+ PCs) — Phi Silica backing
//! - ✗ Firefox, Safari — not yet implemented
//!
//! **Building:**
//! ```sh
//! wasm-pack build --target web --features browser
//! ```
//!
//! **Note:** `LanguageModel` and related Prompt API types are not yet in
//! `web-sys` (they were added in Chrome 138 after the last web-sys release).
//! We bind them directly via `wasm_bindgen` `extern "C"` blocks.

#![cfg(all(target_arch = "wasm32", feature = "browser"))]

use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::capabilities::{BackingModel, BrowserAiCapabilities, BrowserAiOptions, BrowserType};

// ── Chrome Prompt API bindings (not yet in web-sys) ───────────────────────────
//
// Chrome 138+ exposes a top-level `LanguageModel` object (not on `window`,
// on the global itself) with static methods `availability()` and `create()`.

#[wasm_bindgen]
extern "C" {
    /// The `LanguageModel` global object (Chrome 138+ Prompt API).
    #[wasm_bindgen(js_namespace = globalThis, js_name = LanguageModel)]
    type LanguageModelGlobal;

    /// `LanguageModel.availability()` → Promise<"readily" | "after-download" | "no">
    #[wasm_bindgen(static_method_of = LanguageModelGlobal, js_name = availability, catch)]
    async fn availability() -> Result<JsValue, JsValue>;

    /// `LanguageModel.create(options)` → Promise<LanguageModelSession>
    #[wasm_bindgen(static_method_of = LanguageModelGlobal, js_name = create, catch)]
    async fn create(options: &JsValue) -> Result<JsValue, JsValue>;

    /// A live session returned by `LanguageModel.create()`.
    #[wasm_bindgen(js_name = Object)]
    type LanguageModelSession;

    /// `session.prompt(input)` → Promise<string>
    #[wasm_bindgen(method, js_name = prompt, catch)]
    async fn prompt(this: &LanguageModelSession, input: &str) -> Result<JsValue, JsValue>;

    /// `session.promptStreaming(input)` → ReadableStream<string>
    #[wasm_bindgen(method, js_name = promptStreaming)]
    fn prompt_streaming(this: &LanguageModelSession, input: &str) -> JsValue;

    /// `session.destroy()` — release resources.
    #[wasm_bindgen(method, js_name = destroy)]
    fn destroy(this: &LanguageModelSession);
}

/// The `ai.languageModel` namespace (newer Prompt API)
#[wasm_bindgen]
extern "C" {
    /// The `ai` namespace on globalThis
    type AiNamespace;

    /// `ai.languageModel`
    #[wasm_bindgen(method, getter, js_name = languageModel)]
    fn language_model(this: &AiNamespace) -> JsValue;

    /// Opaque type for `ai.languageModel`
    type LanguageModelNamespace;

    /// `languageModel.availability()`
    #[wasm_bindgen(method, js_name = availability, catch)]
    async fn lm_availability(this: &LanguageModelNamespace) -> Result<JsValue, JsValue>;

    /// `languageModel.create(options)`
    #[wasm_bindgen(method, js_name = create, catch)]
    async fn lm_create(
        this: &LanguageModelNamespace,
        options: &JsValue,
    ) -> Result<JsValue, JsValue>;

    /// `ai` global access on globalThis
    type GlobalAi;
    #[wasm_bindgen(js_namespace = globalThis, js_name = ai, getter)]
    fn get_ai() -> JsValue;
}

// ── User-agent helpers ────────────────────────────────────────────

fn user_agent() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_default()
}

fn is_chrome() -> bool {
    let ua = user_agent();
    ua.contains("Chrome") && !ua.contains("Edg")
}

fn is_edge() -> bool {
    user_agent().contains("Edg")
}

fn language_model_exists() -> bool {
    let global = js_sys::global();
    Reflect::has(&global, &JsValue::from_str("LanguageModel")).unwrap_or(false)
}

/// Returns `Some(languageModel)` if the new `self.ai.languageModel` API is available.
fn language_model_v2() -> Option<JsValue> {
    let ai_val = get_ai();
    if ai_val.is_undefined() || ai_val.is_null() {
        return None;
    }
    Reflect::get(&ai_val, &"languageModel".into())
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
}

// ── WasmBrowserBridge ────────────────────────────────────────────────

/// WASM bridge that calls Chrome's Prompt API (`LanguageModel` global).
///
/// Use this on WASM targets:
///
/// ```no_run
/// # #[cfg(all(target_arch = "wasm32", feature = "browser"))]
/// use rs_ai_local::browser::{wasm_bridge::WasmBrowserBridge, BrowserAiOptions};
///
/// # #[cfg(all(target_arch = "wasm32", feature = "browser"))]
/// # async fn example() -> Result<(), String> {
/// let answer = WasmBrowserBridge
///     .generate("Hello", &BrowserAiOptions::default())
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct WasmBrowserBridge;

impl WasmBrowserBridge {
    pub async fn detect(&self) -> BrowserAiCapabilities {
        let browser = if is_edge() {
            BrowserType::Edge
        } else if is_chrome() {
            BrowserType::Chrome
        } else {
            BrowserType::Unknown
        };

        let avail = if let Some(lm) = language_model_v2() {
            let lm_ns: &LanguageModelNamespace = lm.unchecked_ref();
            lm_ns
                .lm_availability()
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default()
        } else if language_model_exists() {
            LanguageModelGlobal::availability()
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default()
        } else {
            return BrowserAiCapabilities {
                available: false,
                browser,
                ..Default::default()
            };
        };

        let available = matches!(avail.as_str(), "readily" | "after-download");
        let backing_model = if is_edge() {
            BackingModel::PhiSilica
        } else {
            BackingModel::GeminiNano
        };

        BrowserAiCapabilities {
            available,
            browser,
            supports_streaming: true,
            supports_system_prompt: true,
            max_tokens: Some(4096),
            backing_model,
            supports_response_constraint: is_chrome(),
        }
    }

    pub async fn generate(
        &self,
        prompt_text: &str,
        options: &BrowserAiOptions,
    ) -> Result<String, String> {
        let session = create_session(options)
            .await
            .map_err(|e| format!("{e:?}"))?;

        let result = session
            .prompt(prompt_text)
            .await
            .map_err(|e| format!("{e:?}"))?;

        session.destroy();
        result
            .as_string()
            .ok_or_else(|| "No text in response".into())
    }

    pub async fn stream(
        &self,
        prompt_text: &str,
        options: &BrowserAiOptions,
    ) -> Result<Vec<String>, String> {
        let session = create_session(options)
            .await
            .map_err(|e| format!("{e:?}"))?;

        let readable = session.prompt_streaming(prompt_text);
        let chunks = drain_readable_stream(readable).await?;
        session.destroy();
        Ok(chunks)
    }
}

// ── Internal helpers ────────────────────────────────────────────────

async fn create_session(options: &BrowserAiOptions) -> Result<LanguageModelSession, JsValue> {
    let opts = Object::new();
    if let Some(ref sp) = options.system_prompt {
        Reflect::set(&opts, &"systemPrompt".into(), &sp.as_str().into())?;
    }
    if let Some(t) = options.temperature {
        Reflect::set(&opts, &"temperature".into(), &JsValue::from_f64(t))?;
    }
    if let Some(k) = options.top_k {
        Reflect::set(&opts, &"topK".into(), &JsValue::from_f64(k as f64))?;
    }
    if let Some(ref constraint) = options.response_constraint {
        let s = serde_json::to_string(constraint).unwrap_or_default();
        Reflect::set(&opts, &"responseConstraint".into(), &s.as_str().into())?;
    }

    // Try new Prompt API (self.ai.languageModel) first, fall back to legacy
    let session_val = if let Some(lm) = language_model_v2() {
        let lm_ns: &LanguageModelNamespace = lm.unchecked_ref();
        lm_ns.lm_create(&opts.into()).await?
    } else {
        LanguageModelGlobal::create(&opts.into()).await?
    };
    Ok(session_val.unchecked_into::<LanguageModelSession>())
}

/// Reads all chunks from a `ReadableStream<string>` and returns them as
/// a `Vec<String>`. Suitable for `promptStreaming()`.
async fn drain_readable_stream(stream: JsValue) -> Result<Vec<String>, String> {
    // Call stream.getReader()
    let get_reader = Reflect::get(&stream, &"getReader".into())
        .map_err(|e| format!("getReader not found: {e:?}"))?;
    let get_reader_fn = get_reader
        .dyn_ref::<Function>()
        .ok_or("getReader is not a function")?;
    let reader = get_reader_fn
        .call0(&stream)
        .map_err(|e| format!("getReader(): {e:?}"))?;

    let read_fn_key = JsValue::from_str("read");
    let mut chunks = Vec::new();

    loop {
        let read_fn =
            Reflect::get(&reader, &read_fn_key).map_err(|e| format!("read not found: {e:?}"))?;
        let read = read_fn
            .dyn_ref::<Function>()
            .ok_or("read is not a function")?;
        let promise = read.call0(&reader).map_err(|e| format!("read(): {e:?}"))?;
        let result = JsFuture::from(Promise::from(promise))
            .await
            .map_err(|e| format!("read promise: {e:?}"))?;

        let done = Reflect::get(&result, &"done".into())
            .map(|v| v.is_truthy())
            .unwrap_or(true);
        if done {
            break;
        }
        if let Some(chunk) = Reflect::get(&result, &"value".into())
            .ok()
            .and_then(|v| v.as_string())
        {
            chunks.push(chunk);
        }
    }
    Ok(chunks)
}
