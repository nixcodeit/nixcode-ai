use crate::message::content::image_source::ImageSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageContent {
    source: ImageSource,
}
