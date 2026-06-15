use anyhow::{anyhow, Result};
use clap::ValueEnum;
use colored::Colorize;
use futures::StreamExt;
use rig::client::CompletionClient;
use rig::completion::{CompletionModel, GetTokenUsage};
use rig::streaming::StreamedAssistantContent;
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::config::{get_config, ModelParameters};
use crate::prompt::Prompt;

/// Prompt model
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Deserialize, Serialize)]
pub enum PromptModel {
    #[clap(name = "openai")]
    #[serde(rename = "openai")]
    OpenAI,
    #[clap(name = "deepseek")]
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[clap(name = "ollama")]
    #[serde(rename = "ollama")]
    Ollama,
    #[clap(name = "minimax")]
    #[serde(rename = "minimax")]
    MiniMax,
}

impl PromptModel {
    pub fn default_model(&self) -> String {
        match self {
            PromptModel::OpenAI => "gpt-3.5-turbo".to_string(),
            PromptModel::DeepSeek => "deepseek v4 flash".to_string(),
            PromptModel::Ollama => "ollama".to_string(),
            PromptModel::MiniMax => "MiniMax-M2.7".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct LLMResult {
    pub commit_message: String,
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
    pub prompt_cache_hit_tokens: Option<i64>,
    pub reasoning_content: Option<String>,
}

pub async fn llm_request(
    diff_content: &str,
    vendor: Option<PromptModel>,
    model: Option<String>,
    prompt: Prompt,
    mut on_token: impl FnMut(&str) + Send,
) -> Result<LLMResult> {
    let config = get_config()?;

    let (model_config, prompt_model) = config
        .model(vendor)
        .ok_or_else(|| anyhow!("No model selected. Run `gitbuddy config` first."))?;

    let model_name = model.unwrap_or_else(|| model_config.model.clone());
    let api_key = model_config.api_key.clone().unwrap_or_default();
    let base_url = model_config.base_url.clone().unwrap_or_default();
    let option = config.model_params();

    let system_prompt = prompt.value().to_string();
    let user_prompt = build_user_prompt(diff_content);

    match prompt_model {
        PromptModel::MiniMax => {
            let base_url = if base_url.is_empty() {
                "https://api.minimaxi.com/anthropic"
            } else {
                &base_url
            };
            let client = rig::providers::anthropic::Client::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build()?;
            let model = client.completion_model(model_name.clone());
            stream_with_rig(model, &system_prompt, &user_prompt, option, &mut on_token).await
        }
        PromptModel::DeepSeek => {
            let base_url = if base_url.is_empty() {
                "https://api.deepseek.com"
            } else {
                &base_url
            };
            let client = rig::providers::openai::Client::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build()?;
            let completions_client = client.completions_api();
            let model = completions_client.completion_model(model_name.clone());
            stream_with_rig(model, &system_prompt, &user_prompt, option, &mut on_token).await
        }
        PromptModel::Ollama => {
            let base_url = if base_url.is_empty() {
                "http://localhost:11434"
            } else {
                &base_url
            };
            let client = rig::providers::openai::Client::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build()?;
            let completions_client = client.completions_api();
            let model = completions_client.completion_model(model_name.clone());
            stream_with_rig(model, &system_prompt, &user_prompt, option, &mut on_token).await
        }
        PromptModel::OpenAI => {
            let mut builder = rig::providers::openai::Client::builder().api_key(api_key);
            if !base_url.is_empty() {
                builder = builder.base_url(base_url);
            }
            let client = builder.build()?;
            let completions_client = client.completions_api();
            let model = completions_client.completion_model(model_name.clone());
            stream_with_rig(model, &system_prompt, &user_prompt, option, &mut on_token).await
        }
    }
}

pub(crate) fn build_user_prompt(diff_content: &str) -> String {
    format!("diff content: \n{diff_content}")
}

async fn stream_with_rig<M>(
    model: M,
    system_prompt: &str,
    user_prompt: &str,
    option: ModelParameters,
    on_token: &mut (impl FnMut(&str) + Send),
) -> Result<LLMResult>
where
    M: CompletionModel,
{
    let mut request = model
        .completion_request(user_prompt)
        .preamble(system_prompt.to_string());

    if option.temperature > 0.0 {
        request = request.temperature(option.temperature);
    }
    if option.max_tokens > 0 {
        request = request.max_tokens(option.max_tokens as u64);
    }

    let mut additional = serde_json::Map::new();
    if option.top_p > 0.0 {
        additional.insert("top_p".to_string(), serde_json::json!(option.top_p));
    }
    if option.top_k > 0 {
        additional.insert("top_k".to_string(), serde_json::json!(option.top_k));
    }
    if !additional.is_empty() {
        request = request.additional_params(serde_json::Value::Object(additional));
    }

    let request = request.build();

    let mut stream = model
        .stream(request)
        .await
        .map_err(|e| anyhow!("LLM streaming error: {}", e))?;

    let mut full_text = String::new();
    let mut reasoning_text = String::new();
    let mut usage_opt = None;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(StreamedAssistantContent::Text(text)) => {
                on_token(&text.text);
                full_text.push_str(&text.text);
            }
            Ok(StreamedAssistantContent::ReasoningDelta { reasoning, .. }) => {
                reasoning_text.push_str(&reasoning);
            }
            Ok(StreamedAssistantContent::Final(res)) => {
                usage_opt = res.token_usage();
            }
            Err(e) => return Err(anyhow!("Streaming error: {}", e)),
            _ => {}
        }
    }

    if full_text.trim().is_empty() && !reasoning_text.is_empty() {
        full_text = reasoning_text.clone();
    }

    let (prompt_tokens, completion_tokens, total_tokens, prompt_cache_hit_tokens) = match usage_opt {
        Some(u) => (
            u.input_tokens as i64,
            u.output_tokens as i64,
            u.total_tokens as i64,
            if u.cached_input_tokens > 0 {
                Some(u.cached_input_tokens as i64)
            } else {
                None
            },
        ),
        None => (0, 0, 0, None),
    };

    Ok(LLMResult {
        commit_message: full_text,
        completion_tokens,
        prompt_tokens,
        total_tokens,
        prompt_cache_hit_tokens,
        reasoning_content: if reasoning_text.is_empty() {
            None
        } else {
            Some(reasoning_text)
        },
    })
}

pub fn confirm_commit(_commit_message: &str) -> Result<bool> {
    print!("{} Commit with this message? [", "💾".yellow().bold());
    print!("{}", "Y".green().bold());
    print!("/");
    print!("{}", "n".red());
    print!("] ");
    let mut input = String::new();

    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim() == "y" || input.trim() == "Y" || input.trim() == "")
}
