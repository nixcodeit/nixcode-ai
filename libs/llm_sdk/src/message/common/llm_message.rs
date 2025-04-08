use crate::errors::llm::LLMError;
use crate::message::content::tools::{ToolResultContent, ToolUseContent};
use crate::message::openai::{OpenAIMessageToolCall, OpenAIResponse, OpenAIUsage};
use crate::message::response::MessageResponse;
use crate::message::usage::AnthropicUsage;
use crate::models::llm_model::LLMModel;
use crate::stop_reason::StopReason;
use crate::tools::Tool;
use serde_json::Value;

/// Common message structure for all LLM providers
#[derive(Clone, Default, Debug)]
pub struct LLMMessage {
    /// The role of the message (system, user, assistant)
    pub role: String,
    /// Main text response
    pub text: Option<String>,
    /// Reasoning or thinking content
    pub reasoning: Option<String>,
    /// Tool/function calls
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool/function results
    pub tool_results: Option<Vec<ToolResult>>,
    /// Media content (images, etc.)
    pub media: Option<Vec<Media>>,
    /// Reason why the message generation stopped
    pub stop_reason: Option<StopReason>,
    /// Usage statistics (tokens, etc.)
    pub usage: Option<Usage>,
}

impl LLMMessage {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            ..Default::default()
        }
    }

    pub fn with_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.text = Some(text.into());
        self
    }

    pub fn with_reasoning(&mut self, reasoning: impl Into<String>) -> &mut Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    pub fn with_tool_calls(&mut self, tool_call: Vec<ToolCall>) -> &mut Self {
        self.tool_calls = Some(tool_call);
        self
    }

    pub fn with_tool_results(&mut self, tool_result: Vec<ToolResult>) -> &mut Self {
        self.tool_results = Some(tool_result);
        self
    }

    pub fn with_media(&mut self, media: Vec<Media>) -> &mut Self {
        self.media = Some(media);
        self
    }

    pub fn user() -> Self {
        LLMMessage::new("user")
    }

    pub fn assistant() -> Self {
        LLMMessage::new("assistant")
    }

    pub fn system() -> Self {
        LLMMessage::new("system")
    }

    pub fn add_tool_result(&mut self, tool_result: ToolResult) -> &mut Self {
        if let Some(tool_results) = &mut self.tool_results {
            tool_results.push(tool_result);
        } else {
            self.tool_results = Some(vec![tool_result]);
        }

        self
    }

    pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
        self.usage = Some(usage);
        self
    }
}

impl LLMMessage {
    pub fn is_empty(&self) -> bool {
        let is_empty_text = self.text.as_ref().map_or(true, |text| text.is_empty());
        let is_empty_tool_calls = self
            .tool_calls
            .as_ref()
            .map_or(true, |calls| calls.is_empty());
        let is_empty_tool_results = self
            .tool_results
            .as_ref()
            .map_or(true, |results| results.is_empty());
        let is_empty_media = self.media.as_ref().map_or(true, |media| media.is_empty());

        is_empty_text && is_empty_tool_calls && is_empty_tool_results && is_empty_media
    }
}

/// Usage information for token tracking
#[derive(Clone, Debug, Default)]
pub struct Usage {
    /// Number of tokes written to cache
    pub cache_writes: Option<usize>,
    /// Number of tokens read from cache
    pub cache_reads: Option<usize>,
    /// Number of input tokens
    pub input_tokens: usize,
    /// Number of output tokens
    pub output_tokens: usize,
    /// Total tokens (input + output)
    pub total_tokens: usize,
    /// Estimated cost of the request
    pub cost: f64,
}

impl From<AnthropicUsage> for Usage {
    fn from(value: AnthropicUsage) -> Self {
        let cache_writes = value.cache_creation_input_tokens.unwrap_or(0);
        let cache_reads = value.cache_read_input_tokens.unwrap_or(0);
        let input_tokens = value.input_tokens;
        let output_tokens = value.output_tokens;

        Usage {
            cache_writes: Some(cache_writes),
            cache_reads: Some(cache_reads),
            input_tokens: value.input_tokens,
            output_tokens: value.output_tokens,
            total_tokens: input_tokens + output_tokens + cache_writes + cache_reads,
            cost: 0.0,
        }
    }
}

/// Tool/function call structure
#[derive(Clone, Debug)]
pub struct ToolCall {
    /// Name of the tool/function
    pub name: String,
    /// Arguments for the tool (as JSON string)
    pub arguments: String,
    /// Unique identifier for the tool call
    pub id: Option<String>,
}

impl From<OpenAIMessageToolCall> for ToolCall {
    fn from(call: OpenAIMessageToolCall) -> Self {
        ToolCall {
            name: call.function.name,
            arguments: call.function.arguments,
            id: Some(call.id),
        }
    }
}

impl TryInto<ToolUseContent> for ToolCall {
    type Error = LLMError;

    fn try_into(self) -> Result<ToolUseContent, Self::Error> {
        let id = self.id.clone().unwrap_or_default();
        let name = self.name.clone();
        let arguments = serde_json::from_str(&self.arguments)
            .map_err(|_| LLMError::Generic("Failed to parse tool call arguments".to_string()))?;

        Ok(ToolUseContent {
            id,
            name,
            input: arguments,
            _input_raw: self.arguments,
        })
    }
}

