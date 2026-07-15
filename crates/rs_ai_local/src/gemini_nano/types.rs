// ─── New ML Kit GenAI Prompt API types ───────────────────────────────────────

/// Configuration for generation (mirrors GenerationConfig in ML Kit).
#[derive(Debug, Clone, Default)]
pub struct NanoGenerationConfig {
    /// Sampling temperature (0.0 - 1.0).
    pub temperature: Option<f32>,
    /// Number of candidates to generate.
    pub candidate_count: Option<u32>,
    /// Maximum number of output tokens.
    pub max_output_tokens: Option<u32>,
}

/// A content part for the prompt (text or image).
#[derive(Debug, Clone)]
pub enum NanoContentPart {
    /// Plain text content.
    Text(String),
    /// Base64-encoded image bytes with MIME type.
    Image {
        /// Base64-encoded image data.
        base64: String,
        /// MIME type of the image (e.g. "image/jpeg").
        mime_type: String,
    },
}

/// Result from generate_content.
#[derive(Debug, Clone)]
pub struct NanoGenerateResult {
    /// Generated text content.
    pub text: String,
}

// ─── Deprecated types (kept for backward compat) ─────────────────────────────

/// The download state of the Gemini Nano model on the device.
///
/// ⚠️ Deprecated in favor of the ML Kit GenAI Prompt API which handles
/// model download automatically. Use `is_available` to check readiness.
#[deprecated(
    since = "0.3.0",
    note = "AICore handles model download automatically; use is_available()"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelDownloadState {
    /// The model has not been downloaded to the device.
    NotDownloaded,
    /// The model is currently being downloaded.
    Downloading {
        /// Download progress as a percentage (0-100).
        progress_percent: u8,
    },
    /// The model has been downloaded and is ready for use.
    Downloaded,
    /// The model download failed.
    Failed {
        /// A human-readable reason for the failure.
        reason: String,
    },
}

/// Capabilities exposed by Gemini Nano on the current device.
#[deprecated(since = "0.3.0", note = "Use is_available() to check device readiness")]
#[derive(Debug, Clone)]
pub struct NanoCapabilities {
    /// Whether text generation is supported.
    pub text_generation: bool,
    /// Whether summarization is supported.
    pub summarization: bool,
    /// Whether rewriting is supported.
    pub rewriting: bool,
}

/// Configuration for a Gemini Nano session.
///
/// ⚠️ Deprecated — the new ML Kit API uses `NanoGenerationConfig`
/// and does not support sessions. Use `generate_content` instead.
#[deprecated(
    since = "0.3.0",
    note = "Use NanoGenerationConfig with generate_content instead"
)]
#[derive(Debug, Clone, Default)]
pub struct NanoSessionConfig {
    /// Sampling temperature (0.0 - 1.0).
    pub temperature: Option<f64>,
    /// Top-k sampling parameter.
    pub top_k: Option<u32>,
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u32>,
}
