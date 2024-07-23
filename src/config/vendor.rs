use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UseModel {
    DeepSeek(OpenAILikeParams),
    OpenAI(OpenAILikeParams),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAILikeParams {
    pub model: String,
    pub api_key: String,
}
