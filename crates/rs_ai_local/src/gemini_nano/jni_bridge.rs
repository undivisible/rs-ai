//! JNI bridge for Android's Gemini Nano via the ML Kit GenAI Prompt API.
//!
//! Calls `GenAI.getClient()` for availability checks and a static Kotlin
//! helper (`com.rs_ai.GeminiNanoBridge`) for generate content.

use std::sync::Arc;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jint;
use jni::JavaVM;
use once_cell::sync::OnceCell;

static JVM: OnceCell<Arc<JavaVM>> = OnceCell::new();

/// JNI bridge for Gemini Nano on Android.
pub struct JniGeminiNanoBridge {
    jvm: Arc<JavaVM>,
}

impl JniGeminiNanoBridge {
    /// Create a new bridge with the given Java VM reference.
    pub fn new(jvm: Arc<JavaVM>) -> Self {
        Self { jvm }
    }

    /// Check if Gemini Nano is available on this device.
    ///
    /// Calls `GenAI.getClient().isAvailable()` via JNI.
    pub fn is_available(&self) -> bool {
        let Ok(mut env) = self.jvm.attach_current_thread() else {
            return false;
        };

        let genai_class = match env.find_class("com/google/mlkit/genai/prompt/GenAI") {
            Ok(c) => c,
            Err(_) => return false,
        };

        let result: Result<bool, jni::errors::Error> = (|| {
            let client = env
                .call_static_method(
                    &genai_class,
                    "getClient",
                    "()Lcom/google/mlkit/genai/prompt/GenAI;",
                    &[],
                )?
                .l()?;

            env.call_method(&client, "isAvailable", "()Z", &[])?.z()
        })();

        result.unwrap_or(false)
    }

    /// Generate content via the Kotlin static helper.
    ///
    /// Serializes the request to JSON, calls
    /// `com.rs_ai.GeminiNanoBridge.generate(json)`, and returns the
    /// JSON result string.
    pub fn generate_content_json(&self, request_json: &str) -> Result<String, String> {
        let mut env = self
            .jvm
            .attach_current_thread()
            .map_err(|e| format!("JNI attach failed: {e}"))?;

        let bridge_class = env
            .find_class("com/rs_ai/GeminiNanoBridge")
            .map_err(|e| format!("Failed to find com.rs_ai.GeminiNanoBridge: {e}"))?;

        let j_request = env
            .new_string(request_json)
            .map_err(|e| format!("Failed to create JNI string: {e}"))?;

        let j_result = env
            .call_static_method(
                &bridge_class,
                "generate",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&j_request.into())],
            )
            .map_err(|e| format!("JNI call to GeminiNanoBridge.generate() failed: {e}"))?;

        let j_result_obj = j_result
            .l()
            .map_err(|e| format!("Failed to extract result object: {e}"))?;

        let result_str: String = env
            .get_string(&JString::from(j_result_obj))
            .map_err(|e| format!("Failed to read result string: {e}"))?
            .into();

        Ok(result_str)
    }
}

// ─── Auto-init via JNI_OnLoad ─────────────────────────────────────────────

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: jni::JavaVM, _reserved: *mut std::ffi::c_void) -> jint {
    let vm = Arc::new(vm);
    JVM.set(vm).ok();
    tracing::info!("Gemini Nano JNI bridge initialized via JNI_OnLoad");
    jni::sys::JNI_VERSION_1_6
}

/// Initialize the JNI bridge (called automatically via JNI_OnLoad).
pub fn init() -> Result<(), String> {
    let vm = JVM
        .get()
        .ok_or_else(|| "JVM not initialized. JNI_OnLoad may not have been called.".to_string())?;
    let _ = vm.clone();
    tracing::info!("Gemini Nano JNI bridge ready");
    Ok(())
}

/// Initialize with an Android Context (alias for init when auto-init works).
///
/// # Safety
/// The context reference must be a valid Android Context object.
#[allow(unused_variables)]
pub unsafe fn init_with_context(context: jni::objects::JObject) -> Result<(), String> {
    init()
}