impl Into<ToolCall> for ToolUseContent {
    fn into(self) -> ToolCall {
        let arguments = if self._input_raw.is_empty() {
            "{}".into()
        } else {
            self._input_raw
        };

        ToolCall {
            name: self.name,
            arguments,
            id: Some(self.id),
        }
    }
}

impl ToolCall {
    pub fn get_execute_params(&self) -> (String, Value) {
        let name = self.name.clone();
        let args = serde_json::from_str(&self.arguments).unwrap_or(Value::Null);
        (name, args)
    }

    pub fn create_response(&self, result: String) -> ToolResult {
        ToolResult {
            result,
            call_id: self.id.clone(),
        }
    }
}

/// Tool result structure
#[derive(Clone, Debug)]
pub struct ToolResult {
    /// Result of the tool execution (as JSON string)
    pub result: String,
    /// Unique identifier for the tool call (to match with a ToolCall)
    pub call_id: Option<String>,
}

impl TryInto<ToolResultContent> for ToolResult {
    type Error = LLMError;

    fn try_into(self) -> Result<ToolResultContent, Self::Error> {
        let tool_use_id = self.call_id.unwrap_or(String::new());
        let content = self.result.clone();

        Ok(ToolResultContent {
            content,
            tool_use_id,
        })
    }
}

impl Into<ToolResult> for ToolResultContent {
    fn into(self) -> ToolResult {
        ToolResult {
            result: self.get_content(),
            call_id: Some(self.get_tool_use_id()),
        }
    }
}

/// Media content structure
#[derive(Clone, Debug)]
pub struct Media {
    /// Type of media (image, audio, etc.)
    pub media_type: String,
    /// URL or base64 content of the media
    pub content: String,
    /// Additional metadata about the media
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
/// Request structure for LLM API calls
pub struct LLMRequest {
    /// The model to use for this request
    pub model: &'static LLMModel,
    /// Messages to send to the LLM
    pub messages: Vec<LLMMessage>,
    /// Optional system prompt
    pub system: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature for randomness
    pub temperature: Option<f32>,
    /// Tools available to the model
    pub tools: Option<Vec<Tool>>,
    /// Whether to stream the response
    pub stream: bool,
    /// Provider-specific parameters
    pub provider_params: Option<Value>,
    // Stop sequences for the model
    pub stop_sequences: Option<Vec<String>>,
}

/// Common event structure for all LLM providers
#[derive(Debug)]
pub enum LLMEvent {
    /// Message started
    MessageStart,
    /// Message updated
    MessageUpdate(LLMMessage),
    /// Message completed
    MessageComplete,
    /// Error occurred
    Error(LLMError),
}

impl From<&MessageResponse> for LLMMessage {
    fn from(response: &MessageResponse) -> Self {
        let text = response.get_text();
        let text = if text.is_empty() { None } else { Some(text) };
        let reasoning = response.get_reasoning();
        let reasoning = if reasoning.is_empty() {
            None
        } else {
            Some(reasoning)
        };

        let tool_calls = response
            .get_tool_calls()
            .iter()
            .map(|call| call.clone().into())
            .collect::<Vec<ToolCall>>();

        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        let tool_results = response
            .get_tool_results()
            .iter()
            .map(|result| result.clone().into())
            .collect::<Vec<ToolResult>>();

        let tool_results = if tool_results.is_empty() {
            None
        } else {
            Some(tool_results)
        };

        LLMMessage {
            role: response.role.clone(),
            text,
            reasoning,
            tool_calls,
            tool_results,
            media: None,
            stop_reason: response.stop_reason.clone(),
            usage: None,
        }
    }
}

impl From<OpenAIResponse> for LLMMessage {
    fn from(response: OpenAIResponse) -> Self {
        let stop_reason = response
            .choices
            .first()
            .and_then(|choice| choice.finish_reason.clone())
            .and_then(|reason| match reason.as_str() {
                "stop" | "content_filter" => Some(StopReason::EndTurn),
                "length" => Some(StopReason::MaxTokens),
                "function_call" => Some(StopReason::ToolUse),
                _ => None,
            });
        let reasoning = response
            .choices
            .first()
            .map(|choice| choice.message.reasoning.clone());
        let text = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone());
        let tool_calls = response.choices.first().and_then(|choice| {
            Some(
                choice
                    .message
                    .tool_calls
                    .iter()
                    .map(|call| call.clone().into())
                    .collect::<Vec<ToolCall>>(),
            )
        });
        let tool_results = None;

        LLMMessage {
            role: "assistant".to_string(),
            text,
            reasoning,
            tool_calls,
            tool_results,
            media: None,
            stop_reason,
            usage: None,
        }
    }
}

impl From<OpenAIUsage> for Usage {
    fn from(value: OpenAIUsage) -> Self {
        Usage {
            cache_writes: None,
            cache_reads: None,
            input_tokens: value.prompt_tokens,
            output_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
            cost: 0.0,
        }
    }
}