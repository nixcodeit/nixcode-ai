# Contributing to nixcode-ai

First of all, thank you for considering contributing to nixcode-ai! This document provides guidelines and instructions for contributing to this terminal-based LLM client project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [License Notice](#license-notice)
- [Getting Started](#getting-started)
  - [Project Setup](#project-setup)
  - [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
  - [Branching Strategy](#branching-strategy)
  - [Commit Guidelines](#commit-guidelines)
  - [Pull Requests](#pull-requests)
- [Coding Standards](#coding-standards)
  - [Rust Specific Guidelines](#rust-specific-guidelines)
  - [Terminal UI Guidelines](#terminal-ui-guidelines)
  - [LLM Integration Guidelines](#llm-integration-guidelines)
  - [Tool Framework Guidelines](#tool-framework-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Issue Tracking](#issue-tracking)
- [Communication](#communication)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md). We expect all contributors to adhere to this code to ensure a positive and respectful environment for everyone.

## License Notice

nixcode-ai is released under a custom Source-Available License. Before contributing, please ensure you understand the license terms in [LICENSE.md](LICENSE.md). By contributing to this project, you agree that your contributions will be subject to the project's license.

**Important**: The license restricts commercial use without explicit permission from the copyright holder.

## Getting Started

### Project Setup

1. **Fork and Clone the Repository**
   ```bash
   git clone https://github.com/yourusername/nixcode-ai.git
   cd nixcode-ai
   ```

2. **Install Dependencies**
   - Ensure you have Rust (2021 edition) installed
   - If you don't have Rust, install it via [rustup](https://rustup.rs/)

3. **Environment Setup**
   - Create a `.env` file in the project root with your API keys:
     ```
     ANTHROPIC_API_KEY=your_api_key_here
     ```

4. **Build the Project**
   ```bash
   cargo build
   ```

5. **Run the Project**
   ```bash
   cargo run
   ```

### Project Structure

The project is organized as a Rust workspace with the following components:

- **apps/nixcode-cli:** The main CLI application with the TUI interface
  - `src/app.rs`: Main application logic
  - `src/main.rs`: Entry point
  - `src/widgets/`: UI components
  - `src/input_mode.rs`: Vim-inspired input mode handling

- **libs/llm_sdk:** SDK for interacting with LLM providers
  - Contains client implementations for LLM APIs
  - Message handling and formatting

- **libs/nixcode:** Core library with tools and utilities
  - `src/tools/`: Tool framework implementations
  - `src/prompts/`: System prompts for LLM interactions

- **libs/nixcode-macros:** Procedural macros for the project

## Development Workflow

### Branching Strategy

1. Create a branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. Develop your changes on this branch.

3. Keep your branch updated with the main branch:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

### Commit Guidelines

- Write clear, concise commit messages in the present tense
- Reference issue numbers in commit messages when applicable
- Keep commits focused on single logical changes
- Example format:
  ```
  feat(cli): add support for command history

  - Implement persistent command history using rustyline
  - Add configurable history file location
  - Ensure proper error handling for history file operations

  Fixes #42
  ```

### Pull Requests

1. Ensure your code passes all tests and builds successfully
2. Update documentation if necessary
3. Create a pull request against the `main` branch
4. Fill out the PR template with:
   - Description of changes
   - Related issue(s)
   - Type of change (feature, bugfix, etc.)
   - Testing performed
   - Screenshots if applicable

5. A maintainer will review your PR and may request changes

## Coding Standards

### Rust Specific Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` to format your code
- Use `clippy` to catch common mistakes and improve code quality
- Prefer using Rust idioms (e.g., `Option`, `Result`, iterators)
- Properly handle errors using `Result` types
- Document public APIs with rustdoc comments

### Terminal UI Guidelines

- Follow the existing pattern of using ratatui components
- Ensure UI is responsive and efficiently redraws only when necessary
- Support terminal resizing gracefully
- Maintain vim-inspired input modes consistency
- Ensure keyboard shortcuts are intuitive and documented

### LLM Integration Guidelines

- Follow established patterns for LLM client implementations
- Properly handle streaming responses
- Implement proper error handling for API failures
- Respect rate limits and implement appropriate backoff strategies
- Keep sensitive information (API keys) secure

### Tool Framework Guidelines

- New tools should follow the existing tool pattern
- Tools must be well-documented and have clear purposes
- Ensure tools are properly typed and validated
- Implement comprehensive error handling

## Testing

- Write unit tests for new functionality
- Ensure existing tests pass with your changes
- Manual testing of TUI components should be documented in PR descriptions
- For complex features, consider adding integration tests

## Documentation

- Update README.md when adding major features
- Document public APIs using rustdoc
- For user-facing features, update usage instructions
- Consider updating GIF demos for significant UI changes

## Issue Tracking

- Before starting work, check for existing issues
- If no issue exists for your contribution, create one first
- Use issue templates when available
- Tag issues appropriately (bug, enhancement, etc.)
- For bugs, provide steps to reproduce, expected behavior, and actual behavior

## Communication

- For quick questions, use GitHub Discussions
- For significant changes, open an issue for discussion before implementing
- Be respectful and considerate in all communications
- If you need direct contact, email: kontakt@nixcode.it

---

Thank you for contributing to nixcode-ai! Your efforts help make this project better for everyone.