# nixcode-cli API Reference

This document provides a detailed API reference for the nixcode-cli application.

## Table of Contents

1. [App](#app)
2. [InputMode](#inputmode)
3. [CommandPopup](#commandpopup)
4. [ModelPopup](#modelpopup)
5. [Chat](#chat)
6. [MessageWidget](#messagewidget)
7. [StatusBar](#statusbar)
8. [UserSingleLineInput](#usersinglelineinput)
9. [PopupUtils](#popuputils)
10. [Highlights](#highlights)

## App

**File**: `app.rs`

### Structs and Enums

#### `AppEvent`

```rust
pub enum AppEvent {
    SetInputMode(InputMode),
    Command(String),
    UpdateChatWidgets,
    RetryLastMessage,
    RemoveLastMessage,
    ClearChat,
    ShowModelPopup,
    ChangeModel(&'static LLMModel),
    Quit,
    Render,
    ChatError(ErrorEventContent),
}
```

#### `AppView`

```rust
enum AppView {
    Chat,
}
```

#### `App`

```rust
pub struct App {
    should_quit: bool,
    chat_view: Chat,
    current_view: AppView,
    input_mode: InputMode,
    rx: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    nixcode_rx: tokio::sync::mpsc::UnboundedReceiver<NixcodeEvent>,
    nixcode: Arc<Nixcode>,
    command_popup: CommandPopup,
    model_popup: Option<ModelPopup>,
    is_changing_model: bool,
}
```

### Methods

#### `new()`

```rust
pub(crate) fn new(nixcode: NewNixcodeResult) -> Result<Self>
```

Creates a new App instance.

#### `run()`

```rust
pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> tokio::io::Result<()>
```

Runs the main event loop.

#### `handle_input_events()`

```rust
async fn handle_input_events(&mut self, event: Event)
```

Handles input events from the terminal.

#### `handle_app_event()`

```rust
async fn handle_app_event(&mut self, event: AppEvent)
```

Handles application events.

#### `handle_nixcode_event()`

```rust
async fn handle_nixcode_event(&mut self, event: NixcodeEvent)
```

Handles events from the Nixcode client.

#### `draw_frame()`

```rust
fn draw_frame(&mut self, frame: &mut Frame)
```

Renders the UI.

#### `execute_command()`

```rust
async fn execute_command(&mut self, command: String)
```

Executes a command.

#### `change_model()`

```rust
async fn change_model(&mut self, model: &'static LLMModel)
```

Changes the LLM model.

#### `quit()`

```rust
fn quit(&mut self)
```

Exits the application.

## InputMode

**File**: `input_mode.rs`

### Structs and Enums

#### `InputMode`

```rust
#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}
```

### Methods

#### `to_string()`

```rust
pub fn to_string(&self) -> &str
```

Converts the input mode to a string representation.

## CommandPopup

**File**: `command_popup.rs`

### Structs and Enums

#### `CommandInfo`

```rust
struct CommandInfo {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
}
```

#### `CommandSuggestion`

```rust
struct CommandSuggestion {
    display_name: String,
    description: &'static str,
    is_alias: bool,
    original_command: &'static str,
}
```

#### `CommandPopup`

```rust
pub struct CommandPopup {
    command: UserSingleLineInput,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    suggestions: Vec<CommandSuggestion>,
    selected_suggestion: Option<usize>,
    command_is_valid: bool,
}
```

### Methods

#### `new()`

```rust
pub(crate) fn new(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> Self
```

Creates a new CommandPopup instance.

#### `handle_input_event()`

```rust
pub(crate) fn handle_input_event(&mut self, event: &Event)
```

Handles input events.

#### `execute_command()`

```rust
fn execute_command(&mut self)
```

Executes a command.

#### `update_suggestions()`

```rust
fn update_suggestions(&mut self)
```

Updates the command suggestions based on the current input.

#### `complete_suggestion()`

```rust
fn complete_suggestion(&mut self)
```

Completes the current input with the selected suggestion.

#### `next_suggestion()`

```rust
fn next_suggestion(&mut self)
```

Navigates to the next suggestion.

#### `prev_suggestion()`

```rust
fn prev_suggestion(&mut self)
```

Navigates to the previous suggestion.

#### `render()`

```rust
fn render(self, area: Rect, buf: &mut Buffer)
```

Renders the command popup.

## ModelPopup

**File**: `model_popup.rs`

### Structs and Enums

#### `ModelPopup`

```rust
pub struct ModelPopup {
    tx: UnboundedSender<AppEvent>,
    selected_index: usize,
    current_model: &'static LLMModel,
}
```

### Methods

#### `new()`

```rust
pub fn new(tx: UnboundedSender<AppEvent>, current_model: &'static LLMModel) -> Self
```

Creates a new ModelPopup instance.

#### `handle_input_event()`

```rust
pub fn handle_input_event(&mut self, event: &Event) -> bool
```

Handles input events. Returns true if the popup should remain open, false otherwise.

#### `render()`

```rust
fn render(self, area: Rect, buf: &mut Buffer)
```

Renders the model popup.

## Chat

**File**: `widgets/chat.rs`

### Structs and Enums

#### `Chat`

```rust
pub struct Chat {
    vertical_scroll_state: ScrollbarState,
    lines: Vec<Line<'static>>,
    paragraph: Paragraph<'static>,
    client: Arc<Nixcode>,
    input_mode: InputMode,
    app_event: UnboundedSender<AppEvent>,
    prompt: UserSingleLineInput,
    area_size: (u16, u16),
    stick_to_bottom: bool,
    scroll: usize,
    total_lines: usize,
    usage: AnthropicUsage,
    waiting: bool,
    error: Option<ErrorEventContent>,
    total_cost: f64,
}
```

### Methods

#### `new()`

```rust
pub fn new(
    client: Arc<Nixcode>,
    input_mode: InputMode,
    app_event: UnboundedSender<AppEvent>,
) -> Self
```

Creates a new Chat instance.

#### `set_input_mode()`

```rust
pub fn set_input_mode(&mut self, mode: InputMode)
```

Sets the input mode.

#### `update_nixcode()`

```rust
pub fn update_nixcode(&mut self, client: Arc<Nixcode>)
```

Updates the Nixcode client.

#### `handle_input_events()`

```rust
pub async fn handle_input_events(&mut self, input_mode: InputMode, event: &Event)
```

Handles input events.

#### `update_chat_widgets()`

```rust
pub async fn update_chat_widgets(&mut self)
```

Updates the chat widgets with the latest messages.

#### `set_vertical_scroll()`

```rust
pub fn set_vertical_scroll(&mut self, scroll: usize)
```

Sets the vertical scroll position.

#### `scroll_up()`

```rust
pub fn scroll_up(&mut self)
```

Scrolls up in the chat.

#### `scroll_down()`

```rust
pub fn scroll_down(&mut self)
```

Scrolls down in the chat.

#### `scroll_to_bottom()`

```rust
pub fn scroll_to_bottom(&mut self)
```

Scrolls to the bottom of the chat.

#### `send_user_message()`

```rust
async fn send_user_message(&mut self)
```

Sends a user message to the LLM.

#### `render_frame()`

```rust
pub fn render_frame(&mut self, frame: &mut Frame, area: Rect)
```

Renders the chat frame.

#### `clear_chat()`

```rust
pub async fn clear_chat(&mut self)
```

Clears the chat history.

#### `retry_last_message()`

```rust
pub async fn retry_last_message(&mut self)
```

Retries the last message.

#### `remove_last_message()`

```rust
pub async fn remove_last_message(&mut self)
```

Removes the last message.

#### `on_error()`

```rust
pub async fn on_error(&mut self, error: ErrorEventContent)
```

Handles errors.

## MessageWidget

**File**: `widgets/message_widget.rs`

### Structs and Enums

#### `MessageWidget`

```rust
pub struct MessageWidget {}
```

### Methods

#### `get_lines()`

```rust
pub fn get_lines<'a>(message: LLMMessage) -> Vec<Line<'a>>
```

Converts an LLMMessage into a vector of Lines for rendering.

#### `format_tool_params()`

```rust
fn format_tool_params(params: &Value) -> String
```

Formats tool parameters for display.

## StatusBar

**File**: `status_bar.rs`

### Structs and Enums

#### `StatusBar`

```rust
pub struct StatusBar {
    current_mode: InputMode,
    current_model: Option<&'static LLMModel>,
}
```

### Methods

#### `new()`

```rust
pub(crate) fn new(status: InputMode) -> Self
```

Creates a new StatusBar instance.

#### `with_model()`

```rust
pub(crate) fn with_model(mut self, model: &'static LLMModel) -> Self
```

Sets the current model.

#### `render()`

```rust
fn render(self, area: Rect, buf: &mut Buffer)
```

Renders the status bar.

## UserSingleLineInput

**File**: `user_input.rs`

### Structs and Enums

#### `UserSingleLineInput`

```rust
pub struct UserSingleLineInput {
    data: String,
    cursor: usize,
    cursor_byte: usize,
    scroll_offset: usize,
    last_area_width: u16,
}
```

### Methods

#### `new()`

```rust
pub fn new(data: String) -> Self
```

Creates a new UserSingleLineInput instance.

#### `as_string()`

```rust
pub fn as_string(&self) -> String
```

Returns the current text as a string.

#### `insert()`

```rust
pub fn insert(&mut self, c: char)
```

Inserts a character at the cursor position.

#### `handle_backspace()`

```rust
pub fn handle_backspace(&mut self)
```

Handles backspace key.

#### `handle_delete()`

```rust
pub fn handle_delete(&mut self)
```

Handles delete key.

#### `flush()`

```rust
pub fn flush(&mut self)
```

Clears the input.

#### `move_cursor()`

```rust
pub fn move_cursor(&mut self, offset: i16)
```

Moves the cursor.

#### `handle_input_events()`

```rust
pub fn handle_input_events(&mut self, event: &Event)
```

Handles input events.

#### `get_cursor_position()`

```rust
pub fn get_cursor_position(&self, area: Rect) -> (u16, u16)
```

Returns the cursor position.

#### `render()`

```rust
fn render(self, area: Rect, buf: &mut Buffer)
```

Renders the input widget.

## PopupUtils

**File**: `popup_utils.rs`

### Functions

#### `popup_area()`

```rust
pub fn popup_area(area: Rect, percent_x: u16) -> Rect
```

Creates a popup area with a specified width percentage.

#### `centered_rect()`

```rust
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect
```

Creates a centered rectangle with specified width and height.

## Highlights

**File**: `utils/highlights.rs`

### Functions

#### `highlight_code()`

```rust
pub fn highlight_code<'a>(
    code: String,
    extension: &str,
) -> Result<Vec<Line<'a>>, Box<dyn std::error::Error>>
```

Highlights code using the syntect library.

#### `syntect_style_to_ratatui()`

```rust
fn syntect_style_to_ratatui(syntect_style: SyntectStyle) -> Style
```

Converts syntect styles to ratatui styles.