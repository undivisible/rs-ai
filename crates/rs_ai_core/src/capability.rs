//! Model capabilities.
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A capability that a model may support.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Accept text input.
    TextInput,
    /// Produce text output.
    TextOutput,
    /// Accept image input.
    ImageInput,
    /// Produce image output.
    ImageOutput,
    /// Support streaming responses.
    Streaming,
    /// Support tool calling.
    ToolCalling,
    /// Support structured output.
    StructuredOutput,
    /// Support text embeddings.
    Embeddings,
    /// Run locally on the device.
    LocalExecution,
    /// Support persistent sessions.
    SessionSupport,
    /// Platform-native execution.
    PlatformNative,
    /// Support extended thinking / reasoning.
    ExtendedThinking,
    /// Accept video input.
    VideoInput,
    /// Accept audio input.
    AudioInput,
    /// Produce audio output.
    AudioOutput,
    /// Generate images from text prompts.
    ImageGeneration,
    /// Generate videos from text prompts.
    VideoGeneration,
    /// Realtime voice/audio conversation.
    RealtimeVoice,
    /// Realtime video conversation.
    RealtimeVideo,
}

/// An ordered set of capabilities.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Underlying set of capabilities.
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

    /// Remove a capability from the set.
    pub fn remove(&mut self, cap: &Capability) -> bool {
        self.inner.remove(cap)
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
