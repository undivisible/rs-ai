//! Gemini Nano (Android Prompt API) local runtime for the Rusty AI SDK.
//!
//! This crate provides a bridge-based integration with Google's Gemini Nano
//! model running on-device via the Android Prompt API. Host applications must
//! implement the [`GeminiNanoBridge`] trait to connect the JNI/Kotlin layer.

mod bridge;
mod model;
mod provider;
mod session;
mod types;

pub use bridge::*;
pub use model::*;
pub use provider::*;
pub use session::*;
pub use types::*;
