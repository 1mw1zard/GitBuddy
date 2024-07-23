use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
pub use crate::config::vendor::{OpenAILikeParams, UseModel};
use crate::llm::PromptModel;

mod vendor;
mod storage;

pub fn handler(vendor: &PromptModel, api_key: &str, model: &str) -> Result<()> {
    let mut config: GlobalConfig = match GlobalConfig::load() {
        None => {
            GlobalConfig::new()
        }
        Some(cfg) => { cfg }
    };

    let openai_like_params = OpenAILikeParams {
        model: model.to_string(),
        api_key: api_key.to_string(),
    };

    let model: UseModel = match vendor {
        PromptModel::DeepSeek => {
            UseModel::DeepSeek(openai_like_params)
        }
        PromptModel::OpenAI => {
            UseModel::OpenAI(openai_like_params)
        }
    };

    config.set_model(model);
    config.save().expect("Failed to save config.");

    println!("Config saved.");

    Ok(())
}

pub fn get_config() -> Result<GlobalConfig> {
    let result = GlobalConfig::load();
    match result {
        Some(config) => Ok(config),
        None => {
            Err(anyhow!("Config not found."))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub model: Option<UseModel>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        GlobalConfig {
            model: None,
        }
    }

    /// Set use model
    pub fn set_model(&mut self, model: UseModel) {
        self.model = Some(model);
    }

    /// save config to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        let _ = storage::save_config(&content)?;
        Ok(())
    }

    /// load config from file
    pub fn load() -> Option<Self> {
        let content = storage::read_config().unwrap_or_default();
        match toml::from_str(content.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("load config error: {}", err);
                None
            }
        }
    }
}


#[cfg(test)]
mod test {
    use crate::config::vendor::OpenAILikeParams;

    use super::*;

    #[test]
    fn test_config() {
        let params = OpenAILikeParams {
            model: String::from("gpt-3.5-turbo"),
            api_key: String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        };

        let mut cfg = GlobalConfig::new();
        cfg.set_model(UseModel::DeepSeek(params));

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

        let cfg: GlobalConfig = toml::from_str(toml_str).unwrap();
    }

    #[test]
    fn save_config() {
        let mut cfg = GlobalConfig::new();

        let params = OpenAILikeParams {
            model: String::from("gpt-3.5-turbo"),
            api_key: String::from("sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        };

        cfg.set_model(UseModel::DeepSeek(params));
        // cfg.save();
    }

}
