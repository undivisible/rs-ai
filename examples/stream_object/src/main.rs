use futures::StreamExt;
use rs_ai_chatgpt::ChatGptProvider;
use rs_ai_traits::*;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct MovieReview {
    title: String,
    rating: f32,
    summary: String,
    pros: Vec<String>,
    cons: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ChatGptProvider::new(std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"));
    let model = provider.gpt4o();

    let options = GenerateOptions {
        output_schema: Some(OutputSchema::from_type::<MovieReview>()),
        ..Default::default()
    };

    let mut stream = model
        .stream(
            Prompt::from("Write a review for the movie 'Inception'"),
            options,
        )
        .await?;

    println!("Streaming object deltas:");
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => print!("{delta}"),
            StreamEvent::ObjectDelta { delta } => {
                println!("Object delta: {delta}");
            }
            StreamEvent::MessageEnd { .. } => println!("\n\nDone!"),
            _ => {}
        }
    }

    Ok(())
}
