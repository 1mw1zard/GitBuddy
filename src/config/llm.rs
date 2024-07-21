use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// TODO 
const DEFAULT_DIR: &str = ".gitbuddy";

/// get config dir path
fn get_config_dir() -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(mut home) => {
            home.push(DEFAULT_DIR);
            Some(home)
        }
        None => None,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UseModel {
    DeepSeek(OpenAILikeParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OpenAILikeParams {
    pub model: String,
    pub api_key: String,
}
