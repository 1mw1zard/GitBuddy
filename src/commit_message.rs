use anyhow::{anyhow, Result};

const INVALID_SUBJECT_ERROR: &str = "LLM did not return a valid conventional commit subject.";

pub fn extract_commit_subject(raw: &str) -> Result<String> {
    if let Some(subject) = extract_strict(raw) {
        return Ok(subject);
    }

    if let Some(subject) = extract_relaxed(raw) {
        return Ok(subject);
    }

    Err(anyhow!(INVALID_SUBJECT_ERROR))
}

pub fn extract_commit_subject_from_json_or_text(raw: &str) -> Result<String> {
    if let Some(subject) = extract_json_subject(raw) {
        return validate_commit_subject(&subject);
    }

    extract_commit_subject(raw)
}

fn extract_json_subject(raw: &str) -> Option<String> {
    let json = clean_json_response(raw);
    let value: serde_json::Value = serde_json::from_str(json).ok()?;

    value.get("subject")?.as_str().map(|subject| subject.to_string())
}

fn clean_json_response(raw: &str) -> &str {
    let raw = raw.trim();
    let raw = raw.strip_prefix("```json").unwrap_or(raw);
    let raw = raw.strip_prefix("```text").unwrap_or(raw);
    let raw = raw.strip_prefix("```").unwrap_or(raw);
    let raw = raw.strip_suffix("```").unwrap_or(raw);

    raw.trim()
}

fn validate_commit_subject(subject: &str) -> Result<String> {
    let subject = clean_line(subject);
    if is_conventional_commit_subject(subject) {
        Ok(subject.to_string())
    } else {
        Err(anyhow!(INVALID_SUBJECT_ERROR))
    }
}

fn extract_strict(raw: &str) -> Option<String> {
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

    candidates.pop()
}

fn extract_relaxed(raw: &str) -> Option<String> {
    let mut candidates = Vec::new();
    let mut in_thinking_block = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("<think>") {
            in_thinking_block = true;
            continue;
        }

        if trimmed.eq_ignore_ascii_case("</think>") {
            in_thinking_block = false;
            continue;
        }

        if in_thinking_block {
            continue;
        }

        if let Some(candidate) = extract_candidate_from_line(trimmed) {
            candidates.push(candidate);
        }
    }

    candidates.pop()
}

fn extract_candidate_from_line(line: &str) -> Option<String> {
    let mut last: Option<String> = None;

    for (idx, _) in line.match_indices(": ") {
        if let Some(candidate) = build_candidate(&line[..idx], &line[idx + 2..]) {
            last = Some(candidate);
        }
    }

    last
}

fn build_candidate(prefix: &str, subject: &str) -> Option<String> {
    let prefix = candidate_prefix(prefix)?;

    let breaking = prefix.ends_with('!');
    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);

    let (commit_type, scope) = if let Some((commit_type, scope)) = prefix.split_once('(') {
        if !scope.ends_with(')') {
            return None;
        }

        let scope = &scope[..scope.len() - 1];
        if scope.is_empty() {
            return None;
        }

        (commit_type, Some(scope))
    } else {
        (prefix, None)
    };

    if !matches!(
        commit_type,
        "feat" | "fix" | "docs" | "style" | "refactor" | "perf" | "test" | "chore" | "ci" | "build"
    ) {
        return None;
    }

    let subject = clean_relaxed_subject(subject);
    if subject.is_empty() {
        return None;
    }

    let bang = if breaking { "!" } else { "" };
    match scope {
        Some(scope) => Some(format!("{}({}){}: {}", commit_type, scope, bang, subject)),
        None => Some(format!("{}{}: {}", commit_type, bang, subject)),
    }
}

fn candidate_prefix(prefix: &str) -> Option<&str> {
    let prefix = prefix.trim_end();
    let mut best = None;

    for commit_type in [
        "feat", "fix", "docs", "style", "refactor", "perf", "test", "chore", "ci", "build",
    ] {
        if let Some(idx) = prefix.rfind(commit_type) {
            let candidate = &prefix[idx..];
            if best.map(|(best_idx, _)| idx > best_idx).unwrap_or(true) {
                best = Some((idx, candidate));
            }
        }
    }

    best.map(|(_, candidate)| candidate.trim_end())
}

fn clean_relaxed_subject(subject: &str) -> &str {
    let subject = subject.trim();
    let end = subject
        .char_indices()
        .find_map(|(idx, ch)| matches!(ch, '"' | '\'' | '`').then_some(idx))
        .unwrap_or(subject.len());

    subject[..end].trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ':' | ')'))
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

    #[test]
    fn extracts_subject_embedded_in_analysis_sentence() {
        let raw = r#"We are asked to generate a conventional commit subject line. The type is likely "refactor" since it's cleaning up the UI. I'll go with "refactor(theme): remove box shadows from card containers and delete unused shadow tokens" or something like that."#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(
            subject,
            "refactor(theme): remove box shadows from card containers and delete unused shadow tokens"
        );
    }

    #[test]
    fn strict_match_takes_precedence_over_embedded_candidate() {
        let raw = r#"Embedded "refactor(theme): remove shadows" here.

refactor(ui): strip box shadows across widgets"#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(subject, "refactor(ui): strip box shadows across widgets");
    }

    #[test]
    fn extracts_last_embedded_candidate() {
        let raw = r#"Option A is "fix(api): handle empty response" and option B is "feat(ui): add loading spinner"."#;

        let subject = extract_commit_subject(raw).unwrap();

        assert_eq!(subject, "feat(ui): add loading spinner");
    }

    #[test]
    fn ignores_candidates_inside_thinking_block_in_relaxed_mode() {
        let raw = r#"<think>
Maybe "fix(api): patch auth" would work.
</think>

No final answer here, only thoughts."#;

        let err = extract_commit_subject(raw).unwrap_err();

        assert_eq!(
            err.to_string(),
            "LLM did not return a valid conventional commit subject."
        );
    }

    #[test]
    fn extracts_subject_from_json_response() {
        let raw = r#"{"subject":"fix(ai): handle invalid json response"}"#;

        let subject = extract_commit_subject_from_json_or_text(raw).unwrap();

        assert_eq!(subject, "fix(ai): handle invalid json response");
    }

    #[test]
    fn extracts_subject_from_json_code_block() {
        let raw = r#"```json
{"subject":"feat(ai): request structured commit responses"}
```"#;

        let subject = extract_commit_subject_from_json_or_text(raw).unwrap();

        assert_eq!(subject, "feat(ai): request structured commit responses");
    }

    #[test]
    fn rejects_json_subject_that_is_not_conventional() {
        let err = extract_commit_subject_from_json_or_text(r#"{"subject":"update response handling"}"#).unwrap_err();

        assert_eq!(
            err.to_string(),
            "LLM did not return a valid conventional commit subject."
        );
    }

    #[test]
    fn falls_back_to_text_subject_when_json_parse_fails() {
        let subject = extract_commit_subject_from_json_or_text("fix(ai): keep text fallback").unwrap();

        assert_eq!(subject, "fix(ai): keep text fallback");
    }
}
