# nixcode-cli User Guide

This guide provides instructions for using the nixcode-cli application, a terminal-based client for interacting with Large Language Models (LLMs).

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [Starting the Application](#starting-the-application)
4. [Input Modes](#input-modes)
5. [Sending Messages](#sending-messages)
6. [Commands](#commands)
7. [Model Selection](#model-selection)
8. [Navigation](#navigation)
9. [Tool Integration](#tool-integration)
10. [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- Rust toolchain (rustc, cargo)
- Git
- API keys for LLM providers (Anthropic, OpenAI, Groq, OpenRouter, Gemini)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/nixcode-ai.git
   cd nixcode-ai
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/nixcode-cli
   ```

## Configuration

### API Keys

nixcode-cli requires API keys for the LLM providers you want to use. You can set these keys as environment variables:

- Anthropic: `ANTHROPIC_API_KEY`
- OpenAI: `OPENAI_API_KEY`
- Groq: `GROQ_API_KEY`
- OpenRouter: `OPENROUTER_API_KEY`
- Gemini: `GEMINI_API_KEY`

You can set these environment variables in your shell or create a `.env` file in the project directory:

```
ANTHROPIC_API_KEY=your_anthropic_api_key
OPENAI_API_KEY=your_openai_api_key
GROQ_API_KEY=your_groq_api_key
OPENROUTER_API_KEY=your_openrouter_api_key
GEMINI_API_KEY=your_gemini_api_key
```

### Configuration Files

nixcode-cli uses a layered configuration system:

1. **Default configuration**: Hard-coded defaults in the code
2. **Global configuration**: `~/.config/nixcode-ai/config.toml` for user-wide settings
3. **Project configuration**: `.nixcode/config.toml` for project-specific settings

Example configuration file:

```toml
[llm]
default_provider = "anthropic"

[tools]
enabled = true

[tools.overrides]
git_add = false
read_text_file = true
```

## Starting the Application

To start nixcode-cli, navigate to your project directory and run:

```bash
nixcode-cli
```

The application will start in Normal mode with an empty chat.

## Input Modes

nixcode-cli uses a Vim-inspired input mode system with three modes:

### Normal Mode

Normal mode is the default mode when you start the application. In this mode, you can:

- Press `i` to enter Insert mode
- Press `:` to enter Command mode
- Use `j` or Down arrow to scroll down
- Use `k` or Up arrow to scroll up

### Insert Mode

Insert mode allows you to type messages to send to the LLM. In this mode, you can:

- Type text to send to the LLM
- Press Enter to send the message
- Press Esc to return to Normal mode

### Command Mode

Command mode allows you to execute commands. In this mode, you can:

- Type commands to execute
- Use Tab for command completion
- Use Up/Down arrows to navigate command suggestions
- Press Enter to execute the command
- Press Esc to return to Normal mode

## Sending Messages

To send a message to the LLM:

1. Enter Insert mode by pressing `i` in Normal mode
2. Type your message
3. Press Enter to send the message

The LLM will respond with its answer, which will be displayed in the chat area.

## Commands

nixcode-cli supports the following commands:

| Command | Aliases | Description |
|---------|---------|-------------|
| `quit` | `exit`, `q` | Exit the application |
| `clear` | | Clear the chat history |
| `retry` | | Retry the last message |
| `remove-last-message` | `remove-last`, `remove-last-msg`, `remove-msg`, `rlm` | Remove the last message from the chat |
| `model` | `models`, `m` | List and select LLM models |

To execute a command:

1. Enter Command mode by pressing `:` in Normal mode
2. Type the command
3. Press Enter to execute the command

## Model Selection

nixcode-cli supports multiple LLM providers and models. To select a model:

1. Enter Command mode by pressing `:` in Normal mode
2. Type `model` and press Enter
3. Use Up/Down arrows or `j`/`k` to navigate the model list
4. Press Enter to select a model

The model popup displays the available models with color-coded providers:

- Anthropic: Purple
- OpenAI: Green
- Groq: Orange
- OpenRouter: Blue
- Gemini: Red

The current model is highlighted in green.

## Navigation

### Scrolling

To scroll through the chat history:

- In Normal mode, use `j` or Down arrow to scroll down
- In Normal mode, use `k` or Up arrow to scroll up

### Cursor Movement

In Insert mode, you can move the cursor:

- Use Left/Right arrows to move the cursor
- Use Home to move to the beginning of the line
- Use End to move to the end of the line

### Command Navigation

In Command mode, you can:

- Use Tab to complete the current command
- Use Up/Down arrows to navigate command suggestions

## Tool Integration

nixcode-cli integrates with the Nixcode tool system, allowing the LLM to:

- Read and write files
- Execute git commands
- Search for content in files
- Run cargo commands
- And more

Tool calls are visualized in the chat interface, and tool results are displayed and sent back to the LLM.

### Tool Visualization

When the LLM uses a tool, the tool call is displayed in the chat interface:

```
tool_name(param1: "value1", param2: "value2")
```

The tool result is displayed below the tool call:

```
[call_id] 
result line 1
result line 2
result line 3
...
```

## Troubleshooting

### API Key Issues

If you encounter issues with API keys:

1. Check that the environment variables are set correctly
2. Check that the API keys are valid
3. Check the log file at `.nixcode/debug.log` for error messages

### Application Crashes

If the application crashes:

1. Check the log file at `.nixcode/debug.log` for error messages
2. Try running the application with debug output:
   ```bash
   RUST_LOG=debug nixcode-cli
   ```

### Model Selection Issues

If you encounter issues with model selection:

1. Check that the API key for the provider is set correctly
2. Check that the provider is supported
3. Try selecting a different model

### Tool Execution Issues

If you encounter issues with tool execution:

1. Check that the tools are enabled in the configuration
2. Check that the tool has the necessary permissions
3. Check the log file at `.nixcode/debug.log` for error messages