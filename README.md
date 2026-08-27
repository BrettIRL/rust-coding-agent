# Rust Coding Agent

A small command-line coding agent written in Rust. It sends a prompt to an
OpenAI-compatible chat completion API and lets the model inspect files, write
files, and run approved shell commands until it returns a final response or
reaches the step limit.

## Warning

This agent can overwrite files and execute arbitrary shell commands with the
same permissions as the user who runs it. It asks for confirmation before each
file write or shell command, but does not sandbox approved operations. Review
the requested operation carefully and commit important work beforehand.

## Features

- Accepts a prompt through a command-line flag.
- Uses OpenRouter by default with the `anthropic/claude-haiku-4.5` model.
- Supports any OpenAI-compatible endpoint through `OPENROUTER_BASE_URL`.
- Gives the model three tools:
  - `Read` reads a UTF-8 text file.
  - `Write` creates or replaces a file.
  - `Bash` executes a command with `bash -c` and returns stdout and stderr.
- Handles every tool call returned in a model response.
- Asks for confirmation before each `Write` and `Bash` call.
- Stops after 20 model responses to prevent unbounded loops.

## Architecture

The implementation lives in `src/main.rs` and has two main parts:

1. `main` parses the prompt, configures the API client, declares the available
   tools, and drives the agent loop.
2. `execute_tool_call` dispatches model requests to the local filesystem or a
   Bash subprocess and returns the result to the model.

Conversation state is an in-memory list of chat messages. After each model
response, the assistant message and any tool result are appended before the
next API request.

## Requirements

- Rust 1.95 or newer
- Bash
- An OpenRouter API key, or credentials for another OpenAI-compatible endpoint

## Setup

Set the API key in your environment:

```sh
export OPENROUTER_API_KEY="your-api-key"
```

OpenRouter is the default endpoint. To use another compatible service, set its
base URL as well:

```sh
export OPENROUTER_BASE_URL="https://example.com/v1"
```

Build the executable with Cargo:

```sh
cargo build --release
```

## Usage

Run the agent with a single prompt:

```sh
cargo run --release -- --prompt "Summarize this Rust project"
```

The short form is also available:

```sh
cargo run --release -- -p "Add tests for the parser"
```

The included runner script builds the release executable in Cargo's standard
`target/release` directory and forwards all arguments to it:

```sh
./your_program.sh --prompt "Explain src/main.rs"
```

## Limitations

- The model name is currently fixed at compile time.
- There is no sandbox, path restriction, command timeout, or output-size limit.
- File reads require UTF-8 text, and file writes replace the complete file.
- Sessions accept one initial prompt and are not persisted between runs.
- API and malformed-response errors terminate the process rather than being
  retried or recovered.

## Attribution

Originally built through the CodeCrafters Build Your Own Claude Code challenge.
