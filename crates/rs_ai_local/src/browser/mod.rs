//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Browser-based local AI model detection for WASM targets.
//!
//! This crate detects and bridges to browser-based AI APIs:
//! - Chrome's built-in AI (Prompt API / `window.ai`)
//! - Edge's built-in AI
//!
//! Compatible with Rust WASM frameworks: Dioxus, Leptos, Yew.
//!
//! On WASM targets, the bridge is implemented via `wasm-bindgen` to call
//! the browser's AI APIs. On non-WASM targets, a no-op implementation is
//! provided for testing.

mod bridge;
mod capabilities;
mod model;
mod provider;
pub mod wasm_bridge;

pub use bridge::*;
pub use capabilities::*;
pub use model::*;
pub use provider::*;
