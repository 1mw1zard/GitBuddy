use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::ai::git::{git_add_all, git_stage_diff, git_stage_filenames, git_stage_stats, has_unstaged_changes};
use crate::commit_message::extract_commit_subject_from_json_or_text;
use crate::config;
use crate::llm;
use crate::llm::PromptModel;
use crate::prompt::Prompt;

mod git;

/// Safe character limit for sending the full diff.
const MAX_DIFF_CHARS_FULL: usize = 30_000;
/// Character limit for sending diff headers plus selected code lines.
const MAX_DIFF_CHARS_SUMMARY: usize = 100_000;
/// Maximum number of code lines retained per file in summary mode.
const MAX_LINES_PER_FILE: usize = 20;

pub async fn handler(
    push: bool,
    dry_run: bool,
    auto_stage: bool,
    auto_commit: bool,
    vendor: Option<PromptModel>,
    model: Option<String>,
    prompt: Prompt,
) -> Result<()> {
    if !is_git_installed() {
        return Err(anyhow!("Git is not installed. Please install git first."));
    }

    if !is_git_directory()? {
        return Err(anyhow!("Not a git directory"));
    }

    let filenames = git_stage_filenames()?;
    if filenames.is_empty() {
        if auto_stage {
            if has_unstaged_changes()? {
                println!("📦 No staged changes found. Auto-staging all changes...");
                git_add_all()?;
                println!("{} {}", "✅".green().bold(), "Changes staged.".green());
            } else {
                println!("ℹ️  No changes to commit.");
                return Ok(());
            }
        } else {
            println!("⚠️  No files added to staging! Did you forget to run `git add`?");
            return Ok(());
        }
    }

    let diff_content = git_stage_diff()?;
    let stats = git_stage_stats().unwrap_or_default();
    let filenames = git_stage_filenames()?.join("\n");

    let enriched = format!(
        "Files changed:\n{}\n\nChange statistics:\n{}\n\n{}",
        filenames, stats, diff_content
    );
    let prompt_content = build_prompt(&enriched)?;

    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "{}",
        "  ____ _ _   ____            _     _       ".truecolor(128, 128, 128)
    );
    println!(
        "{}",
        " / ___(_) |_| __ ) _   _  __| | __| |_   _ ".truecolor(128, 128, 128)
    );
    println!(
        "{}",
        "| |  _| | __|  _ \\| | | |/ _` |/ _` | | | |".truecolor(128, 128, 128)
    );
    println!(
        "{}",
        "| |_| | | |_| |_) | |_| | (_| | (_| | |_| |".truecolor(128, 128, 128)
    );
    println!(
        "{}",
        " \\____|_|\\__|____/ \\__,_|\\__,_|\\__,_|\\__, |".truecolor(128, 128, 128)
    );
    print!("{}", "                              ".truecolor(128, 128, 128));
    print!("{}", version.yellow().bold());
    println!("{}", " |___/ ".truecolor(128, 128, 128));
    println!();

    // Pre-resolve the model name so it is printed before streaming starts.
    let resolved_model = config::get_config()
        .ok()
        .and_then(|cfg| {
            let (mc, _) = cfg.model(vendor)?;
            Some(model.clone().unwrap_or_else(|| mc.model.clone()))
        })
        .unwrap_or_else(|| {
            vendor
                .map(|v| v.default_model())
                .unwrap_or_else(|| "unknown".to_string())
        });
    println!("{} {}", "🎯 Model:".truecolor(128, 128, 128), resolved_model.cyan());
    println!();

    let start = Instant::now();
    println!("🧠 Analyzing code changes...");
    let stat_summary = stats.lines().last().unwrap_or("").trim();
    if !stat_summary.is_empty() {
        println!("📊 {}", stat_summary.truecolor(128, 128, 128));
        println!();
    }

    let user_prompt = llm::build_user_prompt(&prompt_content);
    let llm_result = llm::llm_request(&prompt_content, vendor, model.clone(), prompt, |token| {
        for ch in token.chars() {
            print!("{}", ch.to_string().cyan().bold());
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(5));
        }
    })
    .await?;

    let duration = start.elapsed();
    println!();
    println!();

    if llm_result.commit_message.trim().is_empty() {
        eprintln!("{}", "⚠️  LLM returned an empty commit message.".red().bold());
        if let Some(ref reasoning) = llm_result.reasoning_content {
            eprintln!("{}", "ℹ️  The model produced reasoning content instead:".yellow());
            eprintln!("{}", reasoning.cyan());
        }
        return Err(anyhow!(
            "Empty commit message. This usually happens with reasoning models (e.g., DeepSeek-R1) \
             that output thinking tokens in a separate field. \
             Try using a standard chat model like 'deepseek-chat'."
        ));
    }
    let msg = match extract_commit_subject_from_json_or_text(&llm_result.commit_message) {
        Ok(msg) => msg,
        Err(err) => {
            eprintln!(
                "{}",
                "⚠️  LLM response did not match the required format. Attempting repair...".yellow()
            );
            let repair_prompt =
                llm::build_repair_user_prompt(&llm_result.commit_message, &user_prompt, &err.to_string());
            let repair_result = match llm::repair_commit_subject_request(
                &llm_result.commit_message,
                &user_prompt,
                &err.to_string(),
                vendor,
                model.clone(),
            )
            .await
            {
                Ok(repair_result) => repair_result,
                Err(repair_err) => {
                    return Err(invalid_commit_subject_error(
                        &format!("{}\nRepair request failed: {}", err, repair_err),
                        &llm_result.commit_message,
                        None,
                        Some(&repair_prompt),
                        &user_prompt,
                        prompt,
                        &resolved_model,
                    ));
                }
            };

            extract_commit_subject_from_json_or_text(&repair_result.commit_message).map_err(|repair_err| {
                invalid_commit_subject_error(
                    &format!("{}\nRepair attempt also failed: {}", err, repair_err),
                    &llm_result.commit_message,
                    Some(&repair_result.commit_message),
                    Some(&repair_prompt),
                    &user_prompt,
                    prompt,
                    &resolved_model,
                )
            })?
        }
    };
    print_commit_message(&msg)?;

    let duration_str = if duration.as_secs() >= 1 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    };

    let cached = llm_result.prompt_cache_hit_tokens.unwrap_or(0);
    let cached_part = if cached > 0 {
        format!(" (+ {} cached)", cached)
    } else {
        String::new()
    };

    println!(
        "{} {}",
        "⏱".truecolor(128, 128, 128),
        duration_str.truecolor(128, 128, 128)
    );
    println!(
        "{}",
        format!(
            "Token usage: total={} input={}{} output={}",
            llm_result.total_tokens, llm_result.prompt_tokens, cached_part, llm_result.completion_tokens
        )
        .truecolor(128, 128, 128)
    );
    println!();

    if !auto_commit && !llm::confirm_commit(&msg)? {
        println!("{} {}", "❌".red().bold(), "Commit cancelled".red());
        return Ok(());
    }

    git::git_commit(&msg, dry_run)?;

    let should_push = if push {
        true
    } else {
        print!("{} Push to remote? [", "🚀".yellow().bold());
        print!("{}", "Y".green().bold());
        print!("/");
        print!("{}", "n".red());
        print!("] ");
        let mut input = String::new();
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;
        let line = input.trim_end_matches('\n').trim_end_matches('\r');
        line == "y" || line == "Y" || line.is_empty()
    };

    if should_push {
        git::git_push(dry_run)?;
    }

    Ok(())
}

