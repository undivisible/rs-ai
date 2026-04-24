//! Example: Gemini Nano on Android.
//!
//! This example demonstrates how to use the Gemini Nano provider with the
//! built-in mock bridge. In a real Android app, you would use `JniGeminiNanoBridge`
//! to call into Kotlin.

use rs_ai_gemini_nano::*;
use rs_ai_traits::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the built-in mock bridge for testing
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
