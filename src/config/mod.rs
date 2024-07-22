mod llm;
mod storage;

use std::any::Any;
use serde::{Deserialize, Serialize};
use llm::UseModel;
use anyhow::Result;

pub fn handler() {
    println!("Hello, world!");
}

#[derive(Debug, Serialize, Deserialize)]
struct GlobalConfig {
    model: Option<UseModel>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        GlobalConfig {
            model: None,
        }
    }

    /// set use model
    pub fn set_model(&mut self, model: UseModel) {
        self.model = Some(model);
    }

    /// save config to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self)?;
        let _ = storage::save_config(&content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let content = storage::read_config()?;
        let config: GlobalConfig = toml::from_str(content.as_str())?;
        Ok(config)
    }
}


#[cfg(test)]
mod test {
    use crate::config::llm::OpenAILikeParams;
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

    #[test]
    fn load_config() {
        let result = GlobalConfig::load();
        match result {
            Ok(cfg) => {
                println!("{:?}", cfg);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