/// Build the LLM prompt using a three-level fallback strategy:
///
/// - L1 Full: send the full diff when it is at most 8K characters.
/// - L2 Summary: keep diff headers and up to 20 code lines per file.
/// - L3 Stats: use only `git diff --stat` when the diff is too large.
fn build_prompt(diff: &str) -> Result<String> {
    // Fast path for ASCII-heavy diffs.
    if diff.len() <= MAX_DIFF_CHARS_FULL {
        return Ok(diff.to_string());
    }

    let char_count = diff.chars().count();

    // L1: full diff mode.
    if char_count <= MAX_DIFF_CHARS_FULL {
        return Ok(diff.to_string());
    }

    // L2: summary mode with headers and a limited number of code lines.
    if char_count <= MAX_DIFF_CHARS_SUMMARY {
        let summary = smart_truncate_diff(diff, MAX_LINES_PER_FILE);
        println!(
            "{}",
            format!(
                "📄 Note: Diff is large ({} chars), showing file headers with up to {} code lines per file.",
                char_count, MAX_LINES_PER_FILE
            )
            .yellow()
        );
        return Ok(summary);
    }

    // L3: compact mode using git diff --stat.
    println!(
        "{}",
        format!(
            "Note: Diff is very large ({} chars), using file statistics only.",
            char_count
        )
        .yellow()
    );

    let stats = git_stage_stats().unwrap_or_default();
    if !stats.is_empty() {
        Ok(format!(
            "Generate a concise commit message based on the following file change statistics:\n\n{}",
            stats
        ))
    } else {
        // Fall back to filenames if stats are unavailable.
        let filenames = git_stage_filenames()?.join("\n");
        Ok(format!(
            "Generate a concise commit message for changes in these files:\n\n{}",
            filenames
        ))
    }
}

