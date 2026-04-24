use rs_ai_ai::*;
use rs_ai_chatgpt::ChatGptProvider;
use rs_ai_claude::ClaudeProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using ChatGPT
    let chatgpt =
        ChatGptProvider::new(std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"));
    let model = chatgpt.gpt4o_mini();

    let result = generate_text(&model, "What is Rust programming language?").await?;
    println!("ChatGPT says: {result}");

    // Example 2: Using Claude
    let claude = ClaudeProvider::new(
        std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required"),
    );
    let model = claude.claude_sonnet();

    let result = generate_text(&model, "What is Rust programming language?").await?;
    println!("Claude says: {result}");

    Ok(())
}
