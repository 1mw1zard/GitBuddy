use crate::llm::openai_compatible::OpenAICompatible;
use crate::llm::PromptModel;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
    api_key: String,
}

impl OpenAICompatibleBuilder {
    pub fn new(vendor: PromptModel, model: &str, api_key: &str, base_url: &str) -> Self {
        let url = if base_url.is_empty() {
            default_base_url(vendor).to_string()
        } else {
            base_url.trim_end_matches('/').to_string()
        };

        Self {
            url,
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn build(self, prompt: String) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            model: self.model,
            prompt,
            api_key: self.api_key,
        }
    }
}

fn default_base_url(vendor: PromptModel) -> &'static str {
    match vendor {
        PromptModel::OpenAI => "https://api.openai.com",
        PromptModel::DeepSeek => "https://api.deepseek.com",
        PromptModel::Ollama => "http://localhost:11434",
    }
}
