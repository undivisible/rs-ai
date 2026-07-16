//! Runtime model discovery via `GET /v1/models`.
//!
//! Both xAI and OpenAI expose a standard OpenAI-compatible models endpoint
//! that returns the list of models available to the authenticated key.
//! We fetch at runtime instead of hardcoding catalogs.

use crate::flow::{OAuthError, OAuthProvider};
use serde::Deserialize;

/// Minimal model info from the `/v1/models` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
    pub owned_by: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

/// Fetch available models synchronously.
///
/// Calls `GET {provider.api_base()}/models` with a Bearer token.
/// Returns the list of model IDs available to this token.
pub fn fetch_models(provider: OAuthProvider, token: &str) -> Result<Vec<ModelInfo>, OAuthError> {
    let url = format!("{}/models", provider.api_base());
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| OAuthError::Network(format!("models request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(OAuthError::Network(format!(
            "models request returned status {}",
            response.status()
        )));
    }

    let body: ModelsResponse = response
        .json()
        .map_err(|e| OAuthError::Network(format!("models parse failed: {e}")))?;

    Ok(body.data)
}

/// Fetch available models asynchronously.
pub async fn fetch_models_async(
    provider: OAuthProvider,
    token: &str,
) -> Result<Vec<ModelInfo>, OAuthError> {
    let url = format!("{}/models", provider.api_base());
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("models request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(OAuthError::Network(format!(
            "models request returned status {}",
            response.status()
        )));
    }

    let body: ModelsResponse = response
        .json()
        .await
        .map_err(|e| OAuthError::Network(format!("models parse failed: {e}")))?;

    Ok(body.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_info_deserializes() {
        let json = serde_json::json!({
            "id": "grok-4.5",
            "created": 1700000000,
            "owned_by": "xai"
        });
        let info: ModelInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.id, "grok-4.5");
        assert_eq!(info.owned_by, "xai");
    }

    #[test]
    fn models_response_deserializes() {
        let json = serde_json::json!({
            "data": [
                {"id": "grok-4.5", "created": 1700000000, "owned_by": "xai"},
                {"id": "grok-4.3", "created": 1700000001, "owned_by": "xai"}
            ]
        });
        let resp: ModelsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].id, "grok-4.5");
        assert_eq!(resp.data[1].id, "grok-4.3");
    }
}
