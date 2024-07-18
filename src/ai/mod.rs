mod git;
mod llm;

use std::thread::sleep;
use std::time::{Duration, Instant};
use colored::Colorize;
use crate::ai::git::{git_stage_diff, git_stage_filenames};

pub fn handler(dry_run: bool) {
    if !is_git_directory() {
        println!("Not git directory");
        return;
    }

    if !is_git_installed() {
        println!("Please install git");
        return;
    }

    let filenames = git_stage_filenames();
    if filenames.is_empty() {
        println!("No files added to staging! Did you forget to run `git add` ?");
        return;
    }

    let diff_content = git_stage_diff();

    println!("Generating commit message by LLM...");

    let start = Instant::now();
    let llm_result = llm::openai_request(&diff_content).unwrap();
    let duration = start.elapsed();

    let usage_message = format!(
        "Completed!  duration={:?} - Usage={}(completion={}, prompt={})]",
        duration, llm_result.total_tokens, llm_result.completion_tokens, llm_result.prompt_tokens);
    println!("{}", usage_message.green());

    if !llm::confirm_commit(llm_result.commit_message.as_str()) {
        println!("cancel commit");
        return;
    }

    git::git_commit(llm_result.commit_message.trim(), dry_run);
}

fn is_git_directory() -> bool {
    // 获取当前目录
    let current_dir = std::env::current_dir().unwrap();

    // 检查当前目录是否为git目录
    return if current_dir.join(".git").exists() {
        true
    } else {
        false
    };
}

fn is_git_installed() -> bool {
    // 检查是否安装git
    return if std::process::Command::new("git").arg("--version").output().is_ok() {
        true
    } else {
        false
    };
}
