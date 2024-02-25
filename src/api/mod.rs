use serde::{Deserialize, Serialize};

pub mod bad_words;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse {
    pub message: String,
}
