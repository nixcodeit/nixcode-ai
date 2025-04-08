# nixcode-cli Documentation

Welcome to the nixcode-cli documentation. This documentation provides comprehensive information about the nixcode-cli application, a terminal-based client for interacting with Large Language Models (LLMs).

## Overview

nixcode-cli is a terminal-based client for interacting with Large Language Models (LLMs), with support for multiple providers including Anthropic (Claude), OpenAI (GPT models), Groq, OpenRouter, and Gemini. It's a Rust-based TUI (Text User Interface) application that provides a modern, terminal-friendly interface for communicating with AI assistants directly from the command line.

The application features a vim-inspired input system, a chat interface, and an innovative tool invocation framework that allows the AI to use external tools.

## Documentation Sections

### [README](README.md)

The README provides a high-level overview of the nixcode-cli application, including its features, architecture, and components.

### [Architecture](architecture.md)

The Architecture document provides a detailed overview of the nixcode-cli application architecture, including its component diagram, core components, event flow, rendering flow, command execution flow, model selection flow, and message flow.

### [Components](components.md)

The Components document provides detailed documentation for each component in the nixcode-cli application, including the main entry point, app, input modes, command popup, model popup, chat widget, message widget, status bar, user input, popup utilities, and syntax highlighting.

### [User Guide](user-guide.md)

The User Guide provides instructions for using the nixcode-cli application, including installation, configuration, starting the application, input modes, sending messages, commands, model selection, navigation, tool integration, and troubleshooting.

### [Developer Guide](developer-guide.md)

The Developer Guide provides information for developers who want to contribute to or modify the nixcode-cli application, including project structure, building the project, architecture overview, adding new commands, widgets, input modes, LLM providers, and tools, testing, and debugging.

### [API Reference](api-reference.md)

The API Reference provides a detailed API reference for the nixcode-cli application, including structs, enums, and methods for each component.

## Getting Started

To get started with nixcode-cli, follow these steps:

1. [Install the application](user-guide.md#installation)
2. [Configure the application](user-guide.md#configuration)
3. [Start the application](user-guide.md#starting-the-application)
4. [Send messages to the LLM](user-guide.md#sending-messages)

## Contributing

If you want to contribute to nixcode-cli, follow these steps:

1. [Set up the development environment](developer-guide.md#building-the-project)
2. [Understand the architecture](architecture.md)
3. [Understand the components](components.md)
4. [Make your changes](developer-guide.md#adding-a-new-command)
5. [Test your changes](developer-guide.md#testing)
6. [Submit a pull request](https://github.com/yourusername/nixcode-ai/pulls)

## License

nixcode-cli is licensed under the [MIT License](https://opensource.org/licenses/MIT).