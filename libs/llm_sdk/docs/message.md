# Message API Documentation

The Message module defines the structure and handling of messages exchanged with LLM providers. It provides a unified interface for different message types and content formats.

## Table of Contents

1. [Overview](#overview)
2. [Message Structure](#message-structure)
3. [Content Types](#content-types)
   - [Text Content](#text-content)
   - [Image Content](#image-content)
   - [Thinking Content](#thinking-content)
   - [Tool Content](#tool-content)
4. [Message Delta](#message-delta)
5. [Provider-Specific Implementations](#provider-specific-implementations)
6. [Usage Tracking](#usage-tracking)
7. [Response Handling](#response-handling)

## Overview

The message module is organized as follows:

```
message/
├── anthropic/
│   ├── events.rs
│   ├── mod.rs
│   └── tokens.rs
├── common/
│   ├── llm_message.rs
│   └── mod.rs
├── content/
│   ├── image.rs
│   ├── image_source.rs
│   ├── mod.rs
│   ├── text.rs
│   ├── thinking.rs
│   └── tools.rs
├── message.rs
├── mod.rs
├── openai/
│   ├── events.rs
│   ├── mod.rs
│   └── tokens.rs
├── response.rs
└── usage.rs
```

The module provides:
- A unified message structure for different providers
- Different content types (text, image, thinking, tools)
- Provider-specific message handling
- Usage tracking for token consumption

## Message Structure

The `Message` enum represents a message in a conversation:

```rust
pub enum Message {
    User(Contents),
    Assistant(Contents),
}
```

Where `Contents` is a vector of `Content` items.

### Methods

- `get_content() -> Contents`: Returns the content of the message
- `get_content_mut() -> &mut Contents`: Returns a mutable reference to the content
- `set_content(new_content: Contents)`: Sets the content of the message

### LLMMessage

The `LLMMessage` struct is used for internal representation of messages:

```rust
pub struct LLMMessage {
    pub role: String,
    pub text: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}
```

### LLMRequest

The `LLMRequest` struct represents a request to an LLM provider:

```rust
pub struct LLMRequest {
    pub model: LLMModel,
    pub messages: Vec<LLMMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub thinking: Option<ThinkingOptions>,
    pub cache_control: Option<CacheControl>,
}
```

## Content Types

The `Content` enum represents different types of content in a message:

```rust
pub enum Content {
    Empty,
    Text(TextContent),
    Image(ImageContent),
    Thinking(ThinkingContent),
    RedactedThinking(RedactedThinkingContent),
    ToolUse(ToolUseContent),
    ToolResult(ToolResultContent),
}
```

### Methods

- `is_text() -> bool`: Checks if the content is text
- `get_text() -> Option<TextContent>`: Gets the text content if available
- `extend_text(new_text: impl Into<String>)`: Extends the text content
- `is_tool_use() -> bool`: Checks if the content is a tool use
- `validate_content() -> bool`: Validates the content
- `extend_delta(delta: ContentDelta)`: Extends the content with a delta

### Factory Methods

- `new_text(text: impl Into<String>) -> Self`: Creates new text content
- `new_tool_result(result: ToolResultContent) -> Self`: Creates new tool result content
- `new_tool_use(tool_use: ToolUseContent) -> Self`: Creates new tool use content
- `new_tool_results(results: Vec<ToolResultContent>) -> Vec<Content>`: Creates multiple tool result contents
- `new_reasoning(text: impl Into<String>) -> Self`: Creates new reasoning content

### Text Content

The `TextContent` struct represents plain text content:

```rust
pub struct TextContent {
    pub text: String,
}
```

### Image Content

The `ImageContent` struct represents image content:

```rust
pub struct ImageContent {
    pub source: ImageSource,
}
```

Where `ImageSource` can be:

```rust
pub enum ImageSource {
    Url(String),
    Base64(String),
}
```

### Thinking Content

The `ThinkingContent` struct represents reasoning or thinking content:

```rust
pub struct ThinkingContent {
    pub thinking: String,
    pub signature: String,
}
```

### Tool Content

The `ToolUseContent` struct represents a tool/function call:

```rust
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: Value,
}
```

The `ToolResultContent` struct represents the result of a tool/function call:

```rust
pub struct ToolResultContent {
    pub tool_call_id: String,
    pub result: Value,
}
```

## Message Delta

The `MessageDelta` struct represents a delta update to a message:

```rust
pub struct MessageDelta {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}
```

The `ContentDelta` enum represents a delta update to content:

```rust
pub enum ContentDelta {
    TextDelta(ContentTextDelta),
    ThinkingDelta(ContentThinkingDelta),
    SignatureDelta(ContentSignatureDelta),
    InputJsonDelta(ContentInputJsonDelta),
}
```

## Provider-Specific Implementations

### Anthropic

The `anthropic` module provides Anthropic-specific message handling:

- `events.rs`: Defines Anthropic event types
- `tokens.rs`: Handles token counting for Anthropic

### OpenAI

The `openai` module provides OpenAI-specific message handling:

- `events.rs`: Defines OpenAI event types
- `tokens.rs`: Handles token counting for OpenAI

## Usage Tracking

The `usage.rs` module defines structures for tracking token usage:

```rust
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_reads: Option<u32>,
    pub cache_writes: Option<u32>,
}
```

Provider-specific usage structures are also defined:

```rust
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## Response Handling

The `response.rs` module defines structures for handling LLM responses:

```rust
pub struct Response {
    pub content: String,
}
```

The `LLMEvent` enum represents events from an LLM provider:

```rust
pub enum LLMEvent {
    PartialContent(usize, Content),
    Content(Content),
    Stop(Option<StopReason>),
}
```

## Usage Example

Here's an example of creating and using messages:

```rust
use llm_sdk::message::message::{Message, Contents};
use llm_sdk::message::content::{Content, TextContent};
use llm_sdk::message::common::llm_message::LLMMessage;

// Create a user message with text content
let user_message = Message::User(vec![
    Content::Text(TextContent { text: "Hello, world!".to_string() }),
]);

// Get the content of the message
let contents = user_message.get_content();
for content in contents {
    if let Content::Text(text) = content {
        println!("User said: {}", text.text);
    }
}

// Create an LLMMessage from a Message
let llm_message = LLMMessage {
    role: "user".to_string(),
    text: Some("Hello, world!".to_string()),
    tool_calls: None,
    tool_results: None,
};

// Convert LLMMessage back to Message
let message = Message::try_from(&llm_message).unwrap();
```