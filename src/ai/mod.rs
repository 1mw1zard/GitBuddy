use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::ai::git::{git_stage_diff, git_stage_filenames, git_stage_stats, has_unstaged_changes, git_add_all};
use crate::llm;

mod git;

/// L1 阈值：完整 diff 的安全上限（字符数）
const MAX_DIFF_CHARS_FULL: usize = 8_000;
/// L2 阈值：保留 header + 部分代码的上限（字符数）
const MAX_DIFF_CHARS_SUMMARY: usize = 30_000;
/// L2 模式下每个文件保留的最大代码行数
const MAX_LINES_PER_FILE: usize = 20;

pub fn handler(push: bool, dry_run: bool, auto_stage: bool, auto_commit: bool) -> Result<()> {
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
                println!("No staged changes found. Auto-staging all changes...");
                git_add_all()?;
                println!("{}", "Changes staged.".green());
            } else {
                println!("No changes to commit.");
                return Ok(());
            }
        } else {
            println!("No files added to staging! Did you forget to run `git add` ?");
            return Ok(());
        }
    }

    let diff_content = git_stage_diff()?;
    let prompt_content = build_prompt(&diff_content)?;

    println!("Generating commit message by LLM...");

    let start = Instant::now();
    let llm_result = llm::llm_request(&prompt_content)?;
    let duration = start.elapsed();

    // 流式逐字输出 commit message
    println!("--------------------------------------");
    let colored_msg = llm_result.commit_message.cyan().bold().to_string();
    typewriter_print(&colored_msg, 25)?;
    println!();
    println!("--------------------------------------");

    let usage_message = format!(
        "duration={:?} - Usage={}(completion={}, prompt={})]",
        duration, llm_result.total_tokens, llm_result.completion_tokens, llm_result.prompt_tokens
    );
    println!("{}", usage_message.truecolor(128, 128, 128));

    if !auto_commit && !llm::confirm_commit()? {
        println!("{}", "Cancel commit".red());
        return Ok(());
    }

    git::git_commit(llm_result.commit_message.trim(), dry_run)?;
    println!("{}", "Commit success!!!".green().bold());

    // push
    if push {
        git::git_push(dry_run)?;
        println!("{}", "Push success!!!".green());
    }

    Ok(())
}

/// 三层降级策略构建发送给 LLM 的 prompt：
///
/// - L1 Full:    diff 字符数 <= 8K   → 发送完整 diff
/// - L2 Summary: diff 字符数 8K~30K → 保留 diff header，每个文件只留前 20 行代码
/// - L3 Stats:   diff 字符数 > 30K   → 只发送 `git diff --stat` 的统计信息
fn build_prompt(diff: &str) -> Result<String> {
    // 快速路径：字节层面未超限
    if diff.len() <= MAX_DIFF_CHARS_FULL {
        return Ok(diff.to_string());
    }

    let char_count = diff.chars().count();

    // L1: 完整模式
    if char_count <= MAX_DIFF_CHARS_FULL {
        return Ok(diff.to_string());
    }

    // L2: 摘要模式 - 保留 header，每个文件只留前 N 行代码
    if char_count <= MAX_DIFF_CHARS_SUMMARY {
        let summary = smart_truncate_diff(diff, MAX_LINES_PER_FILE);
        println!(
            "{}",
            format!(
                "Note: Diff is large ({} chars), showing file headers with up to {} code lines per file.",
                char_count, MAX_LINES_PER_FILE
            )
            .yellow()
        );
        return Ok(summary);
    }

    // L3: 极简模式 - 使用 git diff --stat
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
        // 兜底：如果连 stats 都拿不到，至少给出文件名列表
        let filenames = git_stage_filenames()?.join("\n");
        Ok(format!(
            "Generate a concise commit message for changes in these files:\n\n{}",
            filenames
        ))
    }
}

/// 智能截断：保留所有 diff header（文件名、hunk 位置等），
/// 每个文件最多保留 `max_lines` 行实际代码（以 `+` `-` ` ` 开头）。
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
            // hunk header 保留，不消耗代码行配额
            result.push_str(line);
            result.push('\n');
        } else if line.starts_with('+') || line.starts_with('-') || line.starts_with(' ') {
            // 实际代码行
            if lines_remaining > 0 {
                result.push_str(line);
                result.push('\n');
                lines_remaining -= 1;
            } else {
                skipped_any = true;
            }
        } else {
            // 其他 header 行（index、---、+++ 等）
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

fn typewriter_print(text: &str, delay_ms: u64) -> Result<()> {
    for ch in text.chars() {
        print!("{}", ch);
        std::io::stdout().flush()?;
        thread::sleep(Duration::from_millis(delay_ms));
    }
    Ok(())
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
