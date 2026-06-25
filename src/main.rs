use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    let mut messages = vec![json!({"role": "user", "content": args.prompt})];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": messages,
                "model": "anthropic/claude-haiku-4.5",
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "Read",
                        "description": "Read and return the contents of a file.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "The path to the file to read.",
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                }, {
                        "type": "function",
                        "function": {
                            "name": "Write",
                            "description": "Write content to a file.",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "file_path": {
                                        "type": "string",
                                        "description": "The path of the file to write to.",
                                    },
                                    "content": {
                                        "type": "string",
                                        "description": "The content to write to the file.",
                                    }
                                },
                                "required": ["file_path", "content"]
                            }
                        }
                    },
                    {
                        "type": "function",
                        "function": {
                            "name": "Bash",
                            "description": "Execute a shell command.",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "command": {
                                        "type": "string",
                                        "description": "The command to execute.",
                                    }
                                },
                                "required": ["command"]
                            }
                        }
                    }]
            }))
            .await?;

        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        let message = &response["choices"][0]["message"];
        messages.push(message.clone());

        if let Some(tool_calls) = message["tool_calls"].as_array() {
            let tool_call = &tool_calls[0];
            let tool_call_id = tool_call["id"].as_str().unwrap();
            let name = tool_call["function"]["name"].as_str().unwrap();
            let arguments: Value =
                serde_json::from_str(tool_call["function"]["arguments"].as_str().unwrap())?;
            let contents = execute_tool_call(name, arguments)?;

            messages
                .push(json!({"role": "tool", "tool_call_id": tool_call_id,  "content": contents}));
        } else if let Some(content) = message["content"].as_str() {
            println!("{}", content);
            break;
        }
    }

    Ok(())
}

fn execute_tool_call(name: &str, arguments: Value) -> Result<String, Box<dyn std::error::Error>> {
    match name {
        "Read" => {
            let file_path = arguments["file_path"].as_str().unwrap();
            let contents = std::fs::read_to_string(file_path)?;
            Ok(contents)
        }
        "Write" => {
            let file_path = arguments["file_path"].as_str().unwrap();
            let content = arguments["content"].as_str().unwrap();
            std::fs::write(file_path, content)?;
            Ok(format!("Successfully wrote to {}", file_path))
        }
        "Bash" => {
            let command = arguments["command"].as_str().unwrap();
            let output = std::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()?;
            Ok(format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ))
        }
        _ => Err(format!("Unknown tool call: {}", name).into()),
    }
}
