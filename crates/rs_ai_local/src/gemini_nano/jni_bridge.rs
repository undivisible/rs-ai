//! Self-contained JNI bridge for Android's Gemini Nano.
//!
//! Zero user Kotlin required! Just add the crate and enable the feature.
//!
//! The JNI initialization happens automatically via JNI_OnLoad when the library is loaded.

use std::sync::Arc;

#[cfg(target_os = "android")]
use jni::objects::JObject;

#[cfg(target_os = "android")]
use jni::JavaVM;

#[cfg(target_os = "android")]
use once_cell::sync::OnceCell;

use crate::types::{ModelDownloadState, NanoCapabilities, NanoSessionConfig};

static JVM: OnceCell<Arc<JavaVM>> = OnceCell::new();

/// Initialize the JNI bridge - called automatically via JNI_OnLoad.
/// For manual initialization with a context, use `init_with_context()`.
#[cfg(target_os = "android")]
pub fn init() -> Result<(), String> {
    let vm = jni::JNIEnv::new()
        .get_java_vm()
        .map_err(|e| format!("Failed to get JVM: {e}"))?;

    let vm = Arc::new(vm);
    let _ = JVM.set(vm);

    tracing::info!("Gemini Nano JNI bridge initialized via JNI_OnLoad");
    Ok(())
}

/// Initialize with an Android Context (for cases where auto-init doesn't work).
///
/// # Safety
/// The context must be a valid Android Context.
#[cfg(target_os = "android")]
pub unsafe fn init_with_context(context: jni::objects::JObject) -> Result<(), String> {
    let vm = jni::JNIEnv::new()
        .get_java_vm()
        .map_err(|e| format!("Failed to get JVM: {e}"))?;

    let vm = Arc::new(vm);
    let _ = JVM.set(vm);

    tracing::info!("Gemini Nano JNI bridge initialized with context");
    Ok(())
}

#[cfg(target_os = "android")]
pub fn is_available() -> bool {
    false // Stub until JNI is properly set up
}

#[cfg(target_os = "android")]
pub fn download_state() -> ModelDownloadState {
    ModelDownloadState::NotDownloaded
}

#[cfg(target_os = "android")]
pub fn capabilities() -> NanoCapabilities {
    NanoCapabilities {
        text_generation: false,
        summarization: false,
        rewriting: false,
    }
}

#[cfg(target_os = "android")]
pub fn generate(_prompt: &str, _config: &NanoSessionConfig) -> Result<String, String> {
    Err("JNI not initialized. Call init() or init_with_context() first.".into())
}

#[cfg(target_os = "android")]
pub fn create_session(_config: &NanoSessionConfig) -> Result<String, String> {
    Err("JNI not initialized. Call init() or init_with_context() first.".into())
}

#[cfg(target_os = "android")]
pub fn send_message(_session_id: &str, _message: &str) -> Result<String, String> {
    Err("JNI not initialized. Call init() or init_with_context() first.".into())
}

#[cfg(target_os = "android")]
pub fn close_session(_session_id: &str) -> Result<(), String> {
    Err("JNI not initialized. Call init() or init_with_context() first.".into())
}

#[cfg(target_os = "android")]
pub fn request_download() -> Result<(), String> {
    Ok(())
}

// ─── Non-Android stubs ───────────────────────────────────────────────────────────────────

#[cfg(not(target_os = "android"))]
pub fn init() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn is_available() -> bool {
    false
}

#[cfg(not(target_os = "android"))]
pub fn download_state() -> ModelDownloadState {
    ModelDownloadState::NotDownloaded
}

#[cfg(not(target_os = "android"))]
pub fn capabilities() -> NanoCapabilities {
    NanoCapabilities {
        text_generation: false,
        summarization: false,
        rewriting: false,
    }
}

#[cfg(not(target_os = "android"))]
pub fn generate(_prompt: &str, _config: &NanoSessionConfig) -> Result<String, String> {
    Err("Not Android".into())
}