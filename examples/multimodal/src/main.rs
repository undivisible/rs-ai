use rusty_ai::*;
use rusty_chatgpt::ChatGptProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ChatGptProvider::new(std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"));
    let model = provider.gpt4o();

    // Create a message with an image URL
    let message = Message::user("Describe this image in detail.").with_image(ImageData::Url {
        url: "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/\
              Rust_programming_language_black_logo.svg/\
              1200px-Rust_programming_language_black_logo.svg.png"
            .into(),
        detail: Some(ImageDetail::Auto),
    });

    let result = model
        .generate(Prompt::Messages(vec![message]), GenerateOptions::default())
        .await?;

    if let Some(text) = &result.text {
        println!("Description: {text}");
    }

    Ok(())
}
