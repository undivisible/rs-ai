use secrecy::SecretString;

/// Configuration for connecting to an OpenAI-compatible API.
#[derive(Clone)]
pub struct OpenAiCompatibleConfig {
    pub(crate) base_url: String,
    pub(crate) api_key: SecretString,
    pub(crate) org_id: Option<String>,
    pub(crate) default_headers: Vec<(String, String)>,
}

impl OpenAiCompatibleConfig {
    /// Create a new configuration with a custom base URL.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: SecretString::from(api_key.into()),
            org_id: None,
            default_headers: Vec::new(),
        }
    }

    /// Create a configuration pre-pointed at the official OpenAI API.
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new("https://api.openai.com/v1", api_key)
    }

    /// Set an optional organization ID header.
    pub fn with_org(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Replace the API key (e.g. to override with an OAuth token).
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = SecretString::from(api_key.into());
        self
    }

    /// Add a default header that will be sent with every request.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// The base URL (e.g. `https://api.openai.com/v1`).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Expose the API key for building HTTP headers.
    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }
}

impl std::fmt::Debug for OpenAiCompatibleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiCompatibleConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("org_id", &self.org_id)
            .finish()
    }
}
