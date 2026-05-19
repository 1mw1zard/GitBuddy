use anyhow::{anyhow, Result};

pub fn extract_commit_subject(raw: &str) -> Result<String> {
    let mut candidates = Vec::new();
    let mut in_thinking_block = false;

    for line in raw.lines() {
        let line = clean_line(line);
        if line.eq_ignore_ascii_case("<think>") {
            in_thinking_block = true;
            continue;
        }

        if line.eq_ignore_ascii_case("</think>") {
            in_thinking_block = false;
            continue;
        }

        if in_thinking_block {
            continue;
        }

        if is_conventional_commit_subject(line) {
            candidates.push(line.to_string());
        }
    }

    candidates
        .pop()
        .ok_or_else(|| anyhow!("LLM did not return a valid conventional commit subject."))
}

fn clean_line(line: &str) -> &str {
    line.trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
}

fn is_conventional_commit_subject(line: &str) -> bool {
    let Some((prefix, subject)) = line.split_once(": ") else {
        return false;
    };

    if subject.trim().is_empty() {
        return false;
    }

    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);
    let commit_type = if let Some((commit_type, scope)) = prefix.split_once('(') {
        if !scope.ends_with(')') {
            return false;
        }

        let scope = &scope[..scope.len() - 1];
        if scope.is_empty() {
            return false;
        }

        commit_type
    } else {
        prefix
    };

    matches!(
        commit_type,
        "feat" | "fix" | "docs" | "style" | "refactor" | "perf" | "test" | "chore" | "ci" | "build"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_clean_subject() {
        let raw = "fix(api): handle empty response";

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(subject, "fix(api): handle empty response");
    }

    #[test]
    fn extracts_final_subject_from_analysis_output() {
        let raw = r#"Let me analyze the code changes:

1. AGENTS.md changes:
   - Replaced "Temporal" with "River" throughout the documentation

Let me craft a good message:

refactor(workflow): migrate from Temporal to River for PostgreSQL-based execution"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(
            subject,
            "refactor(workflow): migrate from Temporal to River for PostgreSQL-based execution"
        );
    }

    #[test]
    fn extracts_subject_after_thinking_block() {
        let raw = r#"<think>
I should compare possible commit messages.
</think>

docs(readme): clarify installation steps"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(subject, "docs(readme): clarify installation steps");
    }

    #[test]
    fn ignores_subject_inside_thinking_block() {
        let raw = r#"<think>
fix(api): use the first idea from private reasoning
</think>"#;

        let err = extract_commit_subject(raw).unwrap_err();

        assert_eq!(
            err.to_string(),
            "LLM did not return a valid conventional commit subject."
        );
    }

    #[test]
    fn extracts_subject_from_markdown_code_block() {
        let raw = r#"```text
build(dev): add telepresence targets for local service interception
```"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(
            subject,
            "build(dev): add telepresence targets for local service interception"
        );
    }

    #[test]
    fn returns_last_valid_subject() {
        let raw = r#"docs(agents): document River-based workflow execution

refactor(workflow): migrate from Temporal to River for PostgreSQL-based execution"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(
            subject,
            "refactor(workflow): migrate from Temporal to River for PostgreSQL-based execution"
        );
    }

    #[test]
    fn rejects_unknown_type() {
        let err = extract_commit_subject("release: bump version").unwrap_err();

        assert_eq!(
            err.to_string(),
            "LLM did not return a valid conventional commit subject."
        );
    }

    #[test]
    fn rejects_output_without_valid_subject() {
        let err = extract_commit_subject("Let me analyze the diff first.").unwrap_err();

        assert_eq!(
            err.to_string(),
            "LLM did not return a valid conventional commit subject."
        );
    }

    #[test]
    fn accepts_breaking_change_markers() {
        let raw = r#"feat!: remove legacy config format
feat(api)!: require explicit base_url"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(subject, "feat(api)!: require explicit base_url");
    }
}
