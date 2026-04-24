//! Apple Foundation Models bridge for the Rusty AI SDK.
//!
//! This crate provides integration with Apple's Foundation Models framework.
//! The actual Swift/ObjC bridge lives in the separate `undivisible/rs_ai_foundationmodels`
//! repository. This crate provides the [`rs_ai_ai::LanguageModel`] integration layer.
//!
//! Host applications must implement the [`FoundationModelBridge`] trait.

mod bridge;
mod model;
mod provider;
mod types;

pub use bridge::*;
pub use model::*;
pub use provider::*;
pub use types::*;
