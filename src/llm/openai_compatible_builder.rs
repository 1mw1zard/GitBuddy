use crate::llm::openai_compatible::OpenAICompatible;
use crate::llm::PromptModel;

pub(crate) struct OpenAICompatibleBuilder {
    url: String,
    model: String,
    api_key: String,
}

const PROMPT: &str = "Generate a concise commit message based on \
            the following git difference content. The generated message is plain text,\
             does not contain identifiers such as markdown \"`\", \
             and the generated content does not exceed 100 tokens. \
             Depending on the nature of the change, it starts with one of the following prefixes:\
              'build' (build system), 'chore' (chores), 'ci' (continuous integration), \
              'docs' (documentation), 'feat' (new feature), 'fix' (fix), 'perf' (performance),\
               'refactor' (refactoring), 'style' (style), 'test' (test):";

impl OpenAICompatibleBuilder {
    pub fn new(vendor: PromptModel, model: &str, api_key: &str) -> Self {
        match vendor {
            PromptModel::OpenAI => OpenAICompatibleBuilder {
                url: String::from("https://api.openai.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            },
            PromptModel::DeepSeek => OpenAICompatibleBuilder {
                url: String::from("https://api.deepseek.com"),
                model: model.to_string(),
                api_key: api_key.to_string(),
            }
        }
    }

    pub fn build(self) -> OpenAICompatible {
        OpenAICompatible {
            url: self.url,
            model: self.model,
            prompt: PROMPT.parse().unwrap(),
            api_key: self.api_key,
        }
    }
}
