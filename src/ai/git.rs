use std::process::Command;

use anyhow::{anyhow, Result};

pub fn git_stage_filenames() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--no-ext-diff",
            "--diff-algorithm=minimal",
            "--name-only",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| anyhow!("Invalid UTF-8 in git output: {}", e))?;

    Ok(stdout
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

pub fn git_stage_diff() -> Result<String> {
    let exclude_path: Vec<String> = ignore_filenames()
        .iter()
        .map(|path| format!(":(exclude){}", path))
        .collect();

    let mut command = Command::new("git");
    command.args(["diff", "--cached", "--no-ext-diff", "--diff-algorithm=minimal"]);

    for path in exclude_path {
        command.arg(path);
    }

    let output = command.output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    String::from_utf8(output.stdout).map_err(|e| anyhow!("Invalid UTF-8 in git diff output: {}", e))
}

/// Return statistics for staged changes.
pub fn git_stage_stats() -> Result<String> {
    let output = Command::new("git").args(["diff", "--cached", "--stat"]).output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    String::from_utf8(output.stdout).map_err(|e| anyhow!("Invalid UTF-8 in git stat output: {}", e))
}

fn ignore_filenames() -> Vec<&'static str> {
    vec![
        /* Rust files */
        "Cargo.lock",
        /* Node.js files */
        "node_modules",
        "dist",
        "package-lock.json",
        "pnpm-lock.json",
    ]
}

/// Commits the changes to the repository.
pub fn git_commit(message: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    let output = Command::new("git").args(["commit", "-m", message]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("commit failed: {}", stderr.trim()))
    }
}

/// Pushes the changes to the remote repository.
pub fn git_push(dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    let output = Command::new("git").args(["push", "origin", "HEAD"]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("push failed: {}", stderr.trim()))
    }
}

/// Check whether the working tree has unstaged changes.
pub fn has_unstaged_changes() -> Result<bool> {
    let output = Command::new("git").args(["diff", "--quiet"]).output()?;
    Ok(!output.status.success())
}

/// Stage all working tree changes.
pub fn git_add_all() -> Result<()> {
    let output = Command::new("git").args(["add", "."]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("git add failed: {}", stderr.trim()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_git_stage_filename() {
        let filenames = git_stage_filenames().unwrap();

        println!("filenames: {:?}", filenames);
        assert!(!filenames.iter().any(|s| s.is_empty()));
    }

    #[test]
    fn test_git_stage_diff() {
        let diff = git_stage_diff().unwrap();

        println!("diff: {:?}", diff);
        // diff may be empty if no staged changes or all excluded, so we don't assert non-empty
    }

    #[test]
    fn test_git_stage_stats() {
        let stats = git_stage_stats().unwrap();
        println!("stats: {:?}", stats);
    }
}
