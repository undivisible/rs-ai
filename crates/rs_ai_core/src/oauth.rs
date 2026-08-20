//! Re-exports from `rs_ai_oauth`.
//!
//! The OAuth flow and model discovery logic lives in the `rs_ai_oauth` crate.
//! This module re-exports it for backward compatibility.

pub use rs_ai_oauth::{
    fetch_logged_in_models, fetch_logged_in_models_async, fetch_models, fetch_models_async,
    refresh_oauth_token, start_oauth_flow, ModelCatalog, ModelCatalogUpdate, ModelInfo,
    ModelLimits, ModelPricing, OAuthError, OAuthProvider, OAuthTokens, ProviderModels,
};

/// Convert OAuth discovery metadata into the core model metadata shape.
impl From<rs_ai_oauth::ModelInfo> for crate::ModelInfo {
    fn from(model: rs_ai_oauth::ModelInfo) -> Self {
        let mut capabilities = crate::CapabilitySet::new();
        for capability in model.capabilities {
            let mapped = match capability.as_str() {
                "text_input" => Some(crate::Capability::TextInput),
                "text_output" => Some(crate::Capability::TextOutput),
                "image_input" => Some(crate::Capability::ImageInput),
                "image_output" => Some(crate::Capability::ImageOutput),
                "streaming" => Some(crate::Capability::Streaming),
                "tool_calling" => Some(crate::Capability::ToolCalling),
                "structured_output" => Some(crate::Capability::StructuredOutput),
                "embeddings" => Some(crate::Capability::Embeddings),
                "extended_thinking" => Some(crate::Capability::ExtendedThinking),
                "audio_input" => Some(crate::Capability::AudioInput),
                "audio_output" => Some(crate::Capability::AudioOutput),
                "image_generation" => Some(crate::Capability::ImageGeneration),
                "video_input" => Some(crate::Capability::VideoInput),
                "video_generation" => Some(crate::Capability::VideoGeneration),
                "realtime_voice" => Some(crate::Capability::RealtimeVoice),
                "realtime_video" => Some(crate::Capability::RealtimeVideo),
                _ => None,
            };
            if let Some(mapped) = mapped {
                capabilities = capabilities.with(mapped);
            }
        }

        crate::ModelInfo {
            id: model.id,
            provider: model.provider,
            display_name: model.display_name,
            capabilities,
            limits: crate::ModelLimits {
                context_window: model.limits.context_window,
                max_input_tokens: model.limits.max_input_tokens,
                max_output_tokens: model.limits.max_output_tokens,
            },
            pricing: model.pricing.map(|pricing| crate::ModelPricing {
                input_per_token: pricing.input_per_token,
                output_per_token: pricing.output_per_token,
                request: pricing.request,
                image_input: pricing.image_input,
                reasoning: pricing.reasoning,
                cache_read: pricing.cache_read,
                cache_write: pricing.cache_write,
            }),
            input_modalities: model.input_modalities,
            output_modalities: model.output_modalities,
            supported_parameters: model.supported_parameters,
            description: model.description,
            created: (model.created != 0).then_some(model.created),
            owned_by: (!model.owned_by.is_empty()).then_some(model.owned_by),
            expiration_date: model.expiration_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::capability::Capability;

    fn oauth_model(caps: &[&str], created: u64, owned_by: &str) -> rs_ai_oauth::ModelInfo {
        rs_ai_oauth::ModelInfo {
            id: "m".into(),
            provider: "chatgpt".into(),
            created,
            owned_by: owned_by.into(),
            display_name: "M".into(),
            description: None,
            capabilities: caps.iter().map(|cap| (*cap).to_string()).collect(),
            input_modalities: vec!["text".into()],
            output_modalities: vec!["text".into()],
            supported_parameters: BTreeSet::new(),
            limits: rs_ai_oauth::ModelLimits::default(),
            pricing: None,
            expiration_date: None,
        }
    }

    #[test]
    fn oauth_modelinfo_from_maps_known_capabilities() {
        let converted = crate::ModelInfo::from(oauth_model(
            &[
                "text_input",
                "text_output",
                "image_input",
                "image_output",
                "streaming",
                "tool_calling",
                "structured_output",
                "embeddings",
                "extended_thinking",
                "audio_input",
                "audio_output",
                "image_generation",
                "video_input",
                "video_generation",
                "realtime_voice",
                "realtime_video",
            ],
            1,
            "openai",
        ));
        for cap in [
            Capability::TextInput,
            Capability::TextOutput,
            Capability::ImageInput,
            Capability::ImageOutput,
            Capability::Streaming,
            Capability::ToolCalling,
            Capability::StructuredOutput,
            Capability::Embeddings,
            Capability::ExtendedThinking,
            Capability::AudioInput,
            Capability::AudioOutput,
            Capability::ImageGeneration,
            Capability::VideoInput,
            Capability::VideoGeneration,
            Capability::RealtimeVoice,
            Capability::RealtimeVideo,
        ] {
            assert!(converted.capabilities.has(&cap), "{cap:?}");
        }
        assert_eq!(converted.created, Some(1));
        assert_eq!(converted.owned_by.as_deref(), Some("openai"));
    }

    #[test]
    fn oauth_from_drops_unknown_capability() {
        let converted = crate::ModelInfo::from(oauth_model(&["not_a_cap", "text_input"], 0, ""));
        assert!(converted.capabilities.has(&Capability::TextInput));
        assert_eq!(converted.capabilities.iter().count(), 1);
    }

    #[test]
    fn oauth_from_treats_zero_created_and_empty_owned_by_as_none() {
        let converted = crate::ModelInfo::from(oauth_model(&[], 0, ""));
        assert_eq!(converted.created, None);
        assert_eq!(converted.owned_by, None);
    }
}
