use crate::llm::openai_compatible::OpenAICompatible;
use crate::llm::prompt::PROMPT;
use crate::llm::PromptModel;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
    api_key: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(vendor: PromptModel, model: &str, api_key: &str) -> Self {
        match vendor {
            PromptModel::OpenAI => OpenAICompatibleBuilder {
                url: String::from("https://api.openai.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
            PromptModel::DeepSeek => OpenAICompatibleBuilder {
                url: String::from("https://api.deepseek.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
            PromptModel::Ollama => OpenAICompatibleBuilder {
                url: String::from("http://localhost:11434"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
        }
    }

    pub fn build(self) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            model: self.model,
            prompt: PROMPT.parse().unwrap(),
            api_key: self.api_key,
        }
    }
}
