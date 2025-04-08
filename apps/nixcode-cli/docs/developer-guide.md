# nixcode-cli Developer Guide

This guide provides information for developers who want to contribute to or modify the nixcode-cli application.

## Table of Contents

1. [Project Structure](#project-structure)
2. [Building the Project](#building-the-project)
3. [Architecture Overview](#architecture-overview)
4. [Adding a New Command](#adding-a-new-command)
5. [Adding a New Widget](#adding-a-new-widget)
6. [Adding a New Input Mode](#adding-a-new-input-mode)
7. [Adding a New LLM Provider](#adding-a-new-llm-provider)
8. [Adding a New Tool](#adding-a-new-tool)
9. [Testing](#testing)
10. [Debugging](#debugging)

## Project Structure

The nixcode-cli application is organized as follows:

```
apps/nixcode-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Application entry point
│   ├── app.rs                  # Main application logic
│   ├── input_mode.rs           # Input mode definitions
│   ├── command_popup.rs        # Command popup widget
│   ├── model_popup.rs          # Model selection popup
│   ├── popup_utils.rs          # Popup utility functions
│   ├── status_bar.rs           # Status bar widget
│   ├── user_input.rs           # User input widget
│   ├── utils/                  # Utility functions
│   │   ├── highlights.rs       # Syntax highlighting
│   │   └── mod.rs              # Module definition
│   └── widgets/                # UI widgets
│       ├── chat.rs             # Chat widget
│       ├── message_widget.rs   # Message widget
│       └── mod.rs              # Module definition
└── docs/                       # Documentation
```

## Building the Project

### Prerequisites

- Rust toolchain (rustc, cargo)
- Git

### Development Build

To build the project for development:

```bash
cargo build
```

### Release Build

To build the project for release:

```bash
cargo build --release
```

### Running the Project

To run the project:

```bash
cargo run
```

## Architecture Overview

nixcode-cli follows an event-driven architecture with the following components:

- **App**: The main application component that manages the application state and handles events
- **Widgets**: UI components that render the interface
- **Input Modes**: Different modes for handling user input
- **Events**: Events that are passed between components

The application uses the following event flow:

1. User interacts with the terminal
2. Events are captured by the EventStream
3. Events are processed by the App's `handle_input_events()` method
4. Events are delegated to the appropriate widget based on the current input mode
5. Widgets send events through the `tx` channel
6. Events are received by the App's `rx` channel
7. Events are processed by the App's `handle_app_event()` method
8. UI is updated based on the event

## Adding a New Command

To add a new command to the application:

1. Add the command to the `AVAILABLE_COMMANDS` constant in `command_popup.rs`:

```rust
const AVAILABLE_COMMANDS: &[CommandInfo] = &[
    // Existing commands...
    CommandInfo {
        name: "new-command",
        aliases: &["nc"],
        description: "Description of the new command",
    },
];
```

2. Add a case for the command in the `execute_command()` method in `app.rs`:

```rust
async fn execute_command(&mut self, command: String) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let main_command = parts[0];

    match main_command {
        // Existing commands...
        "new-command" => {
            // Implement the command
        }
        _ => {
            log::warn!("Command not implemented: {}", command);
        }
    }

    self.set_input_mode(InputMode::Normal);
}
```

## Adding a New Widget

To add a new widget to the application:

1. Create a new file in the `widgets` directory:

```rust
// widgets/new_widget.rs
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub struct NewWidget {
    // Widget state
}

impl NewWidget {
    pub fn new() -> Self {
        NewWidget {
            // Initialize state
        }
    }
}

impl Widget for &NewWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the widget
    }
}
```

2. Add the widget to the `widgets/mod.rs` file:

```rust
pub mod chat;
pub mod message_widget;
pub mod new_widget;
```

3. Use the widget in the application:

```rust
use crate::widgets::new_widget::NewWidget;

// Create the widget
let new_widget = NewWidget::new();

// Render the widget
frame.render_widget(&new_widget, area);
```

## Adding a New Input Mode

To add a new input mode to the application:

1. Add the mode to the `InputMode` enum in `input_mode.rs`:

```rust
#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
    NewMode,
}

impl InputMode {
    pub fn to_string(&self) -> &str {
        match self {
            InputMode::Normal => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
            InputMode::NewMode => "New Mode",
        }
    }
}
```

2. Add a handler for the mode in the `handle_input_events()` method in `app.rs`:

```rust
match self.input_mode {
    InputMode::Insert => self.handle_insert_input_events(&event),
    InputMode::Normal => self.handle_normal_input_events(&event),
    InputMode::Command => self.handle_command_input_events(event),
    InputMode::NewMode => self.handle_new_mode_input_events(&event),
}
```

3. Implement the handler method:

```rust
fn handle_new_mode_input_events(&mut self, event: &Event) {
    // Handle input events for the new mode
}
```

## Adding a New LLM Provider

To add a new LLM provider to the application:

1. Add the provider to the `LLMProvider` enum in the Nixcode core library
2. Implement the provider client in the Nixcode core library
3. Add the provider to the `change_model()` method in `app.rs`:

```rust
let provider = match model.provider() {
    LLMProvider::Anthropic => "anthropic",
    LLMProvider::OpenAI => "openai",
    LLMProvider::Groq => "groq",
    LLMProvider::OpenRouter => "open_router",
    LLMProvider::Gemini => "gemini",
    LLMProvider::NewProvider => "new_provider",
};
```

4. Add the provider color to the `get_provider_color()` function in `model_popup.rs`:

```rust
fn get_provider_color(provider: &LLMProvider) -> Color {
    match provider {
        LLMProvider::Anthropic => Color::Rgb(163, 77, 253), // Purple
        LLMProvider::OpenAI => Color::Rgb(16, 163, 127),    // Green
        LLMProvider::Groq => Color::Rgb(255, 165, 0),       // Orange
        LLMProvider::OpenRouter => Color::Rgb(59, 130, 246), // Blue
        LLMProvider::Gemini => Color::Rgb(234, 67, 53),     // Red
        LLMProvider::NewProvider => Color::Rgb(255, 0, 255), // New color
    }
}
```

5. Add the provider color to the `render()` method in `status_bar.rs`:

```rust
let provider_color = match model.provider() {
    LLMProvider::Anthropic => Color::Rgb(163, 77, 253), // Purple
    LLMProvider::OpenAI => Color::Rgb(16, 163, 127),    // Green
    LLMProvider::Groq => Color::Rgb(255, 165, 0),       // Orange
    LLMProvider::OpenRouter => Color::Rgb(59, 130, 246), // Blue
    LLMProvider::Gemini => Color::Rgb(234, 67, 53),     // Red
    LLMProvider::NewProvider => Color::Rgb(255, 0, 255), // New color
};
```

## Adding a New Tool

To add a new tool to the application:

1. Implement the tool in the Nixcode core library
2. The tool will automatically be available to the LLM

## Testing

### Running Tests

To run the tests:

```bash
cargo test
```

### Writing Tests

To write a test for a component:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component() {
        // Test the component
    }
}
```

## Debugging

### Logging

The application uses the `log` crate for logging. You can enable debug logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

Logs are written to the `.nixcode/debug.log` file.

### Debugging with VS Code

To debug the application with VS Code, add the following configuration to `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug nixcode-cli",
            "cargo": {
                "args": [
                    "build",
                    "--bin=nixcode-cli",
                    "--package=nixcode-cli"
                ],
                "filter": {
                    "name": "nixcode-cli",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Inspecting State

To inspect the application state, you can add debug prints to the code:

```rust
log::debug!("App state: {:?}", self);
```

You can also use the `dbg!` macro:

```rust
dbg!(&self.input_mode);
```