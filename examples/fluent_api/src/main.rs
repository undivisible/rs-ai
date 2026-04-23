//! Demonstrates the fluent rs-ai API with simple one-line builder pattern.
//!
//! This example shows how to use rs-ai's ergonomic builder API for quick AI operations.

use rs_ai::{chatgpt, claude, compatible, gemini};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using Claude with fluent API
    println!("=== Claude Example ===");
    let claude_response = claude()
        .api_key("your-anthropic-api-key-here")
        .model("claude-sonnet-4-6")
        .generate("What is 2+2?")
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("Claude response: {}\n", claude_response);

    // Example 2: Using ChatGPT with fluent API
    println!("=== ChatGPT Example ===");
    let chatgpt_response = chatgpt()
        .api_key("your-openai-api-key-here")
        .model("gpt-4o-mini")
        .generate("What is the capital of France?")
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("ChatGPT response: {}\n", chatgpt_response);

    // Example 3: Using Gemini with fluent API
    println!("=== Gemini Example ===");
    let gemini_response = gemini()
        .api_key("your-google-api-key-here")
        .model("gemini-2.5-flash")
        .generate("List 3 interesting facts about penguins")
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("Gemini response: {}\n", gemini_response);

    // Example 4: Using OpenAI-compatible endpoint (OpenRouter)
    println!("=== OpenRouter (Compatible) Example ===");
    let compatible_response = compatible("https://api.openrouter.ai/api/v1")
        .api_key("your-openrouter-key-here")
        .model("meta-llama/llama-2-70b-chat")
        .generate("Explain quantum computing in one sentence")
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("OpenRouter response: {}\n", compatible_response);

    // Example 5: Method chaining in different orders
    println!("=== Method Chaining (Order Independent) ===");
    let flexible_response = claude()
        .model("claude-sonnet-4-6")  // Can set model first
        .api_key("your-anthropic-api-key-here")  // Then api_key
        .generate("Hello, world!")
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("Flexible response: {}\n", flexible_response);

    println!("✓ Fluent API examples complete!");
    Ok(())
}
