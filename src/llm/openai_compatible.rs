use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::config::ModelParameters;
use crate::llm::LLMResult;

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
    code: Option<String>,
}

#[derive(Debug)]
pub(crate) struct OpenAICompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    object: String,
    system_fingerprint: String,
    choices: Vec<OpenAIResponseChoice>,
    usage: OpenAIResponseUsage,
    created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: i64,
    message: OpenAIResponseChoiceMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoiceMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
    #[serde(rename = "prompt_cache_hit_tokens")]
    prompt_cache_hit_tokens: Option<i64>,

}

impl OpenAICompatible {
    pub(crate) fn request(&self, diff_content: &str, option: ModelParameters) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();
        let url = if self.url.ends_with("/chat/completions") {
            self.url.clone()
        } else if self.url.ends_with("/v1") {
            format!("{}/chat/completions", self.url)
        } else {
            format!("{}/v1/chat/completions", self.url)
        };

        let response = client
            .post(url)
            .timeout(Duration::from_secs(120))
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&json!({
                "model": &self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": self.prompt,
                    },
                    {
                        "role": "user",
                        "content": format!("diff content: \n{diff_content}")
                    }
                ],
                "options": option,
                "keep_alive": "30m",
                "max_tokens": option.max_tokens,
            }))
            .send()
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        if response.status().is_success() {
            let response_json: OpenAIResponse = response
                .json()
                .map_err(|e| anyhow!("Failed to parse response as JSON: {}", e))?;

            let choice = response_json
                .choices
                .first()
                .ok_or_else(|| anyhow!("No choices returned from API"))?;

            Ok(LLMResult {
                commit_message: choice.message.content.trim().to_string(),
                total_tokens: response_json.usage.total_tokens,
                prompt_tokens: response_json.usage.prompt_tokens,
                completion_tokens: response_json.usage.completion_tokens,
                prompt_cache_hit_tokens: response_json.usage.prompt_cache_hit_tokens,
                model: self.model.clone(),
            })
        } else {
            let status = response.status();
            let text = response.text().map_err(|e| {
                let msg = e.to_string();
                let truncated = if msg.len() > 100 { &msg[..100] } else { &msg };
                anyhow!("Error reading error response: {}", truncated)
            })?;

            // Try to parse the standard OpenAI-compatible error format.
            if let Ok(api_err) = serde_json::from_str::<ApiErrorResponse>(&text) {
                if matches!(api_err.error.code.as_deref(), Some("context_length_exceeded")) {
                    return Err(anyhow!(
                        "The staged changes are too large for the model's context window. \
                         Try committing fewer files at once with `git add <specific-files>`."
                    ));
                }
                return Err(anyhow!("API error ({}): {}", status, api_err.error.message));
            }

            Err(anyhow!("API error ({}): {}", status, text))
        }
    }
}
