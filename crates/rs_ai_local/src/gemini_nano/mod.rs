//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Gemini Nano (Android Prompt API) local runtime for the Rusty AI SDK.
//!
//! This crate provides JNI-based integration with Google's Gemini Nano model
//! running on-device via the Android Prompt API.
//!
//! # Architecture
//!
//! ```text
//! Rust (rs_ai_gemini_nano)
//!   │ JNI calls
//!   ▼
//! Kotlin (GeminiNanoBridge)
//!   │ Android SDK
//!   ▼
//! Android Prompt API (Gemini Nano on-device)
//! ```
//!
//! # Setup
//!
//! 1. Add the Kotlin bridge class to your Android app (see `kotlin/GeminiNanoBridge.kt`)
//! 2. Create the bridge instance from Rust using `JniGeminiNanoBridge::new`
//! 3. Pass it to `GeminiNanoProvider` to get a `LanguageModel`
//!
//! # Requirements
//!
//! - Android API 33+ (Android 13)
//! - Google Play Services with AI Core
//! - Device must support on-device ML

#![deny(missing_docs)]

mod bridge;
#[cfg(target_os = "android")]
mod jni_bridge;
mod model;
mod provider;
mod session;
mod types;

pub use bridge::*;
#[cfg(target_os = "android")]
pub use jni_bridge::*;
pub use model::*;
pub use provider::*;
pub use session::*;
pub use types::*;

// Re-export rs_ai_traits for convenience
pub use rs_ai_core;
