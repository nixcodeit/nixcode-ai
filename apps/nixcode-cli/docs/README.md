# nixcode-cli Documentation

This documentation provides a comprehensive overview of the nixcode-cli application, a terminal-based client for interacting with Large Language Models (LLMs).

## Table of Contents

1. [Introduction](#introduction)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [User Interface](#user-interface)
5. [Input Modes](#input-modes)
6. [Commands](#commands)
7. [Model Selection](#model-selection)
8. [Chat Interface](#chat-interface)
9. [Message Handling](#message-handling)
10. [Syntax Highlighting](#syntax-highlighting)
11. [Status Bar](#status-bar)
12. [Event System](#event-system)
13. [Tool Integration](#tool-integration)

## Introduction

nixcode-cli is a terminal-based client for interacting with Large Language Models (LLMs), with support for multiple providers including Anthropic (Claude), OpenAI (GPT models), Groq, OpenRouter, and Gemini. It's a Rust-based TUI (Text User Interface) application that provides a modern, terminal-friendly interface for communicating with AI assistants directly from the command line.

The application features a vim-inspired input system, a chat interface, and an innovative tool invocation framework that allows the AI to use external tools.

## Architecture

The nixcode-cli application follows a modular architecture with clear separation of concerns:

- **Main Application**: Entry point and application lifecycle management
- **App**: Core application logic and event handling
- **Widgets**: UI components for rendering the interface
- **Input Handling**: Vim-inspired input mode system
- **Command System**: Command parsing and execution
- **Model Selection**: Interface for selecting LLM models
- **Event System**: Event-driven communication between components

The application uses the ratatui library for terminal UI rendering and the crossterm library for terminal input handling. It communicates with the Nixcode core library for LLM interactions and tool execution.

## Core Components

### Main Entry Point (`main.rs`)

The main entry point initializes the application, sets up logging, loads environment variables, creates the Nixcode client, initializes the terminal UI, and runs the main application loop.

### Application Core (`app.rs`)

The App struct is the central component that:
- Manages the application state
- Handles input events
- Processes application events
- Coordinates between UI components and the Nixcode client
- Renders the UI
- Executes commands
- Manages model selection

### Input Modes (`input_mode.rs`)

The application supports three input modes inspired by Vim:
- **Normal**: For navigation and command activation
- **Insert**: For text input
- **Command**: For entering commands

### Command Popup (`command_popup.rs`)

The command popup provides a command-line interface for executing commands with:
- Command suggestions
- Tab completion
- Command history
- Error handling

### Model Popup (`model_popup.rs`)

The model popup allows users to select different LLM models with:
- Visual indication of the current model
- Provider-based color coding
- Navigation and selection controls

### Chat Interface (`widgets/chat.rs`)

The chat widget displays the conversation with the LLM and includes:
- Message rendering
- Scrolling
- Input handling
- Cost tracking
- Status indicators

### Message Widget (`widgets/message_widget.rs`)

The message widget renders individual messages with:
- Role-based styling
- Code block formatting
- Tool call visualization
- Tool result display

## User Interface

The nixcode-cli interface consists of:

1. **Chat Area**: Displays the conversation with the LLM
2. **Input Area**: For entering messages to the LLM
3. **Status Bar**: Shows the current input mode, model, and application status
4. **Command Popup**: Appears when in command mode
5. **Model Selection Popup**: Appears when selecting a model

## Input Modes

### Normal Mode

In Normal mode, you can:
- Press `i` to enter Insert mode
- Press `:` to enter Command mode
- Use `j` or Down arrow to scroll down
- Use `k` or Up arrow to scroll up

### Insert Mode

In Insert mode, you can:
- Type text to send to the LLM
- Press Enter to send the message
- Press Esc to return to Normal mode

### Command Mode

In Command mode, you can:
- Type commands to execute
- Use Tab for command completion
- Use Up/Down arrows to navigate command suggestions
- Press Enter to execute the command
- Press Esc to return to Normal mode

## Commands

The application supports the following commands:

| Command | Aliases | Description |
|---------|---------|-------------|
| `quit` | `exit`, `q` | Exit the application |
| `clear` | | Clear the chat history |
| `retry` | | Retry the last message |
| `remove-last-message` | `remove-last`, `remove-last-msg`, `remove-msg`, `rlm` | Remove the last message from the chat |
| `model` | `models`, `m` | List and select LLM models |

## Model Selection

The application supports multiple LLM providers and models:

### Anthropic
- Claude 3.7 Sonnet
- Claude 3.5 Haiku

### OpenAI
- GPT-4o
- GPT 3o Mini

### Groq
- DeepSeek R1
- Llama 4 Scout
- Qwen Qwq 32b

### OpenRouter
- Quasar Alpha
- DeepSeek V3
- Llama 4 Scout
- Gemini 2.5 Pro Preview

### Gemini
- Gemini models

Models can be selected using the model popup, which is activated with the `model` command.

## Chat Interface

The chat interface displays the conversation with the LLM and includes:

- **Message Display**: Shows user and assistant messages with appropriate styling
- **Scrolling**: Allows scrolling through the conversation history
- **Cost Tracking**: Displays the total cost of the conversation
- **Status Indicators**: Shows when the application is waiting for a response
- **Project Status**: Indicates whether project analysis is initialized

## Message Handling

Messages are processed through the following flow:

1. User enters a message in Insert mode
2. Message is sent to the Nixcode client
3. Nixcode client sends the message to the LLM
4. LLM response is streamed back to the application
5. Response is displayed in the chat interface
6. Tool calls are executed if present in the response
7. Tool results are sent back to the LLM
8. Conversation continues

## Syntax Highlighting

The application includes syntax highlighting for code blocks in messages using the syntect library. Supported languages include:

- Rust
- Python
- JavaScript
- HTML
- CSS
- JSON
- Markdown
- And many more

## Status Bar

The status bar displays:

- Current input mode (Normal, Insert, Command)
- Current LLM model and provider
- Application version
- Current date and time

## Event System

The application uses an event-driven architecture with:

- **Input Events**: From the terminal
- **Application Events**: Internal events for UI updates
- **Nixcode Events**: Events from the Nixcode client

Events are processed in the main event loop in `app.rs`.

## Tool Integration

The application integrates with the Nixcode tool system, allowing the LLM to:

- Read and write files
- Execute git commands
- Search for content in files
- Run cargo commands
- And more

Tool calls are visualized in the chat interface, and tool results are displayed and sent back to the LLM.