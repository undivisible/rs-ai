//! JNI bridge for Android's Gemini Nano Prompt API.
//!
//! This module is only compiled on Android targets. It uses the `jni` crate
//! to call into Kotlin/Java code that wraps the Android Prompt API.

#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
use jni::objects::{JObject, JString};
#[cfg(target_os = "android")]
use jni::signature::JavaType;
#[cfg(target_os = "android")]
use jni::strings::JavaStr;
#[cfg(target_os = "android")]
use jni::sys::{jboolean, jint, jobject};
#[cfg(target_os = "android")]
use jni::{JNIEnv, JavaVM};

#[cfg(target_os = "android")]
use crate::types::{ModelDownloadState, NanoCapabilities, NanoSessionConfig};

/// A bridge to Android's Gemini Nano via JNI.
///
/// This struct holds a reference to the JVM and a Kotlin `GeminiNanoBridge`
/// instance that wraps the Android Prompt API.
#[cfg(target_os = "android")]
pub struct JniGeminiNanoBridge {
    vm: Arc<JavaVM>,
    bridge: GlobalRef,
}

#[cfg(target_os = "android")]
use jni::objects::GlobalRef;

#[cfg(target_os = "android")]
impl JniGeminiNanoBridge {
    /// Creates a new JNI bridge from a JVM and a Kotlin bridge instance.
    ///
    /// The Kotlin class must implement the `GeminiNanoBridge` interface
    /// and provide the following methods:
    /// - `isAvailable(): Boolean`
    /// - `downloadState(): Int` (0=NotDownloaded, 1=Downloading, 2=Downloaded, 3=Failed)
    /// - `requestDownload(): Boolean`
    /// - `capabilities(): Map<String, Boolean>`
    /// - `generate(prompt: String, config: String): String`
    /// - `createSession(config: String): String`
    /// - `sendMessage(sessionId: String, message: String): String`
    /// - `closeSession(sessionId: String): Boolean`
    pub fn new(vm: Arc<JavaVM>, bridge: GlobalRef) -> Self {
        Self { vm, bridge }
    }

    fn env(&self) -> JNIEnv {
        self.vm.attach_current_thread().expect("Failed to attach JVM thread")
    }

    /// Check if Gemini Nano is available.
    pub fn is_available(&self) -> bool {
        let mut env = self.env();
        let result = env.call_method(
            &self.bridge,
            "isAvailable",
            "()Z",
            &[],
        );
        match result {
            Ok(val) => val.z().unwrap_or(false),
            Err(e) => {
                tracing::error!("JNI isAvailable failed: {e}");
                false
            }
        }
    }

    /// Get the current download state.
    pub fn download_state(&self) -> ModelDownloadState {
        let mut env = self.env();
        let result = env.call_method(
            &self.bridge,
            "downloadState",
            "()I",
            &[],
        );
        match result {
            Ok(val) => {
                let state = val.i().unwrap_or(0);
                match state {
                    0 => ModelDownloadState::NotDownloaded,
                    1 => ModelDownloadState::Downloading { progress_percent: 0 },
                    2 => ModelDownloadState::Downloaded,
                    3 => ModelDownloadState::Failed { reason: "Unknown".into() },
                    _ => ModelDownloadState::NotDownloaded,
                }
            }
            Err(e) => {
                tracing::error!("JNI downloadState failed: {e}");
                ModelDownloadState::NotDownloaded
            }
        }
    }

    /// Request model download.
    pub fn request_download(&self) -> Result<(), String> {
        let mut env = self.env();
        let result = env.call_method(
            &self.bridge,
            "requestDownload",
            "()Z",
            &[],
        );
        match result {
            Ok(val) => {
                if val.z().unwrap_or(false) {
                    Ok(())
                } else {
                    Err("Download request failed".into())
                }
            }
            Err(e) => Err(format!("JNI requestDownload failed: {e}")),
        }
    }

    /// Get device capabilities.
    pub fn capabilities(&self) -> NanoCapabilities {
        let mut env = self.env();
        let result = env.call_method(
            &self.bridge,
            "capabilities",
            "()Ljava/util/Map;",
            &[],
        );
        match result {
            Ok(val) => {
                let map = val.l().unwrap_or(JObject::null());
                NanoCapabilities {
                    text_generation: get_bool_from_map(&mut env, &map, "textGeneration"),
                    summarization: get_bool_from_map(&mut env, &map, "summarization"),
                    rewriting: get_bool_from_map(&mut env, &map, "rewriting"),
                }
            }
            Err(e) => {
                tracing::error!("JNI capabilities failed: {e}");
                NanoCapabilities {
                    text_generation: false,
                    summarization: false,
                    rewriting: false,
                }
            }
        }
    }

