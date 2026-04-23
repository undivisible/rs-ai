//! Streaming tool use with Claude — real-time function calling.
//!
//! Tool calls stream incrementally; each argument delta arrives as the model
//! generates it, allowing immediate execution without waiting for the full response.
//!
//! Run: `ANTHROPIC_API_KEY=... cargo run --example streaming_tool_use`

use futures::StreamExt;
use rai_ai::{GenerateOptions, LanguageModel, Prompt, StreamEvent, ToolChoice, ToolDefinition};
use rai_claude::ClaudeProvider;
use schemars::schema_for;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct WeatherInput {
    location: String,
    unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct SearchInput {
    query: String,
    num_results: Option<u32>,
}

async fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "get_weather" => {
            if let Ok(input) = serde_json::from_value::<WeatherInput>(args.clone()) {
                format!(
                    "Weather in {}: 72°F, Sunny (unit: {})",
                    input.location,
                    input.unit.as_deref().unwrap_or("F")
                )
            } else {
                "Error: invalid weather arguments".to_string()
            }
        }
        "search_web" => {
            if let Ok(input) = serde_json::from_value::<SearchInput>(args.clone()) {
                format!(
                    "Search results for '{}' (limit: {}):\n1. Example result 1\n2. Example result 2",
                    input.query,
                    input.num_results.unwrap_or(5)
                )
            } else {
                "Error: invalid search arguments".to_string()
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let provider = ClaudeProvider::new(api_key);
    let model = provider.claude_sonnet();

    let tools = vec![
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get the current weather for a location.".into(),
            parameters: serde_json::to_value(schema_for!(WeatherInput))?,
        },
        ToolDefinition {
            name: "search_web".into(),
            description: "Search the web for information.".into(),
            parameters: serde_json::to_value(schema_for!(SearchInput))?,
        },
    ];

    let options = GenerateOptions::default()
        .with_tools(tools)
        .with_tool_choice(ToolChoice::Auto);

    let prompt = Prompt::Text(
        "What's the weather in San Francisco? Also search for the latest AI news.".into(),
    );

    println!("🚀 Streaming Tool Use — real-time function calling\n");

    let mut stream = model.stream(prompt, options).await?;

    let mut pending: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => {
                print!("{}", delta);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            StreamEvent::ToolCallStart { call_id, tool_name } => {
                println!("\n🔧 Tool call starting: {} [{}]", tool_name, call_id);
                pending.insert(call_id, tool_name);
            }
            StreamEvent::ToolCallDelta { call_id: _, delta } => {
                print!("  {}", delta);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            StreamEvent::ToolCallEnd { call_id, arguments } => {
                if let Some(name) = pending.remove(&call_id) {
                    println!("\n  args: {}", serde_json::to_string_pretty(&arguments)?);
                    let result = execute_tool(&name, &arguments).await;
                    println!("  ✅ result: {}", result);
                }
            }
            StreamEvent::MessageEnd { finish_reason, usage } => {
                println!("\n\n✨ Done — finish_reason: {:?}", finish_reason);
                if let Some(u) = usage {
                    println!(
                        "   tokens: {} prompt / {} completion",
                        u.prompt_tokens.unwrap_or(0),
                        u.completion_tokens.unwrap_or(0)
                    );
                }
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
