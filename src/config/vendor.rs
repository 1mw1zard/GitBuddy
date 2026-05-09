use crate::llm::PromptModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ModelParameters {
    pub temperature: f64,
    pub top_p: f64,
    pub top_k: u32,
    pub max_tokens: u32,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: 0.1,
            top_p: 0.75,
            top_k: 5,
            max_tokens: 1024,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultConfig {
    pub default_service: PromptModel,
    pub timeout: u64,
}
