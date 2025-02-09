use crate::llm::PromptModel;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

mod storage;
mod vendor;

/// Update or create configuration for a specific model
pub fn handler(vendor: &PromptModel, api_key: &str, model: &str) -> Result<()> {
    let mut config = GlobalConfig::load().unwrap_or_else(|| create_default_config());

    let model_config = ModelConfig {
        api_key: Some(api_key.to_string()),
        model: model.to_string(),
        base_url: get_default_base_url(vendor),
    };

    match vendor {
        PromptModel::DeepSeek => config.deepseek = Some(model_config),
        PromptModel::OpenAI => config.openai = Some(model_config),
        PromptModel::Ollama => config.ollama = Some(model_config),
    }

    config.save()?;
    println!("Config saved.");
    Ok(())
}

fn get_default_base_url(vendor: &PromptModel) -> String {
    match vendor {
        PromptModel::OpenAI => "https://api.openai.com/v1".to_string(),
        PromptModel::DeepSeek => "https://api.deepseek.com/v1".to_string(),
        PromptModel::Ollama => "http://localhost:11434".to_string(),
    }
}

fn create_default_config() -> GlobalConfig {
    GlobalConfig {
        default: DefaultConfig {
            default_service: PromptModel::DeepSeek,
            timeout: 30,
        },
        openai: None,
        deepseek: None,
        ollama: None,
        model_parameters: Some(ModelParameters {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2000,
        }),
    }
}

pub fn get_config() -> Result<GlobalConfig> {
    GlobalConfig::load().ok_or_else(|| anyhow!("Config not found."))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default: DefaultConfig,
    pub openai: Option<ModelConfig>,
    pub deepseek: Option<ModelConfig>,
    pub ollama: Option<ModelConfig>,
    pub model_parameters: Option<ModelParameters>,
}

impl GlobalConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        create_default_config()
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        storage::save_config(&content)?;
        Ok(())
    }

    /// Load config from file
    pub fn load() -> Option<Self> {
        let content = storage::read_config().unwrap_or_default();
        match toml::from_str(content.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("Load config error: {}", err);
                None
            }
        }
    }

    // load model
    pub fn model(&self, vendor: Option<PromptModel>) -> Option<(&ModelConfig, PromptModel)> {
        match vendor.unwrap_or(self.default.default_service) {
            PromptModel::OpenAI => self.openai.as_ref().map(|cfg| (cfg, PromptModel::OpenAI)),
            PromptModel::DeepSeek => self.deepseek.as_ref().map(|cfg| (cfg, PromptModel::DeepSeek)),
            PromptModel::Ollama => self.ollama.as_ref().map(|cfg| (cfg, PromptModel::Ollama)),
        }
    }

    pub fn model_params(&self) -> ModelParameters {
        match &self.model_parameters {
            Some(mp) => mp.clone(),
            None => ModelParameters {
                max_tokens: 1024,
                temperature: 0.0,
                top_p: 0.75,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub default_service: PromptModel,
    pub timeout: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelParameters {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: u32,
}

#[cfg(test)]
mod test {
    use crate::config::vendor::ModelConfig;

    use super::*;

    #[test]
    fn test_config() {
        let params = ModelConfig {
            model: String::from("gpt-3.5-turbo"),
            api_key: Some(String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")),
        };

        let mut cfg = GlobalConfig::new();
        // cfg.set_model(UseModel::DeepSeek(params));

        let toml_str = toml::to_string(&cfg).unwrap();
        println!("{}", toml_str);
    }

    #[test]
    fn config_serialization() {
        let toml_str = r#"
[model.DeepSeek]
model = "gpt-3.5-turbo"
api_key = "sk-12345678"
        "#;

        // let cfg: GlobalConfig = toml::from_str(toml_str).unwrap();
    }

    #[test]
    fn save_config() {
        let mut cfg = GlobalConfig::new();

        // let params = OpenAILikeParams {
        //     model: String::from("gpt-3.5-turbo"),
        //     api_key: String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        // };

        // cfg.set_model(UseModel::DeepSeek(params));
        // cfg.save();
    }
}
