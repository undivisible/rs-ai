use async_trait::async_trait;
use rusty_ai::tool::Tool;
use rusty_ai::*;
use rusty_chatgpt::ChatGptProvider;

// Define a calculator tool
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".into(),
            description: "Performs basic arithmetic. Input: JSON with 'operation' (add/subtract/multiply/divide) and 'a', 'b' numbers.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["add", "subtract", "multiply", "divide"] },
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["operation", "a", "b"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, AiError> {
        let op = args["operation"].as_str().unwrap_or("add");
        let a = args["a"].as_f64().unwrap_or(0.0);
        let b = args["b"].as_f64().unwrap_or(0.0);
        let result = match op {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b != 0.0 {
                    a / b
                } else {
                    return Ok("Error: division by zero".into());
                }
            }
            _ => return Ok(format!("Unknown operation: {op}")),
        };
        Ok(format!("{result}"))
    }
}

// Define a weather tool
struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get the current weather for a city.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": { "type": "string" }
                },
                "required": ["city"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, AiError> {
        let city = args["city"].as_str().unwrap_or("Unknown");
        // Fake weather data for demonstration
        Ok(format!(
            "Weather in {city}: 72\u{00b0}F, sunny with light clouds"
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ChatGptProvider::new(std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY required"));
    let model = provider.gpt4o_mini();

    let mut tools = ToolSet::new();
    tools.add(CalculatorTool);
    tools.add(WeatherTool);

    let prompt = Prompt::from("What's the weather in Tokyo? Also, what is 42 * 17?");
    let max_steps = 5;

    // Tool calling loop
    let mut messages = prompt.into_messages();
    for step in 0..max_steps {
        println!("--- Step {step} ---");

        let options = GenerateOptions {
            tools: Some(tools.definitions()),
            tool_choice: Some(ToolChoice::Auto),
            ..Default::default()
        };

        let result = model
            .generate(Prompt::Messages(messages.clone()), options)
            .await?;

        if let Some(text) = &result.text {
            println!("Assistant: {text}");
        }

        if result.tool_calls.is_empty() || result.finish_reason != FinishReason::ToolCall {
            println!("No more tool calls. Done!");
            break;
        }

        // Add assistant message carrying the tool calls
        let mut assistant_parts: Vec<ContentPart> = Vec::new();
        if let Some(text) = &result.text {
            assistant_parts.push(ContentPart::Text { text: text.clone() });
        }
        for call in &result.tool_calls {
            assistant_parts.push(ContentPart::ToolCall { call: call.clone() });
        }
        messages.push(Message {
            role: Role::Assistant,
            content: assistant_parts,
            name: None,
            metadata: Default::default(),
        });

        // Execute each tool call and add results
        for call in &result.tool_calls {
            println!("Calling tool '{}' with args: {}", call.name, call.arguments);
            let tool_result = tools.execute(call).await?;
            println!("Tool result: {}", tool_result.content);
            messages.push(Message::tool_result(
                &tool_result.call_id,
                &tool_result.content,
            ));
        }
    }

    Ok(())
}