    /// Generate text from a prompt.
    pub fn generate(&self, prompt: &str, config: &NanoSessionConfig) -> Result<String, String> {
        let mut env = self.env();
        let config_json = serde_json::to_string(config).map_err(|e| e.to_string())?;
        let prompt_jstring = env.new_string(prompt).map_err(|e| e.to_string())?;
        let config_jstring = env.new_string(&config_json).map_err(|e| e.to_string())?;

        let result = env.call_method(
            &self.bridge,
            "generate",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[
                (&prompt_jstring).into(),
                (&config_jstring).into(),
            ],
        );

        match result {
            Ok(val) => {
                let jstring = val.l().map_err(|e| e.to_string())?;
                if jstring.is_null() {
                    return Err("Generation returned null".into());
                }
                let java_str = env.get_string(&JString::from(jstring)).map_err(|e| e.to_string())?;
                Ok(java_str.to_string_lossy().into_owned())
            }
            Err(e) => Err(format!("JNI generate failed: {e}")),
        }
    }

    /// Create a new session.
    pub fn create_session(&self, config: &NanoSessionConfig) -> Result<String, String> {
        let mut env = self.env();
        let config_json = serde_json::to_string(config).map_err(|e| e.to_string())?;
        let config_jstring = env.new_string(&config_json).map_err(|e| e.to_string())?;

        let result = env.call_method(
            &self.bridge,
            "createSession",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[(&config_jstring).into()],
        );

        match result {
            Ok(val) => {
                let jstring = val.l().map_err(|e| e.to_string())?;
                if jstring.is_null() {
                    return Err("Session creation returned null".into());
                }
                let java_str = env.get_string(&JString::from(jstring)).map_err(|e| e.to_string())?;
                Ok(java_str.to_string_lossy().into_owned())
            }
            Err(e) => Err(format!("JNI createSession failed: {e}")),
        }
    }

    /// Send a message in a session.
    pub fn send_message(&self, session_id: &str, message: &str) -> Result<String, String> {
        let mut env = self.env();
        let session_jstring = env.new_string(session_id).map_err(|e| e.to_string())?;
        let message_jstring = env.new_string(message).map_err(|e| e.to_string())?;

        let result = env.call_method(
            &self.bridge,
            "sendMessage",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            &[
                (&session_jstring).into(),
                (&message_jstring).into(),
            ],
        );

        match result {
            Ok(val) => {
                let jstring = val.l().map_err(|e| e.to_string())?;
                if jstring.is_null() {
                    return Err("Send message returned null".into());
                }
                let java_str = env.get_string(&JString::from(jstring)).map_err(|e| e.to_string())?;
                Ok(java_str.to_string_lossy().into_owned())
            }
            Err(e) => Err(format!("JNI sendMessage failed: {e}")),
        }
    }

    /// Close a session.
    pub fn close_session(&self, session_id: &str) -> Result<(), String> {
        let mut env = self.env();
        let session_jstring = env.new_string(session_id).map_err(|e| e.to_string())?;

        let result = env.call_method(
            &self.bridge,
            "closeSession",
            "(Ljava/lang/String;)Z",
            &[(&session_jstring).into()],
        );

        match result {
            Ok(val) => {
                if val.z().unwrap_or(false) {
                    Ok(())
                } else {
                    Err("Close session failed".into())
                }
            }
            Err(e) => Err(format!("JNI closeSession failed: {e}")),
        }
    }
}

#[cfg(target_os = "android")]
fn get_bool_from_map(env: &mut JNIEnv, map: &JObject, key: &str) -> bool {
    if map.is_null() {
        return false;
    }
    let key_jstring = match env.new_string(key) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let result = env.call_method(
        map,
        "get",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        &[(&key_jstring).into()],
    );
    match result {
        Ok(val) => {
            let obj = val.l().unwrap_or(JObject::null());
            if obj.is_null() {
                return false;
            }
            let bool_result = env.call_method(
                &obj,
                "booleanValue",
                "()Z",
                &[],
            );
            bool_result.ok().and_then(|v| v.z()).unwrap_or(false)
        }
        Err(_) => false,
    }
}
