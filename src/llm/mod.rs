mod openai_compatible;
mod openai_compatible_builder;

use anyhow::Result;
use clap::ValueEnum;
use openai_compatible_builder::OpenAICompatibleBuilder;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Prompt model
pub enum PromptModel {
    #[clap(name = "openai")]
    OpenAI,
    #[clap(name = "deepseek")]
    DeepSeek,
}

impl PromptModel {
    pub fn default_model(&self) -> String {
        return match self {
            PromptModel::OpenAI => {
                "gpt-3.5-turbo".to_string()
            }
            PromptModel::DeepSeek => {
                "deepseek-chat".to_string()
            }
        };
    }
}

// pub fn openai_request(diff_content: &str) -> std::io::Result<LLMResult> {
// }


fn get_commit_message(vendor: PromptModel, model: &str, api_key: &str, diff_content: &str) -> Result<String> {
    let builder = OpenAICompatibleBuilder::new(vendor, model, api_key);

    // generate http request
    let mut m = builder.build();
    m.request(diff_content)
}
