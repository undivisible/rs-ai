//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Platform-specific local AI runtimes for the Rusty AI SDK.
//!
//! Each runtime is gated behind a Cargo feature flag:
//! - `browser` — WASM browser AI (Chrome/Edge built-in AI)
//! - `gemini-nano` — Android Gemini Nano (Prompt API)
//! - `foundationmodels` — Apple Foundation Models
//! - `phi-silica` — Windows Phi Silica

#[cfg(feature = "browser")]
pub mod browser;
#[cfg(feature = "foundationmodels")]
pub mod foundationmodels;
#[cfg(feature = "gemini-nano")]
pub mod gemini_nano;
#[cfg(feature = "phi-silica")]
pub mod phi_silica;

// Re-exports for convenience when the corresponding feature is enabled.
#[cfg(feature = "browser")]
pub use browser::*;
#[cfg(feature = "foundationmodels")]
pub use foundationmodels::*;
#[cfg(feature = "gemini-nano")]
pub use gemini_nano::*;
#[cfg(feature = "phi-silica")]
pub use phi_silica::*;
