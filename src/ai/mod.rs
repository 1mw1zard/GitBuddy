use std::time::Instant;

use colored::Colorize;

use crate::ai::git::{git_stage_diff, git_stage_filenames};
use crate::llm;
use crate::llm::PromptModel;

mod git;

pub fn handler(push: bool, dry_run: bool, vendor: Option<PromptModel>, model: Option<String>) {
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
    // let diff_content = format!("Code changes: \n```\n{}\n```", git_stage_diff());

    println!("Generating commit message by LLM...");

    let start = Instant::now();
    let llm_result = llm::llm_request(&diff_content, vendor, model).unwrap();
    let duration = start.elapsed();

    let usage_message = format!("duration={:?} - Usage={}(completion={}, prompt={})]", duration, llm_result.total_tokens, llm_result.completion_tokens, llm_result.prompt_tokens);

    println!("{}  {}", "Completed!".green(), usage_message.truecolor(128, 128, 128));

    if !llm::confirm_commit(llm_result.commit_message.as_str()) {
        println!("{}", "Cancel commit".red());
        return;
    }

    let result = git::git_commit(llm_result.commit_message.trim(), dry_run);
    match result {
        Ok(_) => {
            println!("{}", "Commit success!!!".green().bold());
        }
        Err(e) => {
            eprintln!("{}", e)
        }
    }

    // push
    if push {
        match git::git_push(dry_run) {
            Ok(_) => {
                println!("{}", "Push success!!!".green())
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }
}

fn is_git_directory() -> bool {
    return std::process::Command::new("git").arg("rev-parse").output().is_ok();
}

fn is_git_installed() -> bool {
    return std::process::Command::new("git").arg("--version").output().is_ok();
}
