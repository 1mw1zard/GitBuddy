use serde_json::json;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::llm::LLMResult;

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
    model: String, // the model used to generate this completion
    object: String,
    system_fingerprint: String, // This fingerprint represents the backend configuration that the model runs with.
    choices: Vec<OpenAIResponseChoice>,
    usage: OpenAIResponseUsage, // the usage information of the request
    created: i64, // the Unix timestamp when the request was created
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoice {
    index: i64, // the index of the choice in the choices list
    message: OpenAIResponseChoiceMessage,
    finish_reason: String, // the reason why the model stopped generating tokens
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseChoiceMessage {
    role: String, // the role of the message
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponseUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}


impl OpenAICompatible {
    pub(crate) fn request(&self, diff_content: &str) -> Result<LLMResult> {
        let client = reqwest::blocking::Client::new();

        let api_key = self.api_key.clone();

        let response = client
            .post(format!("{}/v1/chat/completions", self.url))
            .header("Authorization", format!("Bearer {api_key}", ))
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

        return if response.status().is_success() {
            let _response_json = OpenAIResponse {
                id: "".to_string(),
                model: "".to_string(),
                object: "".to_string(),
                system_fingerprint: "".to_string(),
                choices: vec![],
                usage: OpenAIResponseUsage {
                    completion_tokens: 0,
                    prompt_tokens: 0,
                    total_tokens: 0,
                },
                created: 0,
            };
            let response_json: OpenAIResponse = response.json().expect("Failed to parse response as JSON");

            if response_json.choices.is_empty() {
                panic!("No choices returned from OpenAI API");
            }
            let choice = &response_json.choices[0];
            Ok(LLMResult {
                commit_message: choice.message.content.clone().trim().to_string(),
                total_tokens: response_json.usage.total_tokens,
                prompt_tokens: response_json.usage.prompt_tokens,
                completion_tokens: response_json.usage.completion_tokens,
            })
        } else {
            let reason = match response.text() {
                Ok(text) => text,
                Err(e) => {
                    return Err(anyhow!("Error: {:?}", e.to_string().truncate(100)));
                }
            };
            return Err(anyhow!("Error: {}", reason));
        };
    }
}
