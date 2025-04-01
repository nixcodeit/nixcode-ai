# nixcode-ai

[![Rust Build & Test](https://github.com/nixcodeit/nixcode-ai/actions/workflows/rust-build.yml/badge.svg)](https://github.com/nixcodeit/nixcode-ai/actions/workflows/rust-build.yml)
[![License: Custom](https://img.shields.io/badge/License-Custom-blue.svg)](LICENSE.md)
[![Rust Version: 1.76+](https://img.shields.io/badge/Rust-1.76+-orange.svg)](https://www.rust-lang.org/)
[![Version: 0.2.0](https://img.shields.io/badge/Version-0.2.0-green.svg)](https://github.com/nixcodeit/nixcode-ai)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/nixcodeit/nixcode-ai/pulls)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-red.svg)](https://www.rust-lang.org/)
[![Terminal Application](https://img.shields.io/badge/Terminal-Application-black.svg)](https://github.com/nixcodeit/nixcode-ai)

A terminal-based client for interacting with Large Language Models (LLM), with a focus on Claude AI.

## Overview

nixcode-ai is a Rust-based TUI (Text User Interface) application that provides a modern, terminal-friendly interface for
interacting with LLMs, particularly Anthropic's Claude models. It aims to provide a streamlined, efficient interface for
AI assistance right in your terminal.

![](assets/showcase-1.gif)

## Features

- Terminal-based chat interface built with [ratatui](https://github.com/ratatui-org/ratatui)
- Vim-inspired input modes (normal, insert, command)
- Streaming responses from Claude AI
- Tool invocation framework allowing AI to use external tools
- Command popup for executing special commands
- Configurable via external TOML configuration files
- Event-driven architecture for responsive UI

## Todos

- [x] Vim-inspired input modes
- [x] Terminal-based chat interface
- [x] Anthropic API integration
- [x] Simple tool invocation framework
- [x] Basic command popup
- [x] External configuration file support
- [ ] OpenAI API integration
- [ ] OpenRouter API integration
- [ ] Groq API integration
- [ ] Customizable keybindings
- [ ] More tools for AI interaction
- [ ] Improved tool invocation framework
- [ ] **Autonomous AI Agent that will run independently on the server as a closed service in a separate environment (
  communication via issues/pull requests)**

## Requirements

- Rust 2021 edition
- Anthropic API key (set as `ANTHROPIC_API_KEY` environment variable or in config file)

## Installation

```bash
# Clone the repository
git clone https://github.com/nixcodeit/nixcode-ai.git
cd nixcode-ai

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Configuration

nixcode-ai can be configured using TOML configuration files. Configuration is read (if present) from:

1. User-level config: `~/.config/nixcode-ai/config.toml` (Unix) or `%APPDATA%\nixcode-ai\config.toml` (Windows)
2. Project-specific config: `.nixcode/config.toml` in the current project directory

A sample configuration template is provided at `config.toml.example`. You can copy this to the appropriate location to
customize your settings.

Example configuration:

```toml
[llm]
default_provider = "anthropic"

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
default_model = "claude-3-haiku"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
default_model = "gpt-4o-mini"
```

You can use `${ENV_VAR}` syntax to reference environment variables in configuration values.

If no configuration file is found, sensible defaults will be used, and the application will look for API keys in
environment variables.

## Project Structure

The project is organized as a Rust workspace with the following components:

- `apps/nixcode-cli`: The main CLI application
- `libs/llm_sdk`: SDK for interacting with LLM providers
- `libs/nixcode`: Core library with tools, utilities, and event system
- `libs/nixcode-macros`: Procedural macros for the project

## Usage

```bash
# Run with default settings
cargo run --release

# Make sure to set your Anthropic API key (if not in config)
export ANTHROPIC_API_KEY="your-api-key-here"
```

## Input Modes

The application uses vim-inspired input modes:
- **Normal mode**: For navigating chat history
- **Insert mode**: For typing messages to the AI
- **Command mode**: For executing special commands

## Tools

nixcode-ai includes a comprehensive tool framework that allows the LLM to invoke functions. These tools provide capabilities for the LLM to interact with the local filesystem, search for files, work with Git repositories, and more.

### File System Tools
- **create_file**: Create an empty file at a specified path
- **read_text_file**: Read the content of a text file
- **write_text_file**: Write content to a text file, overwriting existing content
- **delete_file**: Delete a file at a specified path
- **update_text_file_partial**: Update portions of a text file (partial updates)
- **delete_text_file_partial**: Delete portions of a text file

### Git Tools
- **git_add**: Track changes in git by adding files to the index
- **git_status**: Get the current git repository status
- **git_diff**: Get the diff for a specific file
- **git_commit**: Commit tracked changes with a message
- **git_log**: View commit history between references
- **git_branches**: Display git branches
- **git_branch_create**: Create a new git branch
- **git_branch_delete**: Delete a git branch
- **git_stash_save**: Save changes in git stash
- **git_stash_apply**: Apply changes from git stash
- **git_stash_list**: List all stashes in git repository
- **git_stash_drop**: Drop a stash from git stash list
- **git_tag_create**: Create a git tag
- **git_tags_list**: List git tags

### Search Tools
- **search_glob_files**: Search for files in the project directory using glob patterns, with options to include gitignored and hidden files
- **search_content**: Search for text content in files using regex patterns, with options for filtering results, including pagination via offset parameter
- **replace_content**: Replace text content in files based on regex patterns, with support for capture groups in replacements

### Project Analysis Tools
- **get_project_analysis_prompt**: Generate a comprehensive project analysis prompt for better understanding of the codebase

These tools enable powerful use cases such as:
- Code exploration and navigation
- File content analysis and modification
- Code generation and saving to files
- Version control operations
- Project structure understanding
- Text search and replace across multiple files
- Branch and tag management
- Stash operations for work-in-progress changes

The tool system is designed to be extensible, making it easy to add new capabilities for the LLM to leverage.

## License

This project is licensed under the terms found in [LICENSE.md](LICENSE.md).