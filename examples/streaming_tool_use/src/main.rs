//! Example: Streaming Tool Use with Claude
//!
//! This example demonstrates real-time function calling where Claude streams tool calls
//! as they're generated, allowing immediate execution without waiting for the entire
//! model response.
//!
//! Run with:
//! ```bash
//! ANTHROPIC_API_KEY=sk-... cargo run --example streaming_tool_use
//! ```

use futures::StreamExt;
use rai_ai::*;
use rai_claude::ClaudeProvider;
use schemars::schema_for;
use serde::{Deserialize, Serialize};

/// Example tool: Get current weather
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct WeatherInput {
    location: String,
    unit: Option<String>,
}

/// Example tool: Search the web
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct SearchInput {
    query: String,
    num_results: Option<u32>,
}

/// Simulated tool execution - in production, call real APIs
async fn execute_tool(tool_name: &str, args: &serde_json::Value) -> Result<String, String> {
    match tool_name {
        "get_weather" => {
            let input: WeatherInput = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid weather args: {}", e))?;
            Ok(format!(
                "Weather in {}: 72°F, Sunny (unit: {})",
                input.location,
                input.unit.unwrap_or_else(|| "F".to_string())
            ))
        }
        "search_web" => {
            let input: SearchInput = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid search args: {}", e))?;
            Ok(format!(
                "Search results for '{}' (limit: {}):\n1. Example result 1\n2. Example result 2",
                input.query,
                input.num_results.unwrap_or(5)
            ))
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Claude provider
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let provider = ClaudeProvider::new(api_key);
    let model = provider.claude_sonnet();

    // Define tools that the model can use
    let tools = vec![
        ToolDefinition {
            name: "get_weather".to_string(),
            description: "Get the current weather for a location".to_string(),
            input_schema: schema_for!(WeatherInput),
        },
        ToolDefinition {
            name: "search_web".to_string(),
            description: "Search the web for information".to_string(),
            input_schema: schema_for!(SearchInput),
        },
    ];

    let tool_set = ToolSet::new(tools);

    // Create the request with tools
    let request = GenerateRequest {
        prompt: Prompt::Text(
            "What's the weather in San Francisco? Also search for the latest AI news.".to_string(),
        ),
        options: GenerateOptions::default(),
        tools: Some(tool_set),
        system: None,
        ..Default::default()
    };

    println!("🚀 Streaming Tool Use Example\n");
    println!("User: {}", request.prompt);
    println!("\n📡 Streaming response with tool calls:\n");

    // Stream the response
    let mut stream = model.stream(request).await?;

    let mut collected_response = GenerateResult::default();
    let mut pending_tools: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();

    while let Some(event) = stream.next().await {
        let event = event?;

        match event {
            StreamEvent::TextDelta { delta } => {
                print!("{}", delta);
                collected_response.text =
                    Some(collected_response.text.unwrap_or_default() + &delta);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }

            StreamEvent::ToolCallStart {
                call_id,
                tool_name,
            } => {
                println!("\n\n🔧 Tool Call Starting: {}", tool_name);
                pending_tools.insert(call_id.clone(), (tool_name.clone(), String::new()));
            }

            StreamEvent::ToolCallDelta { call_id, delta } => {
                if let Some((_name, args)) = pending_tools.get_mut(&call_id) {
                    args.push_str(&delta);
                    print!("  Arg delta: {}", delta);
                }
            }

            StreamEvent::ToolCallEnd {
                call_id,
                arguments,
            } => {
                if let Some((name, _partial)) = pending_tools.remove(&call_id) {
                    println!("\n✅ Tool Call Complete: {}", name);
                    println!("   Arguments: {}", serde_json::to_string_pretty(&arguments)?);

                    // Execute the tool
                    match execute_tool(&name, &arguments).await {
                        Ok(result) => {
                            println!("   Result: {}", result);
                            // In a real agent loop, you would send this result back to the model
                        }
                        Err(e) => {
                            println!("   Error: {}", e);
                        }
                    }
                }
            }

            StreamEvent::MessageEnd {
                finish_reason,
                usage,
            } => {
                println!("\n\n✨ Message Complete");
                println!("Finish Reason: {:?}", finish_reason);
                if let Some(u) = usage {
                    println!("Tokens - Input: {}, Output: {}", u.input_tokens, u.output_tokens);
                }
            }

            StreamEvent::ThinkingDelta { delta } => {
                print!("[Thinking] {}", delta);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }

            _ => {
                // Handle other event types as needed
            }
        }
    }

    println!("\n\n🎉 Streaming tool use example completed!");
    println!("\nKey Features Demonstrated:");
    println!("✅ Real-time tool call streaming (tools emit as they're generated)");
    println!("✅ Tool argument streaming (arguments arrive incrementally)");
    println!("✅ Immediate tool execution (no need to wait for full response)");
    println!("✅ Mixed text and tool calls in single request");

    Ok(())
}
