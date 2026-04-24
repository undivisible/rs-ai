use rs_ai_ai::*;
use rs_ai_testing::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create mock models
    let local_model = MockLanguageModel::new("local-llm").with_text("Hello from the local model!");

    let cloud_model = MockLanguageModel::new("cloud-llm").with_text("Hello from the cloud model!");

    // Create a local-first router: tries the local model first, falls back to cloud
    let router = Router::local_first(Box::new(local_model), Box::new(cloud_model));

    // The router will try the local model first
    let result = generate_text(&router, "Hello!").await?;
    println!("Router result: {result}");

    // Create a capability-based router
    let text_model = MockLanguageModel::new("text-model").with_text("I handle text!");

    let tool_model = MockLanguageModel::new("tool-model").with_text("I handle tools!");

    let router = Router::new()
        .add_route(Box::new(tool_model), |_prompt, options| {
            options.tools.is_some() && !options.tools.as_ref().unwrap().is_empty()
        })
        .with_fallback(Box::new(text_model));

    // Simple text request -- no tools, goes to fallback
    let result = generate_text(&router, "Hello!").await?;
    println!("Simple text: {result}");

    Ok(())
}
