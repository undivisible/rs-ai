//! UI stream protocol for frontend integration with the Rusty AI SDK.
//!
//! This crate provides typed UI stream events and encoders for two common
//! wire formats: **SSE** (Server-Sent Events) and **NDJSON** (Newline-Delimited
//! JSON). Both encoders accept an [`rai_ai::AiStream`] from a Rusty AI provider and
//! produce a byte stream suitable for sending over HTTP.

mod event;
mod ndjson;
mod sse;

pub use event::*;
pub use ndjson::*;
pub use sse::*;
