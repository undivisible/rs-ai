//! OAuth flows and runtime model discovery for AI providers.
//!
//! Supports xAI Grok and OpenAI ChatGPT via PKCE S256 browser OAuth.
//! Model lists are fetched at runtime from `GET /v1/models` — no hardcoded catalogs.
//!
//! # Example
//!
//! ```no_run
//! use rs_ai_oauth::{start_oauth_flow, OAuthProvider, fetch_models};
//!
//! // 1. Login — opens browser, waits for callback
//! let tokens = start_oauth_flow(OAuthProvider::Xai).unwrap();
//! println!("access token: {}", tokens.access_token);
//!
//! // 2. Fetch available models at runtime
//! let models = fetch_models(OAuthProvider::Xai, &tokens.access_token).unwrap();
//! for m in &models {
//!     println!("  {}", m.id);
//! }
//! ```

mod fetch;
mod flow;

pub use fetch::{fetch_models, fetch_models_async, ModelInfo};
pub use flow::{refresh_oauth_token, start_oauth_flow, OAuthError, OAuthProvider, OAuthTokens};
