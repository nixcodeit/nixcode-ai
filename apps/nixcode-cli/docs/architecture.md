# nixcode-cli Architecture

This document provides a detailed overview of the nixcode-cli application architecture.

## Overview

nixcode-cli is structured as a terminal-based application that follows an event-driven architecture. It uses the ratatui library for terminal UI rendering and the crossterm library for terminal input handling. The application communicates with the Nixcode core library for LLM interactions and tool execution.

## Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        nixcode-cli                               │
│                                                                 │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────────┐   │
│  │  main   │───▶│   App   │◀───│ Widgets │◀───│ Input Modes │   │
│  └─────────┘    └─────────┘    └─────────┘    └─────────────┘   │
│        │             │              │               │           │
│        │             │              │               │           │
│        ▼             ▼              ▼               ▼           │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────────┐   │
│  │ Nixcode │───▶│ Events  │───▶│ Commands│───▶│ Status Bar  │   │
│  └─────────┘    └─────────┘    └─────────┘    └─────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### Main (`main.rs`)

The main entry point is responsible for:
- Initializing logging
- Loading environment variables
- Creating the Nixcode client
- Initializing the terminal UI
- Running the main application loop

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    simple_logging::log_to_file(".nixcode/debug.log", log::LevelFilter::Debug)?;
    
    // Load environment variables
    dotenv().ok();

    // Create project from current directory
    let project = Project::new(current_dir().unwrap_or(PathBuf::from(".")));

    // Create Nixcode client
    let nixcode_result = Nixcode::new_from_env(project);
    
    // Initialize terminal UI
    let mut terminal = ratatui::init();
    
    // Create and run app
    let mut app = App::new(nixcode).expect("Failed to create app");
    let app_result = app.run(&mut terminal).await;
    
    // Restore terminal state
    ratatui::restore();

    app_result
}
```

### App (`app.rs`)

The App struct is the central component that:
- Manages the application state
- Handles input events
- Processes application events
- Coordinates between UI components and the Nixcode client
- Renders the UI
- Executes commands
- Manages model selection

The App struct contains:
- `should_quit`: Boolean flag indicating if the application should exit
- `chat_view`: The chat widget
- `current_view`: The current view (currently only Chat)
- `input_mode`: The current input mode
- `rx`: Receiver for application events
- `tx`: Sender for application events
- `nixcode_rx`: Receiver for Nixcode events
- `nixcode`: Reference to the Nixcode client
- `command_popup`: The command popup widget
- `model_popup`: The model selection popup widget
- `is_changing_model`: Flag indicating if a model change is in progress

The main event loop in `run()` processes three types of events:
1. Application events from `rx`
2. Input events from the terminal
3. Nixcode events from `nixcode_rx`

```rust
pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> tokio::io::Result<()> {
    let mut events = EventStream::new();

    while !self.should_quit {
        self.draw(terminal).await?;

        tokio::select! {
            Some(event) = self.rx.recv() => {
                self.handle_app_event(event).await;
            },
            Some(Ok(event)) = events.next() => {
                self.handle_input_events(event).await;
            },
            Some(nixcode_event) = self.nixcode_rx.recv() => {
                self.handle_nixcode_event(nixcode_event).await;
            }
        }
    }

    Ok(())
}
```

### Input Modes (`input_mode.rs`)

The application supports three input modes inspired by Vim:
- **Normal**: For navigation and command activation
- **Insert**: For text input
- **Command**: For entering commands

```rust
#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}
```

### Command Popup (`command_popup.rs`)

The CommandPopup struct provides a command-line interface for executing commands with:
- Command suggestions
- Tab completion
- Command history
- Error handling

The CommandPopup contains:
- `command`: The current command text
- `tx`: Sender for application events
- `suggestions`: List of command suggestions
- `selected_suggestion`: The currently selected suggestion
- `command_is_valid`: Flag indicating if the current command is valid

Commands are defined in the `AVAILABLE_COMMANDS` constant:

```rust
const AVAILABLE_COMMANDS: &[CommandInfo] = &[
    CommandInfo {
        name: "quit",
        aliases: &["exit", "q"],
        description: "Exit the application",
    },
    CommandInfo {
        name: "clear",
        aliases: &[],
        description: "Clear the chat history",
    },
    // ...
];
```

### Model Popup (`model_popup.rs`)

The ModelPopup struct allows users to select different LLM models with:
- Visual indication of the current model
- Provider-based color coding
- Navigation and selection controls

The ModelPopup contains:
- `tx`: Sender for application events
- `selected_index`: The currently selected model index
- `current_model`: The currently active model

### Chat Widget (`widgets/chat.rs`)

The Chat struct displays the conversation with the LLM and includes:
- Message rendering
- Scrolling
- Input handling
- Cost tracking
- Status indicators

The Chat struct contains:
- `vertical_scroll_state`: State for the scrollbar
- `lines`: The lines of text to display
- `paragraph`: The paragraph widget for rendering text
- `client`: Reference to the Nixcode client
- `input_mode`: The current input mode
- `app_event`: Sender for application events
- `prompt`: The user input widget
- `area_size`: The size of the chat area
- `stick_to_bottom`: Flag indicating if the view should stick to the bottom
- `scroll`: The current scroll position
- `total_lines`: The total number of lines in the chat
- `usage`: Token usage information
- `waiting`: Flag indicating if waiting for a response
- `error`: Any error that occurred
- `total_cost`: The total cost of the conversation

### Message Widget (`widgets/message_widget.rs`)

The MessageWidget struct renders individual messages with:
- Role-based styling
- Code block formatting
- Tool call visualization
- Tool result display

The `get_lines()` method converts an LLMMessage into a vector of Lines for rendering:

```rust
pub fn get_lines<'a>(message: LLMMessage) -> Vec<Line<'a>> {
    let author = match message.role.as_str() {
        "user" => Span::styled("You > ", Style::new().green()),
        "assistant" => Span::styled("Assistant > ", Style::new().yellow()),
        "system" | "developer" => Span::styled("System > ", Style::new().dark_gray()),
        _ => Span::styled("Unknown > ", Style::new().red()),
    }
    .bold();

    // Process message content...
}
```

### Status Bar (`status_bar.rs`)

The StatusBar struct displays:
- Current input mode (Normal, Insert, Command)
- Current LLM model and provider
- Application version
- Current date and time

The StatusBar contains:
- `current_mode`: The current input mode
- `current_model`: The current LLM model

### User Input (`user_input.rs`)

The UserSingleLineInput struct provides a single-line text input widget with:
- Cursor movement
- Text editing
- Unicode support
- Horizontal scrolling

The UserSingleLineInput contains:
- `data`: The text content
- `cursor`: The cursor position in characters
- `cursor_byte`: The cursor position in bytes
- `scroll_offset`: The horizontal scroll offset
- `last_area_width`: The width of the last render area

### Syntax Highlighting (`utils/highlights.rs`)

The syntax highlighting module provides code block highlighting using the syntect library:

```rust
pub fn highlight_code<'a>(
    code: String,
    extension: &str,
) -> Result<Vec<Line<'a>>, Box<dyn std::error::Error>> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme = &THEME;

    // Highlight code...
}
```

## Event Flow

1. **Input Events**:
   - User interacts with the terminal
   - Events are captured by the EventStream
   - Events are processed by the App's `handle_input_events()` method
   - Events are delegated to the appropriate widget based on the current input mode

2. **Application Events**:
   - Widgets send events through the `tx` channel
   - Events are received by the App's `rx` channel
   - Events are processed by the App's `handle_app_event()` method
   - UI is updated based on the event

3. **Nixcode Events**:
   - Nixcode client sends events through the `nixcode_rx` channel
   - Events are processed by the App's `handle_nixcode_event()` method
   - UI is updated based on the event

## Rendering Flow

1. The App's `draw()` method is called in the main event loop
2. The App's `draw_frame()` method renders the UI components:
   - Chat widget
   - Status bar
   - Command popup (if in Command mode)
   - Model popup (if active)
3. Each widget's `render()` method is called with its area
4. The cursor position is set based on the current input mode

## Command Execution Flow

1. User enters Command mode by pressing `:`
2. User types a command
3. Command is validated and suggestions are shown
4. User presses Enter to execute the command
5. Command is sent through the `tx` channel as an AppEvent::Command
6. App's `execute_command()` method processes the command
7. Appropriate action is taken based on the command
8. UI is updated based on the action

## Model Selection Flow

1. User activates the model popup with the `model` command
2. Model popup displays available models
3. User navigates to the desired model
4. User presses Enter to select the model
5. Model selection is sent through the `tx` channel as an AppEvent::ChangeModel
6. App's `change_model()` method processes the selection
7. Nixcode client is reset and recreated with the new model
8. UI is updated to reflect the new model

## Message Flow

1. User enters a message in Insert mode
2. User presses Enter to send the message
3. Message is sent to the Nixcode client
4. Nixcode client sends the message to the LLM
5. LLM response is streamed back to the application
6. Response is displayed in the chat interface
7. Tool calls are executed if present in the response
8. Tool results are sent back to the LLM
9. Conversation continues