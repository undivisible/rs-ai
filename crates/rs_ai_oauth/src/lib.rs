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
//!
//! For a long-running model picker, keep a catalog and refresh it periodically:
//!
//! ```no_run
//! # use rs_ai_oauth::{ModelCatalog, OAuthProvider};
//! # let token = String::new();
//! let mut catalog = ModelCatalog::new(OAuthProvider::Xai);
//! let update = catalog.refresh(&token).unwrap();
//! println!("new models: {}", update.added.len());
//! ```

pub mod claude_code;
mod fetch;
mod flow;

pub mod codex;
pub mod credentials;

pub use fetch::{
    fetch_logged_in_models, fetch_logged_in_models_async, fetch_models, fetch_models_async,
    ModelCatalog, ModelCatalogUpdate, ModelInfo, ModelLimits, ModelPricing, ProviderModels,
};
pub use flow::{refresh_oauth_token, start_oauth_flow, OAuthError, OAuthProvider, OAuthTokens};
