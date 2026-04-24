/// Availability state of Apple Foundation Models on this device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppleModelAvailability {
    /// The model is available and ready to use.
    Available,
    /// The model is not available.
    Unavailable { reason: String },
    /// The model needs to be downloaded first.
    NeedsDownload,
}

/// Configuration for Foundation Model generation.
#[derive(Debug, Clone, Default)]
pub struct FoundationModelConfig {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}
