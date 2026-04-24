use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A capability that a model may support.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    TextInput,
    TextOutput,
    ImageInput,
    ImageOutput,
    Streaming,
    ToolCalling,
    StructuredOutput,
    Embeddings,
    LocalExecution,
    SessionSupport,
    PlatformNative,
    ExtendedThinking,
    VideoInput,
    AudioInput,
    AudioOutput,
}

/// An ordered set of capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    inner: BTreeSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self {
            inner: BTreeSet::new(),
        }
    }

    /// Builder-style: add a capability and return self.
    pub fn with(mut self, cap: Capability) -> Self {
        self.inner.insert(cap);
        self
    }

    /// Check whether the set contains a specific capability.
    pub fn has(&self, cap: &Capability) -> bool {
        self.inner.contains(cap)
    }

    /// Returns `true` if every capability in the slice is present.
    pub fn supports_all(&self, caps: &[Capability]) -> bool {
        caps.iter().all(|c| self.inner.contains(c))
    }

    /// Merge another set into this one.
    pub fn merge(&mut self, other: &CapabilitySet) {
        for cap in &other.inner {
            self.inner.insert(cap.clone());
        }
    }

    /// Iterate over the capabilities.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.inner.iter()
    }
}
