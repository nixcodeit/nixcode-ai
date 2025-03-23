use anyhow::Error;

#[derive(Debug)]
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
            LLMError::Generic(e) => Error::msg(e),
        }
    }
}
