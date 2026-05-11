# GitBuddy

[![Rust CI](https://github.com/1mw1zard/GitBuddy/actions/workflows/rust.yaml/badge.svg)](https://github.com/1mw1zard/GitBuddy/actions/workflows/rust.yaml)
[![codecov](https://codecov.io/github/1mw1zard/gitbuddy/graph/badge.svg?token=PA0ZIXIGI5)](https://codecov.io/github/1mw1zard/gitbuddy)

GitBuddy is an AI-driven tool designed to simplify your Git commit process. With GitBuddy, you can generate meaningful
commit messages, streamline your workflow, and enhance your productivity.

> [!WARNING]
> This project is currently in **development**.

## Features

- **AI-Powered Commit Messages**: Generate intelligent and context-aware commit messages based on your code changes.
- **Customizable Models**: Support for using different AI models, not only GPT-3.5.
- **Multiple Vendor Flexibility**: Compatible with various AI service providers.
- **Multiple Built-in Prompts**: Choose from 5 built-in prompt presets (P1–P5) to tailor commit message style.
- **Smart Diff Truncation**: Automatically truncates large diffs to fit within model context windows.
- **Seamless Integration**: Works seamlessly with your existing Git workflow.
- **Improved Productivity**: Spend less time thinking about commit messages and more time coding.

## Installation

### From crates.io

```sh
cargo install gitbuddy
```

To update if you already have installed, run:

```sh
cargo install --force gitbuddy
```

> 📦 [crates.io/crates/gitbuddy](https://crates.io/crates/gitbuddy)
>
> 🍺 **Homebrew support is coming soon.**

### Configuration

To use GitBuddy, simply run the following command in your terminal:

**Using default model**

```sh
gitbuddy config --api-key <your-api-key> deepseek
```

**Using custom model**

```sh
gitbuddy config --api-key <your-api-key> --model gpt-4o openai
```

## Usage

Using GitBuddy is straightforward. After making your changes, run the following command to generate a commit message:

```sh
gitbuddy ai
```

Or simply run `gitbuddy` for auto-stage and auto-commit mode.

### Options

| Option | Description |
|--------|-------------|
| `--push` | Push the commit to the remote repository after committing. |
| `--dry-run` | Generate the commit message without actually creating the commit. |
| `--prompt <P>`, `-p <P>` | Select a built-in prompt preset: `P1`, `P2`, `P3`, `P4`, or `P5` (default: `P1`). |
| `--vendor <VENDOR>` | Temporarily override the default vendor for this run. |
| `--model <MODEL>`, `-m <MODEL>` | Temporarily override the model for this run. |

**Examples:**

```sh
# Auto-stage all changes, generate message, and auto-commit
gitbuddy

# Generate message from staged changes, confirm, commit, and push
gitbuddy ai --push

# Preview the generated message without committing
gitbuddy ai --dry-run

# Use a different prompt style
gitbuddy ai -p P3

# Temporarily use a specific model
gitbuddy ai --vendor openai --model gpt-4o
```

## Support models

| Vendor   | Default Model        | Custom Model | Status |
|----------|----------------------|:------------:|:------:|
| DeepSeek | deepseek v4 flash    |      yes     |   ✓    |
| Ollama   | ollama               |      yes     |   ✓    |
| OpenAI   | gpt-3.5-turbo        |      yes     |   ✓    |
| MiniMax  | MiniMax-M2.7         |      yes     |   ✓    |

> All vendors support arbitrary model names via the `--model` flag, as long as the endpoint is OpenAI-compatible. MiniMax uses Anthropic-compatible API.

## Roadmap

- [x] Enhance the User Interface.
- [x] Using configuration file instead of environment variables.
- [ ] Support for more AI models.
- [ ] Add statistics and analytics for GitBuddy usage of kinds of Models.
- [ ] Support HTTP proxy.
- [ ] Allow fully custom user-defined prompts.
- [ ] **Install** for using GitBuddy by **Git Hooks** (without `gitbuddy ai`).
- [ ] Submit a single request to receive multiple options for users to select from.
