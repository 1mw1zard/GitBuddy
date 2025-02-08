use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelParameters {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultConfig {
    pub default_service: String,
    pub timeout: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub default: DefaultConfig,
    pub openai: Option<ModelConfig>,
    pub deepseek: Option<ModelConfig>,
    pub ollama: Option<ModelConfig>,
    pub model_parameters: Option<ModelParameters>,
}
