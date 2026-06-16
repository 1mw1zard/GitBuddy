# GitBuddy

[![Rust CI](https://github.com/1mw1zard/GitBuddy/actions/workflows/rust.yaml/badge.svg)](https://github.com/1mw1zard/GitBuddy/actions/workflows/rust.yaml)
[![codecov](https://codecov.io/github/1mw1zard/gitbuddy/graph/badge.svg?token=PA0ZIXIGI5)](https://codecov.io/github/1mw1zard/gitbuddy)

GitBuddy is an AI-driven CLI that generates Conventional Commit messages from your staged Git diff. It can preview the
message, commit for you, optionally push, and fall back to auto-staging when you run `gitbuddy` with no subcommand.

> [!WARNING]
> This project is currently in **development**.

## Features

- **AI-powered commit subjects**: Generate a single Conventional Commit subject from the current staged diff.
- **JSON-first default prompt**: The default prompt asks models to return `{"subject":"..."}` for easier parsing.
- **Repair fallback**: If a model returns analysis, markdown, or malformed output, GitBuddy retries with a repair prompt.
- **Multiple vendors**: Supports DeepSeek, OpenAI-compatible endpoints, Ollama, and MiniMax.
- **Runtime overrides**: Temporarily switch vendor, model, or prompt preset without rewriting your saved config.
- **Smart diff truncation**: Uses the full diff for normal changes, compact summaries for large diffs, and stats for very
  large diffs.
- **Failure diagnostics**: Writes detailed invalid-response diagnostics to your GitBuddy config directory while keeping the
  terminal error short.

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

GitBuddy stores configuration in `~/.config/gitbuddy/config.toml`.

Configure a vendor with its default model:

```sh
gitbuddy config --api-key <your-api-key> deepseek
```

Configure a specific model:

```sh
gitbuddy config --api-key <your-api-key> --model gpt-4o openai
```

For local Ollama, the API key can be any non-empty placeholder:

```sh
gitbuddy config --api-key local --model llama3.1 ollama
```

## Usage

After staging your changes, run:

```sh
gitbuddy ai
```

Or run `gitbuddy` with no subcommand to auto-stage all changes and auto-commit without an extra confirmation prompt.

### Options

| Option | Description |
|--------|-------------|
| `--push` | Push the commit to the remote repository after committing. Without this flag, GitBuddy asks before pushing. |
| `--dry-run` | Generate the commit message and print the Git commands without creating a commit or pushing. |
| `--prompt <P>`, `-p <P>` | Select a built-in prompt preset: `P1`, `P2`, `P3`, `P4`, or `P5` (default: `P1`). |
| `--vendor <VENDOR>` | Temporarily override the default vendor for this run. |
| `--model <MODEL>`, `-m <MODEL>` | Temporarily override the model for this run. |

**Examples:**

```sh
# Auto-stage all changes, generate message, and auto-commit
gitbuddy

# Generate a message from staged changes, confirm before committing, and push
gitbuddy ai --push

# Preview the generated message without committing
gitbuddy ai --dry-run

# Use a different prompt style
gitbuddy ai -p P3

# Temporarily use a specific model
gitbuddy ai --vendor openai --model gpt-4o
```

## Commit Message Format

The default `P1` prompt expects the model to return a JSON object with a single `subject` field:

```json
{"subject":"fix(ai): repair malformed commit subject output"}
```

GitBuddy extracts and validates the subject before committing. Accepted commit types are `feat`, `fix`, `docs`, `style`,
`refactor`, `perf`, `test`, `chore`, `ci`, and `build`.

If the response cannot be parsed, GitBuddy sends a second repair request. If repair also fails, it writes a diagnostic log
such as `~/.config/gitbuddy/llm-diagnostic-<timestamp>.log` with the model name, prompt preset, original response, repair
response, and prompt previews.

## Support models

| Vendor   | Default Model        | Custom Model | Status |
|----------|----------------------|:------------:|:------:|
| DeepSeek | deepseek v4 flash    |      yes     |   ✓    |
| Ollama   | ollama               |      yes     |   ✓    |
| OpenAI   | gpt-3.5-turbo        |      yes     |   ✓    |
| MiniMax  | MiniMax-M2.7         |      yes     |   ✓    |

DeepSeek, OpenAI, and Ollama use OpenAI-compatible APIs. MiniMax uses an Anthropic-compatible API. All vendors support
custom model names via `gitbuddy config --model ...` or the runtime `--model` flag.

Default endpoints are:

| Vendor | Default endpoint |
|--------|------------------|
| DeepSeek | `https://api.deepseek.com` |
| Ollama | `http://localhost:11434` |
| MiniMax | `https://api.minimaxi.com/anthropic` |
| OpenAI | OpenAI SDK default endpoint |

## Troubleshooting

- **`Config not found. Run gitbuddy config first.`**: Configure at least one vendor before running `gitbuddy ai`.
- **`No files added to staging!`**: `gitbuddy ai` reads staged changes. Run `git add ...` first, or run `gitbuddy` to
  auto-stage all changes.
- **`LLM did not return a valid conventional commit subject.`**: Check the diagnostic file path printed in the terminal.
  The log is written under `~/.config/gitbuddy/` and includes the raw model output plus prompt previews.
- **Empty message from a reasoning model**: Use a standard chat model, for example `deepseek-chat`, instead of a model that
  returns only reasoning content.

## Roadmap

- [x] Enhance the User Interface.
- [x] Using configuration file instead of environment variables.
- [ ] Support for more AI models.
- [ ] Add statistics and analytics for GitBuddy usage of kinds of Models.
- [ ] Support HTTP proxy.
- [ ] Allow fully custom user-defined prompts.
- [ ] **Install** for using GitBuddy by **Git Hooks** (without `gitbuddy ai`).
- [ ] Submit a single request to receive multiple options for users to select from.
