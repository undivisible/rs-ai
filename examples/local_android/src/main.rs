//! Example: Gemini Nano on Android.
//!
//! This example demonstrates how to use the Gemini Nano provider with the
//! built-in mock bridge. In a real Android app, you would use `JniGeminiNanoBridge`
//! to call into Kotlin.

use rs_ai_core::*;
use rs_ai_local::gemini_nano::*;

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

    // Multi-turn: use generate_content for each turn
    let reply1 = provider
        .model()
        .generate(
            rs_ai_core::Prompt::Text("What is Rust?".into()),
            rs_ai_core::GenerateOptions::default(),
        )
        .await?;
    println!("Reply 1: {}", reply1.text.unwrap_or_default());

    let reply2 = provider
        .model()
        .generate(
            rs_ai_core::Prompt::Text("What about its memory safety?".into()),
            rs_ai_core::GenerateOptions::default(),
        )
        .await?;
    println!("Reply 2: {}", reply2.text.unwrap_or_default());

    Ok(())
}
