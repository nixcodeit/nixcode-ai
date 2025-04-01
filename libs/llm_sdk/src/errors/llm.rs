use crate::ErrorContent;
use anyhow::Error;

#[derive(Debug, Clone)]
pub enum LLMError {
    CreateClientError(String),
    InvalidRequest,
    InvalidResponseCode(u16, String),
    InvalidResponse(String),
    ParseError(String),
    ReqwestError,
    NetworkError,
    Timeout,
    InputTooLong,
    MissingAPIKey,
    Generic(String),
}

impl Into<Error> for LLMError {
    fn into(self) -> Error {
        match self {
            LLMError::CreateClientError(e) => Error::msg(e),
            LLMError::InvalidRequest => Error::msg("Invalid request"),
            LLMError::InvalidResponseCode(code, body) => Error::msg(format!(
                "Invalid response code: {} with body: {}",
                code, body
            )),
            LLMError::InvalidResponse(e) => Error::msg(e),
            LLMError::ParseError(e) => Error::msg(e),
            LLMError::ReqwestError => Error::msg("Reqwest error"),
            LLMError::NetworkError => Error::msg("Network error"),
            LLMError::Timeout => Error::msg("Timeout"),
            LLMError::InputTooLong => Error::msg("Input too long"),
            LLMError::MissingAPIKey => Error::msg("Missing API key"),
            LLMError::Generic(e) => Error::msg(e),
        }
    }
}

impl Into<ErrorContent> for LLMError {
    fn into(self) -> ErrorContent {
        ErrorContent {
            r#type: match self {
                LLMError::CreateClientError(_) => "create_client_error".into(),
                LLMError::InvalidRequest => "invalid_request".into(),
                LLMError::InvalidResponseCode(_, _) => "invalid_response_code".into(),
                LLMError::InvalidResponse(_) => "invalid_response".into(),
                LLMError::ParseError(_) => "parse_error".into(),
                LLMError::ReqwestError => "reqwest_error".into(),
                LLMError::NetworkError => "network_error".into(),
                LLMError::Timeout => "timeout".into(),
                LLMError::InputTooLong => "input_too_long".into(),
                LLMError::MissingAPIKey => "missing_api_key".into(),
                LLMError::Generic(_) => "generic".into(),
            },
            message: match self {
                LLMError::CreateClientError(e) => e,
                LLMError::InvalidRequest => "Invalid request".into(),
                LLMError::InvalidResponseCode(code, body) => {
                    format!("Invalid response code: {} with body: {}", code, body)
                }
                LLMError::InvalidResponse(e) => e,
                LLMError::ParseError(e) => e,
                LLMError::ReqwestError => "Reqwest error".into(),
                LLMError::NetworkError => "Network error".into(),
                LLMError::Timeout => "Timeout".into(),
                LLMError::InputTooLong => "Input too long".into(),
                LLMError::MissingAPIKey => {
                    "Missing API key. Please provide in config file or environment variable.".into()
                }
                LLMError::Generic(e) => e,
            },
        }
    }
}
