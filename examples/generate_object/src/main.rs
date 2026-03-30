use rusty_ai::model::generate_object;
use rusty_ai::*;
use rusty_chatgpt::ChatGptProvider;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    steps: Vec<String>,
    prep_time_minutes: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ChatGptProvider::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"),
    );
    let model = provider.gpt4o_mini();

    let result: ObjectResult<Recipe> = generate_object(
        &model,
        Prompt::from("Generate a recipe for chocolate chip cookies"),
        GenerateOptions::default(),
    )
    .await?;

    println!("Recipe: {}", result.object.name);
    println!("Prep time: {} minutes", result.object.prep_time_minutes);
    println!("\nIngredients:");
    for ingredient in &result.object.ingredients {
        println!("  - {ingredient}");
    }
    println!("\nSteps:");
    for (i, step) in result.object.steps.iter().enumerate() {
        println!("  {}. {step}", i + 1);
    }

    Ok(())
}
