use futures::StreamExt;
use rs_ai_chatgpt::ChatGptProvider;
use rs_ai_traits::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ChatGptProvider::new(std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"));
    let model = provider.gpt4o_mini();

    let mut stream = stream_text(&model, "Write a haiku about Rust programming").await?;

    print!("Streaming: ");
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => print!("{delta}"),
            StreamEvent::MessageEnd {
                finish_reason,
                usage,
            } => {
                println!("\n\nFinished: {finish_reason:?}");
                if let Some(usage) = usage {
                    println!("Tokens used: {:?}", usage.total_tokens);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
