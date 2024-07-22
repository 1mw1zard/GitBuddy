mod openai_compatible;

use anyhow::Result;
use clap::ValueEnum;

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

fn get_commit_message(model: PromptModel, api_key: &str, diff_content: &str) -> Result<String> {
    let builder = match model {
        PromptModel::OpenAI => openai_compatible::OpenAICompatibleBuilder::default(),
        PromptModel::DeepSeek => {
            openai_compatible::OpenAICompatibleBuilder::default().
                url(String::from("https://api.deepseek.com")).
                use_model(String::from("deepseek-chat"))
        }
    };


    // TODO update the prompt or something else

    let mut m = builder.build();
    m.set_api_key(api_key);

    m.request(diff_content)
}
