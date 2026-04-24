use async_trait::async_trait;
use rs_ai_ai::*;
use rs_ai_gemini_nano::*;

/// Example bridge implementation (in a real app, this would call JNI)
struct MockNanoBridge;

#[async_trait]
impl GeminiNanoBridge for MockNanoBridge {
    async fn is_available(&self) -> bool {
        true
    }

    async fn download_state(&self) -> ModelDownloadState {
        ModelDownloadState::Downloaded
    }

    async fn request_download(&self) -> Result<(), String> {
        Ok(())
    }

    async fn capabilities(&self) -> NanoCapabilities {
        NanoCapabilities {
            text_generation: true,
            summarization: true,
            rewriting: true,
        }
    }

    async fn generate(&self, prompt: &str, _config: &NanoSessionConfig) -> Result<String, String> {
        Ok(format!("[Gemini Nano mock response to: {prompt}]"))
    }

    async fn create_session(&self, _config: &NanoSessionConfig) -> Result<String, String> {
        Ok("mock-session-1".into())
    }

    async fn send_message(&self, _session_id: &str, message: &str) -> Result<String, String> {
        Ok(format!("[Session reply to: {message}]"))
    }

    async fn close_session(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GeminiNanoProvider::new(MockNanoBridge);

    // Check availability
    println!("Gemini Nano available: {}", provider.is_available().await);
    println!("Download state: {:?}", provider.download_state().await);

    // Single-turn generation
    let model = provider.model();
    let result = generate_text(&model, "Summarize the Rust programming language").await?;
    println!("Response: {result}");

    // Multi-turn session
    let session = provider
        .create_session(NanoSessionConfig::default())
        .await?;
    let reply1 = session.send("What is Rust?").await?;
    println!("Session reply 1: {reply1}");
    let reply2 = session.send("What about its memory safety?").await?;
    println!("Session reply 2: {reply2}");

    Ok(())
}
