# nixcode-cli Components

This document provides detailed documentation for each component in the nixcode-cli application.

## Table of Contents

1. [Main Entry Point](#main-entry-point)
2. [App](#app)
3. [Input Modes](#input-modes)
4. [Command Popup](#command-popup)
5. [Model Popup](#model-popup)
6. [Chat Widget](#chat-widget)
7. [Message Widget](#message-widget)
8. [Status Bar](#status-bar)
9. [User Input](#user-input)
10. [Popup Utilities](#popup-utilities)
11. [Syntax Highlighting](#syntax-highlighting)

## Main Entry Point

**File**: `main.rs`

The main entry point initializes the application, sets up logging, loads environment variables, creates the Nixcode client, initializes the terminal UI, and runs the main application loop.

### Key Functions

- `main()`: The main entry point for the application.

### Initialization Flow

1. Initialize logging to `.nixcode/debug.log`
2. Load environment variables from `.env` file if present
3. Create a Project instance from the current directory
4. Create a Nixcode client with configuration from environment or files
5. Initialize the terminal UI
6. Create and run the App
7. Restore terminal state on exit

## App

**File**: `app.rs`

The App struct is the central component that manages the application state, handles input events, processes application events, coordinates between UI components and the Nixcode client, renders the UI, executes commands, and manages model selection.

### Key Structs and Enums

- `AppEvent`: Enum representing application events
- `AppView`: Enum representing different views in the application
- `App`: The main application struct

### Key Methods

- `new()`: Creates a new App instance
- `run()`: Runs the main event loop
- `handle_input_events()`: Handles input events from the terminal
- `handle_app_event()`: Handles application events
- `handle_nixcode_event()`: Handles events from the Nixcode client
- `draw_frame()`: Renders the UI
- `execute_command()`: Executes a command
- `change_model()`: Changes the LLM model
- `quit()`: Exits the application

### Event Handling

The App handles three types of events:
1. Input events from the terminal
2. Application events from the `rx` channel
3. Nixcode events from the `nixcode_rx` channel

```rust
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
```

### Input Mode Handling

The App delegates input events to the appropriate handler based on the current input mode:

```rust
match self.input_mode {
    InputMode::Insert => self.handle_insert_input_events(&event),
    InputMode::Normal => self.handle_normal_input_events(&event),
    InputMode::Command => self.handle_command_input_events(event),
}
```

### Command Execution

The App executes commands through the `execute_command()` method:

```rust
async fn execute_command(&mut self, command: String) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let main_command = parts[0];

    match main_command {
        "quit" => self.quit(),
        "clear" => {
            self.tx.send(AppEvent::ClearChat).ok();
        }
        "retry" => {
            self.tx.send(AppEvent::RetryLastMessage).ok();
        }
        "remove-last-message" => {
            self.tx.send(AppEvent::RemoveLastMessage).ok();
        }
        "model" => {
            self.tx.send(AppEvent::ShowModelPopup).ok();
        }
        _ => {
            log::warn!("Command not implemented: {}", command);
        }
    }

    self.set_input_mode(InputMode::Normal);
}
```

### Model Selection

The App handles model selection through the `change_model()` method:

```rust
async fn change_model(&mut self, model: &'static LLMModel) {
    if self.is_changing_model {
        return; // Prevent concurrent model changes
    }

    self.is_changing_model = true;

    // Create a new Nixcode instance with the selected model
    if let Ok(()) = self.nixcode.reset().await {
        // Create a new Nixcode client with the new model
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project = Project::new(cwd);
        let mut config = self.nixcode.get_config().clone();

        // Update the config to use the provider of the selected model
        let provider = match model.provider() {
            LLMProvider::Anthropic => "anthropic",
            LLMProvider::OpenAI => "openai",
            LLMProvider::Groq => "groq",
            LLMProvider::OpenRouter => "open_router",
            LLMProvider::Gemini => "gemini",
        };

        // Set the default provider in the config to match the model's provider
        config.llm.default_provider = provider.to_string();

        // Create a new client with the updated config and model
        match Nixcode::new_with_config(project, config) {
            Ok((new_rx, client)) => {
                // Update the client with the new model
                let nixcode = Arc::new(client.with_model(model));

                // Update the current Nixcode instance
                self.nixcode = nixcode.clone();
                self.nixcode_rx = new_rx;

                // Update the chat view with the new Nixcode instance
                self.chat_view.update_nixcode(nixcode);

                // Update chat widgets
                self.chat_view.update_chat_widgets().await;

                log::info!("Model changed to {} with provider {}", model, provider);
            }
            Err(e) => {
                log::error!("Failed to change model: {:?}", e);
            }
        }
    }

    self.is_changing_model = false;
}
```

## Input Modes

**File**: `input_mode.rs`

The InputMode enum represents the different input modes in the application, inspired by Vim.

### Key Structs and Enums

- `InputMode`: Enum representing the different input modes

```rust
#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}
```

### Key Methods

- `to_string()`: Converts the input mode to a string representation

```rust
impl InputMode {
    pub fn to_string(&self) -> &str {
        match self {
            InputMode::Normal => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
        }
    }
}
```

## Command Popup

**File**: `command_popup.rs`

The CommandPopup struct provides a command-line interface for executing commands with command suggestions, tab completion, command history, and error handling.

### Key Structs and Enums

- `CommandInfo`: Struct representing information about a command
- `CommandSuggestion`: Struct representing a command suggestion
- `CommandPopup`: The main command popup struct

### Key Methods

- `new()`: Creates a new CommandPopup instance
- `handle_input_event()`: Handles input events
- `execute_command()`: Executes a command
- `update_suggestions()`: Updates the command suggestions based on the current input
- `complete_suggestion()`: Completes the current input with the selected suggestion
- `next_suggestion()`: Navigates to the next suggestion
- `prev_suggestion()`: Navigates to the previous suggestion
- `render()`: Renders the command popup

### Command Handling

The CommandPopup handles commands through the `execute_command()` method:

```rust
fn execute_command(&mut self) {
    // Get the command to execute (either from selection or input)
    let command_to_execute = match self.selected_suggestion {
        Some(index) => self.suggestions.get(index).map_or_else(
            || self.command.as_string(),
            |suggestion| suggestion.display_name.clone(),
        ),
        None => self.command.as_string(),
    };

    // Check if the command is valid before executing
    if !self.is_valid_command(&command_to_execute) {
        self.command_is_valid = false;
        return;
    }

    // Normalize the command (resolve aliases to primary commands)
    let normalized_command = self.normalize_command(&command_to_execute);

    // Send the command for execution
    if self.tx.send(AppEvent::Command(normalized_command)).is_ok() {
        self.flush_command();
    }
}
```

### Suggestion Handling

The CommandPopup handles suggestions through the `update_suggestions()` method:

```rust
fn update_suggestions(&mut self) {
    self.suggestions.clear();
    let current_input = self.command.as_string().to_lowercase().trim().to_string();

    // Update command validity
    self.command_is_valid = current_input.is_empty() || self.is_valid_command(&current_input);

    // Build suggestions list with matching commands and aliases
    for cmd in AVAILABLE_COMMANDS {
        // Add main command if it matches
        if cmd.name.to_lowercase().starts_with(&current_input) || current_input.is_empty() {
            self.suggestions.push(CommandSuggestion {
                display_name: cmd.name.to_string(),
                description: cmd.description,
                is_alias: false,
                original_command: cmd.name,
            });
        }

        // Add any matching aliases
        for &alias in cmd.aliases {
            if alias.to_lowercase().starts_with(&current_input) || current_input.is_empty() {
                self.suggestions.push(CommandSuggestion {
                    display_name: alias.to_string(),
                    description: cmd.description,
                    is_alias: true,
                    original_command: cmd.name,
                });
            }
        }
    }

    // Sort suggestions - primary commands first, then alphabetically
    self.suggestions
        .sort_by(|a, b| match (a.is_alias, b.is_alias) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.display_name.cmp(&b.display_name),
        });

    // Reset selection if needed
    if self.selected_suggestion.is_some()
        && (self.selected_suggestion.unwrap() >= self.suggestions.len())
    {
        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        }
    }
}
```

## Model Popup

**File**: `model_popup.rs`

The ModelPopup struct allows users to select different LLM models with visual indication of the current model, provider-based color coding, and navigation and selection controls.

### Key Structs and Enums

- `ModelPopup`: The main model popup struct

### Key Methods

- `new()`: Creates a new ModelPopup instance
- `handle_input_event()`: Handles input events
- `render()`: Renders the model popup

### Model Selection

The ModelPopup handles model selection through the `handle_input_event()` method:

```rust
pub fn handle_input_event(&mut self, event: &Event) -> bool {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => {
                    return false; // Close popup without selection
                }
                KeyCode::Enter => {
                    // Select the current model
                    if let Some(&model) = AllModels.get(self.selected_index) {
                        self.tx.send(AppEvent::ChangeModel(model)).ok();
                    }
                    return false; // Close popup after selection
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Move selection down
                    if !AllModels.is_empty() {
                        self.selected_index = (self.selected_index + 1) % AllModels.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    // Move selection up
                    if !AllModels.is_empty() {
                        self.selected_index = if self.selected_index == 0 {
                            AllModels.len() - 1
                        } else {
                            self.selected_index - 1
                        };
                    }
                }
                _ => {}
            }
        }
    }
    true // Keep popup open
}
```

## Chat Widget

**File**: `widgets/chat.rs`

The Chat struct displays the conversation with the LLM and includes message rendering, scrolling, input handling, cost tracking, and status indicators.

### Key Structs and Enums

- `Chat`: The main chat widget struct

### Key Methods

- `new()`: Creates a new Chat instance
- `handle_input_events()`: Handles input events
- `update_chat_widgets()`: Updates the chat widgets with the latest messages
- `send_user_message()`: Sends a user message to the LLM
- `render_frame()`: Renders the chat frame
- `clear_chat()`: Clears the chat history
- `retry_last_message()`: Retries the last message
- `remove_last_message()`: Removes the last message
- `on_error()`: Handles errors

### Message Handling

The Chat widget handles messages through the `send_user_message()` method:

```rust
async fn send_user_message(&mut self) {
    if self.client.is_waiting().await {
        return;
    }

    let message_text = self.prompt.as_string().trim().to_string();
    if message_text.is_empty() {
        return;
    }

    // Create a regular LLM message
    let message = LLMMessage::user()
        .with_text(message_text.clone())
        .to_owned();
    self.prompt.flush();

    self.send_message(Some(message)).await;
}
```

### Scrolling

The Chat widget handles scrolling through the `scroll_up()`, `scroll_down()`, and `scroll_to_bottom()` methods:

```rust
pub fn scroll_up(&mut self) {
    if self.scroll > 0 {
        self.set_vertical_scroll(self.scroll - 1);
        // Only update stick_to_bottom if we're not at the bottom anymore
        self.stick_to_bottom = self.scroll >= self.get_max_scroll();
    }
}

pub fn scroll_down(&mut self) {
    let max_scroll = self.get_max_scroll();
    if self.scroll < max_scroll {
        self.set_vertical_scroll(self.scroll + 1);
        // Check if we reached the bottom
        self.stick_to_bottom = self.scroll >= max_scroll;
    }
}

pub fn scroll_to_bottom(&mut self) {
    let max_scroll = self.get_max_scroll();
    self.set_vertical_scroll(max_scroll);
    self.stick_to_bottom = true;
}
```

## Message Widget

**File**: `widgets/message_widget.rs`

The MessageWidget struct renders individual messages with role-based styling, code block formatting, tool call visualization, and tool result display.

### Key Structs and Enums

- `MessageWidget`: The main message widget struct

### Key Methods

- `get_lines()`: Converts an LLMMessage into a vector of Lines for rendering
- `format_tool_params()`: Formats tool parameters for display

### Message Rendering

The MessageWidget renders messages through the `get_lines()` method:

```rust
pub fn get_lines<'a>(message: LLMMessage) -> Vec<Line<'a>> {
    let author = match message.role.as_str() {
        "user" => Span::styled("You > ", Style::new().green()),
        "assistant" => Span::styled("Assistant > ", Style::new().yellow()),
        "system" | "developer" => Span::styled("System > ", Style::new().dark_gray()),
        _ => Span::styled("Unknown > ", Style::new().red()),
    }
    .bold();

    let mut lines = vec![];

    if let Some(text) = message.reasoning {
        let mut reasoning_lines = vec![];

        for line_str in LinesWithEndings::from(text.as_str()) {
            reasoning_lines.push(
                Line::from(vec![Span::raw(String::from(line_str))])
                    .italic()
                    .gray(),
            );
        }

        lines.extend(reasoning_lines);
    }

    if let Some(text) = message.text {
        lines.extend(format_text(text, Some(author.clone())));
    }

    if let Some(tool_calls) = message.tool_calls {
        for tool_call in tool_calls {
            let (name, params) = tool_call.get_execute_params();
            let formatted_params = Self::format_tool_params(&params);
            let tool_info = format!("{}({})", name, formatted_params);

            lines.push(Line::from(tool_info).bold());
        }
    }

    if let Some(tools_results) = message.tool_results {
        for tool_result in tools_results {
            let x = tool_result
                .call_id
                .unwrap_or_else(|| "unknown id".to_string());
            let content = tool_result.result;
            let split_iterator = content.split("\n");
            let total_lines = split_iterator.clone().count();
            let mut lines2 = vec![Line::from(Span::raw(format!("[{}] ", x)).bold())];

            split_iterator
                .take(5)
                .for_each(|line| lines2.push(Line::from(String::from(line))));

            let missing_lines = total_lines.saturating_sub(5);
            if missing_lines > 0 {
                lines2.push(Line::from(format!("... {} more lines", missing_lines)).italic());
            }

            lines2.push(Line::from(vec![]));
            lines.extend(lines2);
        }
    }

    lines
}
```

## Status Bar

**File**: `status_bar.rs`

The StatusBar struct displays the current input mode, model, provider, application version, and current date and time.

### Key Structs and Enums

- `StatusBar`: The main status bar struct

### Key Methods

- `new()`: Creates a new StatusBar instance
- `with_model()`: Sets the current model
- `render()`: Renders the status bar

### Status Bar Rendering

The StatusBar renders the status bar through the `render()` method:

```rust
fn render(self, area: Rect, buf: &mut Buffer)
where
    Self: Sized,
{
    // Get application version from Cargo.toml
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let now = chrono::Local::now();
    let formatted_date = now.format("%d/%m/%Y %H:%M:%S").to_string();
    let version_text = format!("v{}", VERSION);

    // Calculate total length of the right side content (date + version)
    let right_content_length = formatted_date.len() + 1 + version_text.len();
    // Render the mode info on the left
    let mode_line = Line::from(vec![
        Span::raw("Mode: "),
        Span::styled(
            format!(" {} ", self.current_mode.to_string()),
            Style::new()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let mode_line_width = mode_line.width() + 1;

    // Create layout with three sections or two sections based on model info availability
    let horizontal = if self.current_model.is_some() {
        Layout::horizontal([
            Length(mode_line_width as u16),      // Mode info fixed width
            Fill(1),                             // Model info takes remaining space
            Length(right_content_length as u16), // Date+version fixed width
        ])
    } else {
        Layout::horizontal([
            Fill(1),                             // Mode info takes available space
            Length(right_content_length as u16), // Date+version fixed width
        ])
    };

    let inner_margin = area.inner(Margin::new(1, 0));
    let areas = if self.current_model.is_some() {
        let areas: [Rect; 3] = horizontal.areas(inner_margin);
        areas
    } else {
        let areas: [Rect; 2] = horizontal.areas(inner_margin);
        [areas[0], areas[1], Rect::default()] // Add a dummy third area
    };

    Block::new().bg(Color::DarkGray).render(area, buf);

    mode_line.render(areas[0], buf);

    // Render model info in the middle if available
    if let Some(model) = self.current_model {
        let provider_color = match model.provider() {
            LLMProvider::Anthropic => Color::Rgb(163, 77, 253), // Purple
            LLMProvider::OpenAI => Color::Rgb(16, 163, 127),    // Green
            LLMProvider::Groq => Color::Rgb(255, 165, 0),       // Orange
            LLMProvider::OpenRouter => Color::Rgb(59, 130, 246), // Blue
            LLMProvider::Gemini => Color::Rgb(234, 67, 53),     // Red
        };

        let model_line = Line::from(vec![
            Span::raw("Model: "),
            Span::styled(
                format!(" {} ", model.provider().name()),
                Style::new()
                    .fg(Color::Black)
                    .bg(provider_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{}", model),
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]);

        model_line.render(areas[1], buf);

        // Render date and version on the right (third area)
        Line::from(vec![
            Span::styled(
                version_text,
                Style::new()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::from(formatted_date),
        ])
        .render(areas[2], buf);
    } else {
        // If no model info, render date and version in the second area
        Line::from(vec![
            Span::styled(
                version_text,
                Style::new()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::from(formatted_date),
        ])
        .render(areas[1], buf);
    }
}
```

## User Input

**File**: `user_input.rs`

The UserSingleLineInput struct provides a single-line text input widget with cursor movement, text editing, Unicode support, and horizontal scrolling.

### Key Structs and Enums

- `UserSingleLineInput`: The main user input struct

### Key Methods

- `new()`: Creates a new UserSingleLineInput instance
- `as_string()`: Returns the current text as a string
- `insert()`: Inserts a character at the cursor position
- `handle_backspace()`: Handles backspace key
- `handle_delete()`: Handles delete key
- `flush()`: Clears the input
- `move_cursor()`: Moves the cursor
- `handle_input_events()`: Handles input events
- `get_cursor_position()`: Returns the cursor position
- `render()`: Renders the input widget

### Input Handling

The UserSingleLineInput handles input through the `handle_input_events()` method:

```rust
pub fn handle_input_events(&mut self, event: &Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            crossterm::event::KeyCode::Left => self.move_cursor(-1),
            crossterm::event::KeyCode::Right => self.move_cursor(1),
            crossterm::event::KeyCode::Home => {
                self.cursor = 0;
                self.cursor_byte = 0;
                self.scroll_offset = 0;
            }
            crossterm::event::KeyCode::End => {
                self.cursor = self.data.chars().count();
                self.cursor_byte = self.data.len();
                self.adjust_scroll_offset();
            }
            crossterm::event::KeyCode::Backspace => self.handle_backspace(),
            crossterm::event::KeyCode::Delete => self.handle_delete(),
            crossterm::event::KeyCode::Char(c) => self.insert(c),
            _ => (),
        },
        _ => (),
    }
}
```

## Popup Utilities

**File**: `popup_utils.rs`

The popup utilities module provides helper functions for creating and positioning popup dialogs in the UI.

### Key Functions

- `popup_area()`: Creates a popup area with a specified width percentage
- `centered_rect()`: Creates a centered rectangle with specified width and height

```rust
pub fn popup_area(area: Rect, percent_x: u16) -> Rect {
    let vertical = Layout::vertical([Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Helper function to create a centered rect using up certain percentage of the available rect
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Length((r.height.saturating_sub(height)) / 2),
        Constraint::Length(height),
        Constraint::Length((r.height.saturating_sub(height)) / 2),
    ])
    .flex(Flex::Center)
    .split(r);

    Layout::horizontal([
        Constraint::Length((r.width.saturating_sub(width)) / 2),
        Constraint::Length(width),
        Constraint::Length((r.width.saturating_sub(width)) / 2),
    ])
    .flex(Flex::Center)
    .split(popup_layout[1])[1]
}
```

## Syntax Highlighting

**File**: `utils/highlights.rs`

The syntax highlighting module provides code block highlighting using the syntect library.

### Key Functions

- `highlight_code()`: Highlights code using the syntect library
- `syntect_style_to_ratatui()`: Converts syntect styles to ratatui styles

```rust
pub fn highlight_code<'a>(
    code: String,
    extension: &str,
) -> Result<Vec<Line<'a>>, Box<dyn std::error::Error>> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme = &THEME;

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line_str in LinesWithEndings::from(code.as_str()) {
        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line_str, &SYNTAX_SET)?;

        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(syntect_style, segment)| {
                let ratatui_style = syntect_style_to_ratatui(syntect_style);
                Span::styled(segment.to_string(), ratatui_style)
            })
            .collect();

        lines.push(Line::from(spans));
    }

    Ok(lines)
}
```