/// Keep all diff headers and retain up to `max_lines` code lines per file.
fn smart_truncate_diff(diff: &str, max_lines: usize) -> String {
    let mut result = String::new();
    let mut lines_remaining = 0;
    let mut skipped_any = false;

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            lines_remaining = max_lines;
            result.push_str(line);
            result.push('\n');
        } else if line.starts_with("@@ ") {
            // Keep hunk headers without consuming the code-line budget.
            result.push_str(line);
            result.push('\n');
        } else if line.starts_with('+') || line.starts_with('-') || line.starts_with(' ') {
            // Actual code lines.
            if lines_remaining > 0 {
                result.push_str(line);
                result.push('\n');
                lines_remaining -= 1;
            } else {
                skipped_any = true;
            }
        } else {
            // Other header lines, such as index, ---, and +++.
            result.push_str(line);
            result.push('\n');
        }
    }

    if skipped_any {
        format!(
            "{}\n[Note: Diff was smart-truncated (max {} code lines per file).]",
            result.trim_end(),
            max_lines
        )
    } else {
        result
    }
}

fn terminal_width() -> Option<usize> {
    std::process::Command::new("tput")
        .arg("cols")
        .output()
        .ok()
        .and_then(|output| String::from_utf8_lossy(&output.stdout).trim().parse().ok())
}

fn print_commit_message(msg: &str) -> Result<()> {
    let max_line_width = msg.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let inner_width = max_line_width.max(40);
    let box_width = inner_width + 6; // 2 indent + 2 border + 2 padding

    let term_w = terminal_width().unwrap_or(120);

    if box_width <= term_w {
        // Terminal is wide enough: draw rounded box with typewriter effect.
        println!("  ╭{}╮", "─".repeat(inner_width + 2));
        for line in msg.lines() {
            let padded = format!("{:<width$}", line, width = inner_width);
            print!("  │ ");
            let colored = padded.cyan().bold().to_string();
            for ch in colored.chars() {
                print!("{}", ch);
                std::io::stdout().flush()?;
                thread::sleep(Duration::from_millis(12));
            }
            println!(" │");
        }
        println!("  ╰{}╯", "─".repeat(inner_width + 2));
    } else {
        // Terminal is too narrow: skip the box, just typewriter-print.
        println!();
        for line in msg.lines() {
            let colored = line.cyan().bold().to_string();
            for ch in colored.chars() {
                print!("{}", ch);
                std::io::stdout().flush()?;
                thread::sleep(Duration::from_millis(12));
            }
            println!();
        }
        println!();
    }
    Ok(())
}

fn invalid_commit_subject_error(
    cause: &str,
    raw_response: &str,
    repair_response: Option<&str>,
    repair_prompt: Option<&str>,
    user_prompt: &str,
    prompt: Prompt,
    model: &str,
) -> anyhow::Error {
    match config::storage::get_config_dir() {
        Some(config_dir) => invalid_commit_subject_error_to_dir(
            cause,
            raw_response,
            repair_response,
            repair_prompt,
            user_prompt,
            prompt,
            model,
            &config_dir,
        ),
        None => anyhow!(
            "{}\nDiagnostic file was not written: failed to resolve config directory.",
            cause
        ),
    }
}

fn invalid_commit_subject_error_to_dir(
    cause: &str,
    raw_response: &str,
    repair_response: Option<&str>,
    repair_prompt: Option<&str>,
    user_prompt: &str,
    prompt: Prompt,
    model: &str,
    config_dir: &Path,
) -> anyhow::Error {
    let content = format_commit_subject_diagnostic(
        cause,
        raw_response,
        repair_response,
        repair_prompt,
        user_prompt,
        prompt,
        model,
    );
    match write_commit_subject_diagnostic(config_dir, &content) {
        Ok(path) => anyhow!("{}\nDiagnostic written to: {}", cause, path.display()),
        Err(err) => anyhow!("{}\nDiagnostic file was not written: {}", cause, err),
    }
}

fn format_commit_subject_diagnostic(
    cause: &str,
    raw_response: &str,
    repair_response: Option<&str>,
    repair_prompt: Option<&str>,
    user_prompt: &str,
    prompt: Prompt,
    model: &str,
) -> String {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let repair_section = repair_response
        .map(|repair| format!("\n\nRepair LLM response:\n{}", preview(repair)))
        .unwrap_or_default();
    let repair_prompt_section = repair_prompt
        .map(|repair_prompt| format!("\n\nRepair prompt:\n{}", preview(repair_prompt)))
        .unwrap_or_default();

    format!(
        "GitBuddy LLM diagnostic\nCreated at (unix seconds): {}\n\n{}\n\nModel: {}\nPrompt preset: {}\n\nOriginal LLM response:\n{}{}{}\n\nOriginal LLM request:\nSystem prompt:\n{}\n\nUser prompt:\n{}",
        created_at,
        cause,
        model,
        prompt,
        preview(raw_response),
        repair_section,
        repair_prompt_section,
        preview(prompt.value()),
        preview(user_prompt)
    )
}

