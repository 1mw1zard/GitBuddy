use std::process::Command;

pub fn git_stage_filenames() -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--no-ext-diff", "--diff-algorithm=minimal", "--name-only"]).output().unwrap();

    if !output.status.success() {
        return vec![];
    }

    return String::from_utf8(output.stdout).unwrap()
        .split('\n')
        .map_while(|s| if s.is_empty() { None } else { Some(s.to_string()) })
        .collect::<Vec<_>>();
}

pub fn git_stage_diff() -> String {
    let exclude_path: Vec<String> = ignore_filenames()
        .iter()
        .map(|path| format!(":(exclude){}", path))
        .collect();

    let commnad = Command::new("git")
        .args(["diff", "--cached", "--no-ext-diff", "--diff-algorithm=minimal"]);
    for path in exclude_path {
        commnad.arg(path);
    }

    let output = commnad.output().unwrap();

    if !output.status.success() {
        return "".to_string();
    }

    String::from_utf8(output.stdout).unwrap()
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
        "*.lock",
    ]
}


pub fn git_commit(message: &str) {
    let output = std::process::Command::new("git")
        .args(["commit", "-m", message]).output().unwrap();

    if output.status.success() {
        println!("commit success")
    } else {
        println!("commit failed")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_git_stage_filename() {
        let filenames = git_stage_filenames();

        println!("filenames: {:?}", filenames);
        assert!(!filenames.iter().any(|s| s.is_empty()));
    }

    #[test]
    fn test_git_stage_diff() {
        let diff = git_stage_diff();

        println!("diff: {:?}", diff);
        assert!(!diff.is_empty());
    }
}