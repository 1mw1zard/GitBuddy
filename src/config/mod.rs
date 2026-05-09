use crate::llm::PromptModel;
use anyhow::{anyhow, Result};

pub use vendor::{DefaultConfig, ModelConfig, ModelParameters};

mod storage;
mod vendor;

pub fn handler(vendor: &PromptModel, api_key: &str, model: &str) -> Result<()> {
    let mut config = GlobalConfig::load().unwrap_or_else(GlobalConfig::new);

    let model_config = ModelConfig {
        api_key: if api_key.is_empty() {
            None
        } else {
            Some(api_key.to_string())
        },
        model: model.to_string(),
        base_url: default_base_url(vendor).to_string(),
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

fn default_base_url(vendor: &PromptModel) -> &'static str {
    match vendor {
        PromptModel::OpenAI => "https://api.openai.com",
        PromptModel::DeepSeek => "https://api.deepseek.com",
        PromptModel::Ollama => "http://localhost:11434",
    }
}

pub fn get_config() -> Result<GlobalConfig> {
    GlobalConfig::load().ok_or_else(|| anyhow!("Config not found. Run `gitbuddy config` first."))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct GlobalConfig {
    pub default: DefaultConfig,
    pub openai: Option<ModelConfig>,
    pub deepseek: Option<ModelConfig>,
    pub ollama: Option<ModelConfig>,
    pub model_parameters: Option<ModelParameters>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            default: DefaultConfig {
                default_service: PromptModel::DeepSeek,
                timeout: 30,
            },
            openai: None,
            deepseek: None,
            ollama: None,
            model_parameters: Some(ModelParameters::default()),
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        storage::save_config(&content)?;
        Ok(())
    }

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

    pub fn model(&self, vendor: Option<PromptModel>) -> Option<(&ModelConfig, PromptModel)> {
        match vendor.unwrap_or(self.default.default_service) {
            PromptModel::OpenAI => self.openai.as_ref().map(|cfg| (cfg, PromptModel::OpenAI)),
            PromptModel::DeepSeek => self.deepseek.as_ref().map(|cfg| (cfg, PromptModel::DeepSeek)),
            PromptModel::Ollama => self.ollama.as_ref().map(|cfg| (cfg, PromptModel::Ollama)),
        }
    }

    pub fn model_params(&self) -> ModelParameters {
        self.model_parameters.unwrap_or_default()
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_config_serializes() {
        let cfg = GlobalConfig::new();
        let toml_str = toml::to_string(&cfg).unwrap();

        assert!(toml_str.contains("[default]"));
        assert!(toml_str.contains("default_service = \"deepseek\""));
    }

    #[test]
    fn config_deserializes_vendor_sections() {
        let toml_str = r#"
[default]
default_service = "deepseek"
timeout = 30

[deepseek]
model = "deepseek-chat"
api_key = "sk-12345678"
base_url = "https://api.deepseek.com"
        "#;

        let cfg: GlobalConfig = toml::from_str(toml_str).unwrap();
        let (model_config, vendor) = cfg.model(None).unwrap();

        assert_eq!(vendor, PromptModel::DeepSeek);
        assert_eq!(model_config.model, "deepseek-chat");
        assert_eq!(model_config.api_key.as_deref(), Some("sk-12345678"));
    }
}
