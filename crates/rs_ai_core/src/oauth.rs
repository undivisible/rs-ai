//! Re-exports from `rs_ai_oauth`.
//!
//! The OAuth flow and model discovery logic lives in the `rs_ai_oauth` crate.
//! This module re-exports it for backward compatibility.

pub use rs_ai_oauth::{
    fetch_models, fetch_models_async, refresh_oauth_token, start_oauth_flow, ModelInfo, OAuthError,
    OAuthProvider, OAuthTokens,
};
