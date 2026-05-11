use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::config::ModelParameters;
use crate::llm::LLMResult;

#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    #[serde(rename = "base_resp")]
    base_resp: Option<AnthropicBaseResp>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicBaseResp {
    status_code: i64,
    status_msg: String,
}

#[derive(Debug)]
pub(crate) struct AnthropicCompatible {
    pub(crate) url: String,
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    usage: AnthropicResponseUsage,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
    #[serde(rename = "base_resp")]
    base_resp: Option<AnthropicBaseResp>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    thinking: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicResponseUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: i64,
    #[serde(rename = "output_tokens")]
    output_tokens: i64,
}

impl AnthropicCompatible {
    pub(crate) fn request(&self, diff_content: &str, option: ModelParameters) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();
        let url = if self.url.ends_with("/messages") {
            self.url.clone()
        } else if self.url.ends_with("/v1") {
            format!("{}/messages", self.url)
        } else {
            format!("{}/v1/messages", self.url)
        };

        let response = client
            .post(url)
            .timeout(Duration::from_secs(120))
            .header("X-Api-Key", api_key)
            .json(&json!({
                "model": &self.model,
                "system": self.prompt,
                "messages": [
                    {
                        "role": "user",
                        "content": format!("diff content: \n{diff_content}")
                    }
                ],
                "max_tokens": option.max_tokens,
                "temperature": option.temperature,
                "top_p": option.top_p,
            }))
            .send()
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        if response.status().is_success() {
            let response_json: AnthropicResponse = response
                .json()
                .map_err(|e| anyhow!("Failed to parse response as JSON: {}", e))?;

            // Extract text content from content blocks
            let mut msg = String::new();
            let mut reasoning = None;
            for block in &response_json.content {
                match block.block_type.as_str() {
                    "text" => {
                        if let Some(ref text) = block.text {
                            msg.push_str(text.trim());
                        }
                    }
                    "thinking" => {
                        if let Some(ref thinking_text) = block.thinking {
                            reasoning = Some(thinking_text.trim().to_string());
                        }
                    }
                    _ => {}
                }
            }

            if msg.is_empty() {
                if let Some(ref reasoning_text) = reasoning {
                    msg = reasoning_text.clone();
                }
            }

            Ok(LLMResult {
                commit_message: msg,
                total_tokens: response_json.usage.input_tokens + response_json.usage.output_tokens,
                prompt_tokens: response_json.usage.input_tokens,
                completion_tokens: response_json.usage.output_tokens,
                prompt_cache_hit_tokens: None,
                reasoning_content: reasoning,
                model: self.model.clone(),
            })
        } else {
            let status = response.status();
            let text = response.text().map_err(|e| {
                let msg = e.to_string();
                let truncated = if msg.len() > 100 { &msg[..100] } else { &msg };
                anyhow!("Error reading error response: {}", truncated)
            })?;

            // Try to parse the MiniMax anthropic error format.
            if let Ok(api_err) = serde_json::from_str::<AnthropicErrorResponse>(&text) {
                if let Some(base_resp) = api_err.base_resp {
                    return Err(anyhow!(
                        "API error ({}): {} (code: {})",
                        status,
                        base_resp.status_msg,
                        base_resp.status_code
                    ));
                }
            }

            Err(anyhow!("API error ({}): {}", status, text))
        }
    }
}