fn write_commit_subject_diagnostic(config_dir: &Path, content: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(config_dir)
        .map_err(|err| anyhow!("failed to create config directory '{}': {}", config_dir.display(), err))?;

    let path = config_dir.join(format!("llm-diagnostic-{}.log", diagnostic_timestamp()));
    std::fs::write(&path, content)
        .map_err(|err| anyhow!("failed to write diagnostic file '{}': {}", path.display(), err))?;

    Ok(path)
}

fn diagnostic_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn preview(value: &str) -> String {
    const MAX_CHARS: usize = 50_000;
    let mut chars = value.chars();
    let preview: String = chars.by_ref().take(MAX_CHARS).collect();

    if chars.next().is_some() {
        format!("{preview}\n[truncated after {MAX_CHARS} chars]")
    } else {
        preview
    }
}

fn is_git_directory() -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "true")
}

fn is_git_installed() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_subject_diagnostic_includes_llm_request_and_response_context() {
        let content = format_commit_subject_diagnostic(
            "LLM did not return a valid conventional commit subject.",
            "Here is my analysis instead.",
            Some(r#"{"subject":"fix(ai): repair malformed response"}"#),
            Some(r#"Return only JSON in this exact shape: {"subject":"<type>(<scope>): <subject>"}"#),
            "diff content: \ndiff --git a/src/main.rs b/src/main.rs",
            Prompt::P1,
            "deepseek-chat",
        );

        assert!(content.contains("LLM did not return a valid conventional commit subject."));
        assert!(content.contains("Model: deepseek-chat"));
        assert!(content.contains("Prompt preset: p1"));
        assert!(content.contains("Original LLM response:"));
        assert!(content.contains("Here is my analysis instead."));
        assert!(content.contains("Repair LLM response:"));
        assert!(content.contains(r#"{"subject":"fix(ai): repair malformed response"}"#));
        assert!(content.contains("Repair prompt:"));
        assert!(content.contains("Return only JSON in this exact shape"));
        assert!(content.contains("Original LLM request:"));
        assert!(content.contains("System prompt:"));
        assert!(content.contains("User prompt:"));
        assert!(content.contains("diff --git a/src/main.rs b/src/main.rs"));
    }

    #[test]
    fn diagnostic_preview_truncates_large_values() {
        let value = "a".repeat(50_001);

        let preview = preview(&value);

        let (body, notice) = preview.split_once('\n').unwrap();
        assert_eq!(body.len(), 50_000);
        assert!(body.chars().all(|ch| ch == 'a'));
        assert_eq!(notice, "[truncated after 50000 chars]");
        assert!(preview.ends_with("[truncated after 50000 chars]"));
    }

    #[test]
    fn commit_subject_error_writes_verbose_context_to_diagnostic_file() {
        let temp_dir = std::env::temp_dir().join("gitbuddy-diagnostic-test").join(uuid());
        std::fs::create_dir_all(&temp_dir).unwrap();

        let err = invalid_commit_subject_error_to_dir(
            "LLM did not return a valid conventional commit subject.",
            "Here is my analysis instead.",
            Some(r#"{"subject":"fix(ai): repair malformed response"}"#),
            Some(r#"Return only JSON in this exact shape: {"subject":"<type>(<scope>): <subject>"}"#),
            "diff content: \ndiff --git a/src/main.rs b/src/main.rs",
            Prompt::P1,
            "deepseek-chat",
            &temp_dir,
        );

        let msg = err.to_string();
        assert!(msg.contains("LLM did not return a valid conventional commit subject."));
        assert!(msg.contains("Diagnostic written to:"));
        assert!(msg.contains(temp_dir.to_string_lossy().as_ref()));
        assert!(!msg.contains("Here is my analysis instead."));
        assert!(!msg.contains("diff --git a/src/main.rs b/src/main.rs"));

        let files: Vec<_> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].extension().and_then(|ext| ext.to_str()), Some("log"));

        let content = std::fs::read_to_string(&files[0]).unwrap();
        assert!(content.contains("Model: deepseek-chat"));
        assert!(content.contains("Prompt preset: p1"));
        assert!(content.contains("Original LLM response:"));
        assert!(content.contains("Here is my analysis instead."));
        assert!(content.contains("Repair LLM response:"));
        assert!(content.contains(r#"{"subject":"fix(ai): repair malformed response"}"#));
        assert!(content.contains("Repair prompt:"));
        assert!(content.contains("Return only JSON in this exact shape"));
        assert!(content.contains("Original LLM request:"));
        assert!(content.contains("diff --git a/src/main.rs b/src/main.rs"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    fn uuid() -> String {
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("{}", dur.as_millis())
    }
}
