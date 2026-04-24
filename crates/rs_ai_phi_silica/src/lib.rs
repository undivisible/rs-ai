//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Windows Phi Silica local runtime for the Rusty AI SDK.
//!
//! This crate provides integration with Microsoft's Phi Silica model
//! running on Windows devices with NPU support. Host applications must
//! implement the [`PhiSilicaBridge`] trait.

mod bridge;
mod model;
mod provider;
mod types;

pub use bridge::*;
pub use model::*;
pub use provider::*;
pub use types::*;
