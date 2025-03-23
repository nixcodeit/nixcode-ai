use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub image_type: String,
    pub media_type: String,
    pub data: String,
}
