use serde_json::json;
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    url: String,
    default_model: String,
    prompt: String,
    api_key: Option<String>,
}

impl OpenAICompatible {
    pub(crate) fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_string());
    }
    pub(crate) fn request(&self, diff_content: &str) -> Result<String> {
        let client = reqwest::blocking::Client::new();

        let real_api_key: String;
        if let Some(api_key) = self.api_key.clone() {
            real_api_key = api_key;
        } else {
            return Err(anyhow::anyhow!("OpenAI API key is empty"));
        }


        let response = client
            .post(format!("{}/v1/chat/completions", self.url))
            .header("Authorization", format!("Bearer {real_api_key}", ))
            .json(&json!({
            "model": self.default_model,
            "messages": [
                {
                    "role": "system",
                    "content": self.prompt,
                },
                {
                    "role": "user",
                    "content": diff_content
                }
            ],
            "max_tokens": 100
        }))
            .send()
            .expect("Error sending request");

        // TODO
        _ = response;

        Ok("todo".to_string())
    }
}

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    default_model: String,
    prompt: String,
}

impl Default for OpenAICompatibleBuilder {
    fn default() -> Self {
        Self {
            url: String::from("https://api.openai.com"),
            default_model: String::from("gpt-3.5-turbo"),
            prompt: String::from("Generate a concise commit message based on \
            the following git difference content. The generated message is plain text,\
             does not contain identifiers such as markdown \"`\", \
             and the generated content does not exceed 100 tokens. \
             Depending on the nature of the change, it starts with one of the following prefixes:\
              'build' (build system), 'chore' (chores), 'ci' (continuous integration), \
              'docs' (documentation), 'feat' (new feature), 'fix' (fix), 'perf' (performance),\
               'refactor' (refactoring), 'style' (style), 'test' (test):"),
        }
    }
}

impl OpenAICompatibleBuilder {
    /// Set the name of the model.
    pub(crate) fn url(mut self, url: String) -> OpenAICompatibleBuilder {
        // Set the name on the builder itself, and return the builder by value.
        self.url = url;
        self
    }

    /// Set the name of the model.
    pub(crate) fn use_model(mut self, model: String) -> OpenAICompatibleBuilder {
        self.default_model = model;
        self
    }

    pub fn build(self) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            default_model: self.default_model,
            prompt: self.prompt,
            api_key: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::llm::openai_compatible::{OpenAICompatible, OpenAICompatibleBuilder};

    #[test]
    fn test_builder_default() {
        let cfg: OpenAICompatible = OpenAICompatibleBuilder::default().build();
        assert_eq!(cfg.url, "https://api.openai.com");
    }

    #[test]
    fn builder_test() {
        let cfg: OpenAICompatible = OpenAICompatibleBuilder::default().url(String::from("https://api.deepseek.com")).build();
        assert_eq!(cfg.url, "https://api.deepseek.com");
        assert_eq!(cfg.default_model, "gpt-3.5-turbo");
        assert_eq!(cfg.api_key, None);

        let cfg: OpenAICompatible = OpenAICompatibleBuilder::default().
            url(String::from("https://api.deepseek.com")).
            use_model(String::from("deepseek-chat")).
            build();
        assert_eq!(cfg.url, "https://api.deepseek.com");
        assert_eq!(cfg.default_model, "deepseek-chat");
        assert_eq!(cfg.api_key, None);
    }
}
