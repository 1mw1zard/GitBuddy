mod openai_compatible;

use anyhow::Result;

/// Prompt model
enum PromptModel {
    OpenAI,
    DeepSeek,
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
