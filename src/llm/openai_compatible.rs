use serde_json::json;
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: Option<String>,
}

impl OpenAICompatible {
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
            "model": &self.model,
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
