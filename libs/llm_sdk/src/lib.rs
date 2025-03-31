pub mod config;
pub mod errors;
pub mod json_schema;
pub mod message;
pub mod providers;
pub mod stop_reason;
pub mod tools;

use crate::tools::Tool;
use config::LLMConfig;
use errors::llm::LLMError;
use eventsource_stream::{Event, Eventsource};
use futures::StreamExt;
use message::content::{Content, ContentDelta};
use message::message::Message;
use message::response::MessageResponse;
use message::usage::{Usage, UsageDelta};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ops::AddAssign;
use stop_reason::StopReason;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub type MessageResponseStream = UnboundedReceiver<MessageResponseStreamEvent>;

#[derive(Debug)]
pub struct Response {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThinkingOptions {
    r#type: String,
    budget_tokens: u32,
}

impl ThinkingOptions {
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            r#type: "enabled".into(),
            budget_tokens,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum ApiStandard {
    #[default]
    Anthropic,
    OpenAI,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<Vec<Content>>,
    #[serde(skip)]
    _cache: Option<bool>,
}

impl Default for Request {
    fn default() -> Self {
        Request {
            model: "claude-3-7-sonnet-20250219".to_string(),
            messages: Vec::new(),
            max_tokens: None,
            stream: true,
            thinking: None,
            tools: None,
            system: None,
            _cache: None,
        }
    }
}

impl Request {
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_thinking(mut self, thinking: ThinkingOptions) -> Self {
        self.thinking = Some(thinking);
        self
    }

    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_system_prompt(mut self, system: Vec<Content>) -> Self {
        self.system = Some(system);
        self
    }

    pub fn with_cache(mut self) -> Self {
        self._cache = Some(true);
        self
    }

