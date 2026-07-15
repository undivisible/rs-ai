//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Google Gemini provider for the Rusty AI SDK.

mod api_types;
mod convert;
pub(crate) mod image;
pub mod live_api;
mod model;
mod provider;
mod stream_parser;

pub(crate) mod speech;
pub(crate) mod video;

pub use image::{
    GeminiImageModel, GEMINI_2_5_FLASH_IMAGE, GEMINI_3_1_FLASH_IMAGE, GEMINI_3_PRO_IMAGE, IMAGEN_3,
    IMAGEN_3_FAST, IMAGEN_4, IMAGEN_4_FAST, IMAGEN_4_ULTRA,
};
pub use live_api::{GeminiLiveSessionAdapter, LiveSession};
pub use model::*;
pub use provider::*;
pub use speech::GeminiSpeechModel;
pub use video::GeminiVideoModel;
