//! Native C# bridge for Phi Silica via FFI.
//!
//! This module is only compiled on Windows when the C# bridge was successfully
//! built by `build.rs`. It calls into the `PhiSilicaNative.dll` compiled from C#.

use async_trait::async_trait;

use super::types::PhiSilicaAvailability;
use crate::PhiSilicaBridge;

#[cfg(phi_silica_bridge)]
use std::ffi::{c_char, c_int, CStr, CString};

#[cfg(phi_silica_bridge)]
unsafe extern "C" {
    fn phi_silica_availability() -> c_int;
    fn phi_silica_generate(prompt: *const c_char, max_tokens: c_int) -> *mut c_char;
    fn phi_silica_free_string(ptr: *mut c_char);
}

/// A bridge that calls Phi Silica through the C# Native AOT library.
///
/// This is only available on Windows when the C# bridge was successfully compiled.
/// Use [`super::MockPhiSilicaBridge`] for testing on other platforms.
pub struct NativePhiSilicaBridge;

#[async_trait]
impl PhiSilicaBridge for NativePhiSilicaBridge {
    async fn availability(&self) -> PhiSilicaAvailability {
        #[cfg(phi_silica_bridge)]
        {
            let code = unsafe { phi_silica_availability() };
            match code {
                0 => PhiSilicaAvailability::Available,
                2 => PhiSilicaAvailability::WindowsVersionTooOld,
                3 => PhiSilicaAvailability::NpuNotDetected,
                _ => PhiSilicaAvailability::Unavailable,
            }
        }
        #[cfg(not(phi_silica_bridge))]
        {
            PhiSilicaAvailability::Unavailable
        }
    }

    async fn generate(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String, String> {
        #[cfg(phi_silica_bridge)]
        {
            let c_prompt = CString::new(prompt).map_err(|e| e.to_string())?;
            let max_tok = max_tokens.unwrap_or(0) as c_int;

            let result_ptr = unsafe { phi_silica_generate(c_prompt.as_ptr(), max_tok) };
            if result_ptr.is_null() {
                return Err("Phi Silica returned null".into());
            }

            let result = unsafe {
                let c_str = CStr::from_ptr(result_ptr);
                let s = c_str.to_string_lossy().into_owned();
                phi_silica_free_string(result_ptr);
                s
            };

            if result.starts_with("[error:") {
                Err(result)
            } else {
                Ok(result)
            }
        }
        #[cfg(not(phi_silica_bridge))]
        {
            let _ = (prompt, max_tokens);
            Err(
                "Phi Silica C# bridge not available. Use a custom PhiSilicaBridge implementation."
                    .into(),
            )
        }
    }
}

/// Check if the native C# bridge is available.
pub fn native_bridge_available() -> bool {
    cfg!(phi_silica_bridge)
}
