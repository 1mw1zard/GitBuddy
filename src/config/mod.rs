mod llm;

use serde::{Deserialize, Serialize};
use llm::UseModel;

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
    pub fn save(&self)  {
        // TODO
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
}
