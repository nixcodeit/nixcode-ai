use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Usage {
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl AddAssign<Usage> for Usage {
    fn add_assign(&mut self, rhs: Usage) {
        self.output_tokens += rhs.output_tokens;
        self.input_tokens += rhs.input_tokens;
        self.cache_read_input_tokens = match (
            self.cache_read_input_tokens,
            rhs.cache_creation_input_tokens,
        ) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        self.cache_creation_input_tokens = match (
            self.cache_creation_input_tokens,
            rhs.cache_creation_input_tokens,
        ) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

impl AddAssign<&Usage> for Usage {
    fn add_assign(&mut self, rhs: &Usage) {
        *self += rhs.clone();
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UsageDelta {
    pub output_tokens: u32,
}

impl AddAssign<UsageDelta> for Usage {
    fn add_assign(&mut self, rhs: UsageDelta) {
        self.output_tokens += rhs.output_tokens;
    }
}
