use std::time::Instant;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::ai::git::{git_add_all, git_stage_diff, git_stage_filenames, git_stage_stats, has_unstaged_changes};
use crate::llm;
use crate::llm::PromptModel;
use crate::prompt::Prompt;

mod git;

/// Safe character limit for sending the full diff.
const MAX_DIFF_CHARS_FULL: usize = 8_000;
/// Character limit for sending diff headers plus selected code lines.
const MAX_DIFF_CHARS_SUMMARY: usize = 30_000;
/// Maximum number of code lines retained per file in summary mode.
const MAX_LINES_PER_FILE: usize = 20;

pub fn handler(
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

    println!("{}", "🤖 GitBuddy".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".truecolor(128, 128, 128));
    println!();

    let start = Instant::now();
    println!("🧠 Analyzing code changes...");
    let llm_result = llm::llm_request(&prompt_content, vendor, model, prompt)?;
    let duration = start.elapsed();
    println!(
        "{} {}",
        "🎯 Model:".truecolor(128, 128, 128),
        llm_result.model.cyan()
    );

    let msg = llm_result.commit_message.trim();
    let max_line_width = msg.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let inner_width = max_line_width.max(40);

    println!();
    println!("  ╭{}╮", "─".repeat(inner_width + 2));
    for line in msg.lines() {
        let padded = format!("{:<width$}", line, width = inner_width);
        println!("  │ {} │", padded.cyan().bold());
    }
    println!("  ╰{}╯", "─".repeat(inner_width + 2));

    let duration_str = if duration.as_secs() >= 1 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    };

    println!();
    println!(
        "{}",
        format!(
            "⏱  {}  ·  🪙  {} tokens  ·  📝  {} prompt  ·  ✨  {} completion",
            duration_str,
            llm_result.total_tokens,
            llm_result.prompt_tokens,
            llm_result.completion_tokens
        )
        .truecolor(128, 128, 128)
    );
    println!();

    if !auto_commit && !llm::confirm_commit(&llm_result.commit_message)? {
        println!("{} {}", "❌".red().bold(), "Commit cancelled".red());
        return Ok(());
    }

    git::git_commit(llm_result.commit_message.trim(), dry_run)?;
    println!(
        "{} {}",
        "✅".green().bold(),
        "Commit successful".green().bold()
    );

    if push {
        git::git_push(dry_run)?;
        println!(
            "{} {}",
            "🚀".green().bold(),
            "Push successful".green().bold()
        );
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
