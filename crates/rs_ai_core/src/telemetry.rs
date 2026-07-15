//! OpenTelemetry support for model calls.
//! Vercel-equivalent of `@ai-sdk/otel`.
//!
//! Uses `tracing-opentelemetry` bridge to export spans from our existing
//! `ObservableModel` to any OTLP-compatible backend.

use crate::model::LanguageModel;
use crate::observability::ObservableModel;

/// Configure OpenTelemetry export for model calls.
#[derive(Debug, Clone, Default)]
pub struct TelemetryConfig {
    /// Whether to record input/output content (may contain PII).
    pub record_content: bool,
    /// Service name for traces.
    pub service_name: Option<String>,
}

/// Initialize OpenTelemetry tracing with an OTLP exporter.
///
/// Call once at application startup. Requires `telemetry` feature.
#[cfg(feature = "telemetry")]
pub fn init_telemetry(config: &TelemetryConfig) -> Result<(), String> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::SpanExporter;
    use opentelemetry_sdk::runtime;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .map_err(|e| format!("Failed to create OTLP exporter: {e}"))?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new(
                "service.name",
                config
                    .service_name
                    .clone()
                    .unwrap_or_else(|| "rs_ai".into()),
            ),
        ]))
        .build();

    let tracer = provider.tracer("rs_ai");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set tracing subscriber: {e}"))?;

    Ok(())
}

/// Wrap a model with OpenTelemetry tracing instrumentation.
///
/// The returned `ObservableModel` emits tracing spans for each generate/stream
/// call. When the `telemetry` feature is enabled and `init_telemetry` has been
/// called, those spans are exported via OTLP.
pub fn with_telemetry(
    model: Box<dyn LanguageModel>,
    _config: TelemetryConfig,
) -> ObservableModel {
    ObservableModel::new(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(!config.record_content);
        assert!(config.service_name.is_none());
    }
}
