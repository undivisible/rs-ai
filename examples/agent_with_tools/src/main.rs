//! Agent with tool streaming — demonstrates real-time tool use via streaming events.
//!
//! Run: `ANTHROPIC_API_KEY=... cargo run --example agent_with_tools`

use futures::StreamExt;
use rs_ai_claude::ClaudeProvider;
use rs_ai_traits::{
    GenerateOptions, LanguageModel, Prompt, StreamEvent, ToolChoice, ToolDefinition,
};
use schemars::schema_for;
use serde::{Deserialize, Serialize};

// Define tool request/response schemas
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct WeatherInput {
    location: String,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct SearchInput {
    query: String,
}

fn get_weather_tool() -> ToolDefinition {
    ToolDefinition {
        name: "get_weather".into(),
        description: "Get the current weather for a location. Returns temperature in Celsius."
            .into(),
        parameters: serde_json::to_value(schema_for!(WeatherInput)).unwrap(),
    }
}

fn search_tool() -> ToolDefinition {
    ToolDefinition {
        name: "search".into(),
        description: "Search for information about a topic.".into(),
        parameters: serde_json::to_value(schema_for!(SearchInput)).unwrap(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let provider = ClaudeProvider::new(api_key);
    let model = provider.claude_sonnet();

    println!("🤖 Agent initialized with tools: get_weather, search");
    println!("📨 Streaming response with tool calls...\n");

    // Define tools
    let tools = vec![get_weather_tool(), search_tool()];

    // Request with tools and streaming
    let options = GenerateOptions::default()
        .with_tools(tools)
        .with_tool_choice(ToolChoice::Auto);

    let prompt = "What's the weather in Paris and what are the top sights to visit? Please use tools to get this information.";

    // Stream the response
    let mut stream = model
        .stream(Prompt::Text(prompt.to_string()), options)
        .await?;

    let mut text_response = String::new();
    let mut tool_calls_seen = 0;

    println!("📥 Streaming response:\n");

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::TextDelta { delta } => {
                print!("{}", delta);
                text_response.push_str(&delta);
            }
            StreamEvent::ToolCallStart {
                call_id: id,
                tool_name,
            } => {
                tool_calls_seen += 1;
                println!("\n🔧 Tool call [{}]: {}", id, tool_name);
            }
            StreamEvent::ToolCallDelta { call_id, delta } => {
                print!("  📝 [{}] {}", call_id, delta);
            }
            StreamEvent::ToolCallEnd {
                call_id: _,
                arguments,
            } => {
                let tool_name = arguments
                    .get("location")
                    .is_some()
                    .then_some("get_weather")
                    .or_else(|| arguments.get("query").map(|_| "search"))
                    .unwrap_or("unknown");

                let result = execute_tool(tool_name, &arguments);
                println!("\n  ✅ Executed: {} with result: {}", tool_name, result);
            }
            StreamEvent::MessageEnd {
                finish_reason,
                usage: _,
            } => {
                println!(
                    "\n\n✅ Message generation complete (finish_reason: {:?})",
                    finish_reason
                );
                break;
            }
            _ => {}
        }
    }

    println!("\n📊 Summary:");
    println!("  Tool calls made: {}", tool_calls_seen);
    println!("  Text response length: {} chars", text_response.len());
    Ok(())
}

fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "get_weather" => {
            if let Some(location) = args.get("location").and_then(|v| v.as_str()) {
                format!(
                    "Current weather in {}: 15°C, partly cloudy, light breeze",
                    location
                )
            } else {
                "Error: invalid location".to_string()
            }
        }
        "search" => {
            if let Some(query) = args.get("query").and_then(|v| v.as_str()) {
                format!("Search results for '{}': 3 relevant articles found", query)
            } else {
                "Error: invalid query".to_string()
            }
        }
        _ => "Unknown tool".to_string(),
    }
}
