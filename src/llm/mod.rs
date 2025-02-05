mod openai_compatible;
mod openai_compatible_builder;

use crate::config;
use crate::config::UseModel;
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use colored::Colorize;
use openai_compatible_builder::OpenAICompatibleBuilder;
use std::io::Write;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Prompt model
pub enum PromptModel {
    #[clap(name = "openai")]
    OpenAI,
    #[clap(name = "deepseek")]
    DeepSeek,
    #[clap(name = "ollama")]
    Ollama,
}

impl PromptModel {
    pub fn default_model(&self) -> String {
        return match self {
            PromptModel::OpenAI => "gpt-3.5-turbo".to_string(),
            PromptModel::DeepSeek => "deepseek-chat".to_string(),
            PromptModel::Ollama => "ollama".to_string(),
        };
    }
}

#[derive(Debug)]
pub struct LLMResult {
    pub commit_message: String,
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
}

struct RequestsWrap {
    vendor: PromptModel,
    model: String,
    api_key: String,
}

impl RequestsWrap {
    fn new(vendor: PromptModel, model: String, api_key: String) -> Self {
        RequestsWrap {
            vendor,
            model,
            api_key,
        }
    }
}

pub fn llm_request(diff_content: &str) -> Result<LLMResult> {
    let config = config::get_config()?;

    let model = match config.model {
        Some(model) => model,
        None => {
            return Err(anyhow!("No model selected"));
        }
    };

    let RequestsWrap {
        vendor,
        model,
        api_key,
    } = match model {
        UseModel::DeepSeek(params) => {
            RequestsWrap::new(PromptModel::DeepSeek, params.model, params.api_key)
        }
        UseModel::OpenAI(params) => {
            RequestsWrap::new(PromptModel::OpenAI, params.model, params.api_key)
        }
        UseModel::Ollama(params) => {
            RequestsWrap::new(PromptModel::Ollama, params.model, params.api_key)
        }
    };

    get_commit_message(vendor, model.as_str(), api_key.as_str(), diff_content)
}

fn get_commit_message(
    vendor: PromptModel,
    model: &str,
    api_key: &str,
    diff_content: &str,
) -> Result<LLMResult> {
    let builder = OpenAICompatibleBuilder::new(vendor, model, api_key);

    // generate http request
    let m = builder.build();
    let result = m.request(diff_content)?;
    Ok(result)
}

pub fn confirm_commit(commit_message: &str) -> bool {
    println!("--------------------------------------");
    println!("{}", commit_message.cyan().bold());
    println!("--------------------------------------");
    print!("Are you sure you want to commit? (Y/n) ");
    let mut input = String::new();

    // flush
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    return input.trim() == "y" || input.trim() == "Y" || input.trim() == "";
}
