# Events

The events module defines the event system used for communication between components in the nixcode-ai application.

## Overview

The event system is based on Rust's `tokio::sync::mpsc` channel system, allowing for asynchronous communication between components. Events are dispatched through a channel and can be received and processed by any component that has access to the receiver end of the channel.

## NixcodeEvent Enum

The `NixcodeEvent` enum defines the types of events that can be dispatched:

```rust
#[derive(Debug)]
pub enum NixcodeEvent {
    GeneratingResponse,
    GeneratedResponse,
    NewMessage,
    MessageUpdated,
    Error(LLMError),
    ToolStart(ToolCall),
    ToolEnd(ToolResult),
    ToolsFinished,
}
```

### Event Types

- `GeneratingResponse`: Indicates that the application is generating a response from the LLM
- `GeneratedResponse`: Indicates that the application has finished generating a response from the LLM
- `NewMessage`: Indicates that a new message has been added to the conversation
- `MessageUpdated`: Indicates that a message has been updated (typically during streaming)
- `Error(LLMError)`: Indicates that an error has occurred during LLM communication
- `ToolStart(ToolCall)`: Indicates that a tool execution has started
- `ToolEnd(ToolResult)`: Indicates that a tool execution has completed
- `ToolsFinished`: Indicates that all tool executions have completed

## Event Flow

Events are dispatched by the `Nixcode` instance through the `tx` channel and can be received by any component that has access to the `rx` channel.

```rust
// Dispatch an event
self.tx.send(NixcodeEvent::NewMessage).ok();

// Receive events
while let Some(event) = rx.recv().await {
    match event {
        NixcodeEvent::NewMessage => {
            // Handle new message
        }
        NixcodeEvent::MessageUpdated => {
            // Handle message update
        }
        // Handle other events
        _ => {}
    }
}
```

## Event Handling

Events are typically handled in the main application loop, which processes events from the `rx` channel and updates the UI accordingly.

### Example Event Handling

```rust
// Create a new Nixcode instance
let (rx, nixcode) = Nixcode::new_from_env(project).unwrap();
let nixcode = Arc::new(nixcode);

// Process events from the LLM
while let Some(event) = rx.recv().await {
    match event {
        NixcodeEvent::NewMessage => {
            // Update UI with new message
        }
        NixcodeEvent::MessageUpdated => {
            // Update UI with updated message
        }
        NixcodeEvent::ToolsFinished => {
            // Handle tool execution completion
            nixcode.clone().send_tools_results().await;
        }
        NixcodeEvent::Error(err) => {
            // Handle error
            println!("Error: {:?}", err);
        }
        // Handle other events
        _ => {}
    }
}
```

## Event Dispatch Points

Events are dispatched at various points in the application:

- `GeneratingResponse`: When the application starts generating a response from the LLM
- `GeneratedResponse`: When the application finishes generating a response from the LLM
- `NewMessage`: When a new message is added to the conversation
- `MessageUpdated`: When a message is updated during streaming
- `Error`: When an error occurs during LLM communication
- `ToolStart`: When a tool execution starts
- `ToolEnd`: When a tool execution completes
- `ToolsFinished`: When all tool executions have completed

## Integration with Nixcode

The `Nixcode` struct includes a `tx` field of type `UnboundedSender<NixcodeEvent>`, which is used to dispatch events. The `new` method returns a tuple of `(UnboundedReceiver<NixcodeEvent>, Nixcode)`, allowing the caller to receive events from the `Nixcode` instance.

```rust
pub type NewNixcodeResult = (UnboundedReceiver<NixcodeEvent>, Nixcode);

impl Nixcode {
    pub fn new(
        project: Project,
        client: LLMClient,
        config: Config,
    ) -> Result<NewNixcodeResult, LLMError> {
        let (tx, rx) = unbounded_channel::<NixcodeEvent>();
        let nixcode = Self {
            // ...
            tx,
            // ...
        };

        Ok((rx, nixcode))
    }
}
```