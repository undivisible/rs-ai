//! Example: Windows Phi Silica / WinML integration.
//!
//! This example demonstrates how to use the Phi Silica provider with
//! both the built-in mock bridge and the Windows ML model loader.

use rs_ai_phi_silica::*;
use rs_ai_traits::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using the built-in mock bridge
    println!("=== Mock Bridge ===");
    let provider = PhiSilicaProvider::new(MockPhiSilicaBridge);
    let model = provider.model();

    println!("Phi Silica available: {:?}", provider.availability().await);

    let result = generate_text(&model, "What is the Windows Copilot Runtime?").await?;
    println!("Response: {result}");

    // Example 2: Using Windows ML (on Windows only)
    #[cfg(windows)]
    {
        println!("\n=== Windows ML ===");
        println!("WinML availability: {:?}", winml_available());

        // In a real app, you would provide an ONNX model file:
        // let winml = WinMlProvider::new("model.onnx");
        // let model = winml.model()?;
    }

    Ok(())
}

/// Mock bridge for demonstration purposes.
pub struct MockPhiSilicaBridge;

#[async_trait::async_trait]
impl PhiSilicaBridge for MockPhiSilicaBridge {
    async fn availability(&self) -> PhiSilicaAvailability {
        PhiSilicaAvailability::Available
    }

    async fn generate(&self, prompt: &str, _max_tokens: Option<u32>) -> Result<String, String> {
        Ok(format!("[Phi Silica mock response to: {prompt}]"))
    }

    async fn stream_tokens(
        &self,
        prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<Vec<String>, String> {
        let text = self.generate(prompt, _max_tokens).await?;
        Ok(vec![text])
    }
}
