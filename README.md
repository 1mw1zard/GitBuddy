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

## Support models

| Vendor    | Model(s)                        | Support |
|-----------|---------------------------------|:-------:|
| DeepSeek  | deepseek-chat                   |   yes   |
| OpenAI    | gpt-3.5-turbo                   |   no    |
| ByteDance | Doubao-lite-4k<br>Doubao-pro-4k |   no    |
| Baidu     | ERNIE 4.0                       |   no    |
| Alibaba   | qwen-turbo                      |   no    |

## Roadmap

- [x] Enhance the User Interface.
- [ ] Using configuration file instead of environment variables.
- [ ] Support for more AI models.
- [ ] Add statistics and analytics for GitBuddy usage of kinds of Models.
- [ ] Support http proxy.
- [ ] Custom prompts.
- [ ] **Install** for using GitBuddy by **Git Hooks** (without `gitbuddy ai`).
- [ ] Submit a single request to receive multiple options for users to select from.