    pub fn is_cache_enabled(&self) -> bool {
        self._cache.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub struct MessageDelta {
    stop_reason: Option<StopReason>,
    stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageStartEventContent {
    message: MessageResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockStartEventContent {
    index: usize,
    content_block: Content,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockDeltaEventContent {
    index: usize,
    delta: ContentDelta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockStopEventContent {
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageDeltaEventContent {
    delta: MessageDelta,
    usage: UsageDelta,
}

impl MessageDeltaEventContent {
    pub fn get_delta(&self) -> MessageDelta {
        self.delta.clone()
    }

    pub fn get_usage(&self) -> UsageDelta {
        self.usage.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorContent {
    r#type: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageResponseStreamEvent {
    MessageStart(MessageStartEventContent),
    ContentBlockStart(ContentBlockStartEventContent),
    ContentBlockDelta(ContentBlockDeltaEventContent),
    ContentBlockStop(ContentBlockStopEventContent),
    MessageDelta(MessageDeltaEventContent),
    MessageStop,
    Ping,
    Error { error: ErrorContent },
}

#[derive(Debug)]
pub struct AnthropicClient {
    total_usages: Usage,
    history: Vec<Message>,
    options: LLMConfig,
    client: reqwest::Client,
}

impl AnthropicClient {
    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }
}

impl AnthropicClient {
    pub fn get_usage(&self) -> Usage {
        self.total_usages.clone()
    }
}

impl AddAssign<Message> for AnthropicClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl TryFrom<Event> for MessageResponseStreamEvent {
    type Error = LLMError;

    fn try_from(value: Event) -> Result<Self, LLMError> {
        let data = value.data;

        let event = serde_json::from_str::<MessageResponseStreamEvent>(&data);

        if let Err(err) = event {
            return Err(LLMError::ParseError(err.to_string()));
        }

        Ok(event.unwrap())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LLMEvent {
    PartialContent(usize, Content),
    Content(Content),
    Stop(Option<StopReason>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct InputTokens {
    input_tokens: u32,
}

pub enum LLMClient {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
}

impl LLMClient {
    pub fn new_openai(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = OpenAIClient::new(options);

        if let Err(client) = client {
            return Err(client);
        }

        Ok(LLMClient::OpenAI(client?))
    }

    pub fn new_anthropic(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = AnthropicClient::new(options);

        if let Err(client) = client {
            return Err(client);
        }

        Ok(LLMClient::Anthropic(client?))
    }

    pub async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        match self {
            LLMClient::OpenAI(client) => client.count_tokens(request).await,
            LLMClient::Anthropic(client) => client.count_tokens(request).await,
        }
    }

    pub async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        match self {
            LLMClient::OpenAI(client) => client.send(request).await,
            LLMClient::Anthropic(client) => client.send(request).await,
        }
    }
}

pub struct OpenAIClient {}

pub trait LLMClientImpl {
    fn count_tokens(
        &self,
        request: Request,
    ) -> impl std::future::Future<Output = Result<u32, LLMError>> + Sync;
    fn send(
        &self,
        request: Request,
    ) -> impl std::future::Future<
        Output = Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError>,
    > + Sync;

    fn get_config(&self) -> LLMConfig;
}

impl LLMClientImpl for OpenAIClient {
    async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        todo!()
    }

    async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        todo!()
    }

    fn get_config(&self) -> LLMConfig {
        todo!()
    }
}

impl OpenAIClient {
    pub fn new(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        todo!()
    }
}

impl AnthropicClient {
    pub fn new(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            options.api_key.expose_secret().parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        // headers.insert(
        //     AUTHORIZATION,
        //     format!("Bearer {}", options.api_key.expose_secret())
        //         .parse()
        //         .unwrap(),
        // );

        let reqwest_client = reqwest::Client::builder().default_headers(headers).build();
        if reqwest_client.is_err() {
            return Err(LLMError::CreateClientError(
                "Failed to create client".to_string(),
            ));
        }

        let client = AnthropicClient {
            options,
            client: reqwest_client.unwrap(),
            history: Vec::new(),
            total_usages: Usage::default(),
        };

        Ok(client)
    }
}

impl LLMClientImpl for AnthropicClient {
    async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        let body = serde_json::to_value(&request).unwrap();

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages/count_tokens")
            .json(&body)
            .send()
            .await;

        if response.is_err() {
            return Err(LLMError::ReqwestError);
        }

        let response = response.unwrap();

        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap(),
            ));
        }

        let body = response.json::<InputTokens>().await;

        if body.is_err() {
            return Err(LLMError::InvalidResponse(body.unwrap_err().to_string()));
        }

        Ok(body.unwrap().input_tokens)
    }
    async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        let mut body = serde_json::to_value(&request).unwrap();
        if request.is_cache_enabled() && !request.messages.is_empty() {
            body.as_object_mut()
                .unwrap()
                .get_mut("messages")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .as_object_mut()
                .unwrap()
                .get_mut("content")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("cache_control".into(), json!({"type": "ephemeral"}));

            if request.system.is_some() {
                body.as_object_mut()
                    .unwrap()
                    .get_mut("system")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("cache_control".into(), json!({"type": "ephemeral"}));
            }
            if request.tools.is_some() {
                body.as_object_mut()
                    .unwrap()
                    .get_mut("tools")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("cache_control".into(), json!({"type": "ephemeral"}));
            }
        }

        let result = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .json(&body)
            .send()
            .await;

        if result.is_err() {
            return Err(LLMError::ReqwestError);
        }

        let response = result.unwrap();

        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap(),
            ));
        }

        let (tx, rx) = unbounded_channel::<MessageResponseStreamEvent>();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream().eventsource();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(event) => {
                        let event = MessageResponseStreamEvent::try_from(event);

                        if let Err(err) = event {
                            tx.send(MessageResponseStreamEvent::Error { error: err.into() })
                                .ok();
                            continue;
                        }

                        tx.send(event.unwrap()).ok();
                    }
                    Err(e) => {
                        tx.send(MessageResponseStreamEvent::Error {
                            error: ErrorContent {
                                r#type: "EventStreamError".into(),
                                message: e.to_string(),
                            },
                        })
                        .ok();
                    }
                };
            }
        });

        Ok(rx)
    }

    fn get_config(&self) -> LLMConfig {
        todo!()
    }
}
