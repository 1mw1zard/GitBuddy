# GitBuddy

[![Rust CI](https://github.com/fujianbang/GitBuddy/actions/workflows/rust.yaml/badge.svg)](https://github.com/fujianbang/GitBuddy/actions/workflows/rust.yaml)

GitBuddy is an AI-driven tool designed to simplify your Git commit process. With GitBuddy, you can generate meaningful
commit messages, streamline your workflow, and enhance your productivity.

> [!WARNING]
> This project is currently in **development**.

## Features

- **AI-Powered Commit Messages**: Generate intelligent and context-aware commit messages based on your code changes.
- **Customizable Models**: Support for using different AI models, not only GPT-3.5.
- **Multiple Vendor Flexibility**: Compatible with various AI service providers.
    + [x] [DeepSeek](https://www.deepseek.com/)
    + [ ] [OpenAI ChatGPT](https://platform.openai.com/docs/guides/gpt/chat-completions-api)
    + [ ] [Anthropic Claude](https://anthropic.com)
    + [ ] [Google PaLM2/Gemini](https://developers.generativeai.google)
    + [ ] [Mistral](https://mistral.ai/)
    + [ ] [ByteDance doubao](https://console.volcengine.com/ark/region:ark+cn-beijing/model)
    + [ ] [Baidu qianfan/wenxin](https://qianfan.cloud.baidu.com/)
    + [ ] [Alibaba TONGYI](https://tongyi.aliyun.com/)
    + [ ] [xfyun xinghuo](https://xinghuo.xfyun.cn/sparkapi)
    + [ ] [Zhipuai](https://open.bigmodel.cn/)
    + [ ] [360](https://ai.360.cn)
    + [ ] [Tencent Hunyuan](https://hunyuan.tencent.com/)
    + [ ] [Moonshot AI](https://platform.moonshot.cn/)
    + [ ] [Baichuan](https://platform.baichuan-ai.com)
    + [ ] [MINIMAX](https://api.minimax.chat/)
    + [ ] [Groq](https://wow.groq.com/)
    + [ ] [Ollama](https://github.com/ollama/ollama)
    + [ ] [01.ai lingyiwanwu](https://platform.lingyiwanwu.com/)
    + [ ] [Coze](https://www.coze.com/)
    + [ ] [Cloudflare Workers AI](https://developers.cloudflare.com/workers-ai/)
    + [ ] [DeepL](https://www.deepl.com/)
    + [ ] [together.ai](https://www.together.ai/)
- **Proxy Support**: Easily configure proxy settings for network-restricted environments.
- **Customizable Prompts**: Tailor the AI's suggestions to fit your project's specific needs.
- **Seamless Integration**: Works seamlessly with your existing Git workflow.
- **Improved Productivity**: Spend less time thinking about commit messages and more time coding.

## Installation

To get started with GitBuddy, follow these simple steps:

```sh
cargo install gitbuddy
```

## Usage

Using GitBuddy is straightforward. After making your changes, run the following command to generate a commit message:

```sh
gitbuddy ai
```

## Roadmap

- [ ] Support for more AI models.
- [ ] Add statistics and analytics for GitBuddy usage of kinds of Models.
- [ ] Easy configuration.
- [ ] Support Proxy.
- [ ] Better prompts.
- [ ] Enhance the User Interface.
- [ ] **Install** for using GitBuddy by **Git Hooks** (without `gitbuddy ai`).