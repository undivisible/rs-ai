use async_trait::async_trait;
use rs_ai_core::{AiError, AiResult, RerankedDocument, RerankingModel, RerankOptions, RerankResult, Usage};

/// Cohere reranking model implementing the `RerankingModel` trait.
pub struct CohereRerankingModel {
    model_id: String,
    api_key: String,
    base_url: String,
}

impl CohereRerankingModel {
    pub fn new(model_id: &str, api_key: &str, base_url: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

#[async_trait]
impl RerankingModel for CohereRerankingModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "cohere"
    }

    async fn rerank(
        &self,
        query: &str,
        documents: Vec<String>,
        options: RerankOptions,
    ) -> AiResult<RerankResult> {
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": self.model_id,
            "query": query,
            "documents": documents,
        });

        if let Some(top_k) = options.top_k {
            body["top_n"] = serde_json::json!(top_k);
        }
        if let Some(true) = options.return_documents {
            body["return_documents"] = serde_json::json!(true);
        }

        let resp = client
            .post(format!("{}/v2/rerank", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: format!("Cohere rerank request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        let data: serde_json::Value = resp.json().await.map_err(|e| AiError::Transport {
            message: format!("Cohere rerank parse failed: {e}"),
            source: Some(Box::new(e)),
        })?;

        let results = data["results"]
            .as_array()
            .ok_or_else(|| AiError::Transport {
                message: "Cohere rerank: missing results array".into(),
                source: None,
            })?
            .iter()
            .map(|r| {
                let doc = r["document"]
                    .as_object()
                    .and_then(|d| d.get("text").and_then(|t| t.as_str().map(String::from)));
                RerankedDocument {
                    index: r["index"].as_u64().unwrap_or(0) as usize,
                    score: r["relevance_score"].as_f64().unwrap_or(0.0),
                    document: doc,
                }
            })
            .collect();

        // Cohere v2 rerank doesn't return token usage; use defaults.
        Ok(RerankResult {
            results,
            usage: Usage::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohere_model_identity() {
        let model = CohereRerankingModel::new("rerank-v3.5", "test-key", "https://api.cohere.com");
        assert_eq!(model.provider_id(), "cohere");
        assert_eq!(model.model_id(), "rerank-v3.5");
    }
}
