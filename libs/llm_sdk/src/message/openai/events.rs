use crate::errors::llm::LLMError;
use crate::message::content::ContentDelta;
use serde::{Deserialize, Serialize};

/// OpenAI's streaming response format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIStreamResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIStreamChoice {
    pub index: usize,
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

impl TryInto<ContentDelta> for OpenAIDelta {
    type Error = LLMError;

    fn try_into(self) -> Result<ContentDelta, LLMError> {
        if let Some(content) = self.content {
            return Ok(ContentDelta::TextDelta(crate::message::content::text::ContentTextDelta {
                text: content,
            }))
        } else if let Some(tool_calls) = self.tool_calls {
            if tool_calls.is_empty() {
                return Err(LLMError::ParseError("Empty tool calls".to_string()));
            }
            if tool_calls.len() > 1 {
                return Err(LLMError::ParseError("Multiple tool calls".to_string()));
            }

            let tool_content = tool_calls[0].clone();

            return Ok(ContentDelta::InputJsonDelta(crate::message::content::tools::ContentInputJsonDelta {
                partial_json: tool_content.function.arguments
            }))
        }

        Err(LLMError::ParseError("No content or tool calls".to_string()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIToolCall {
    pub index: usize,
    pub id: Option<String>,
    pub function: OpenAIFunctionContent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIFunctionContent {
    pub name: Option<String>,
    pub arguments: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIUsage {
    #[serde(rename = "prompt_tokens")]
    pub input_tokens: u32,
    #[serde(rename = "completion_tokens")]
    pub output_tokens: u32,
    pub total_tokens: u32,
}