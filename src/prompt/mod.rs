use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Prompt {
    P1,
    P2,
    P3,
    P4,
    P5,
}

impl Display for Prompt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Prompt::P1 => write!(f, "p1"),
            Prompt::P2 => write!(f, "p2"),
            Prompt::P3 => write!(f, "p3"),
            Prompt::P4 => write!(f, "p4"),
            Prompt::P5 => write!(f, "p5"),
        }
    }
}

impl Prompt {
    pub(crate) fn value(self) -> &'static str {
        match self {
            Prompt::P1 => PROMPT,
            Prompt::P2 => PROMPT2,
            Prompt::P3 => PROMPT3,
            Prompt::P4 => PROMPT4,
            Prompt::P5 => PROMPT5,
        }
    }
}

pub const PROMPT: &str = r###"You are an elite software architect writing commit messages. Analyze the provided code changes deeply and generate a commit message that demonstrates technical insight and clarity.

Format:
<type>(<scope>): <imperative mood, strong verb + specific intent>

[Optional body — highly encouraged for non-trivial changes]
- Motivation: WHY this change was necessary (not just WHAT changed)
- Technical decision: What approach was chosen and what trade-offs were made
- Impact: Who or what is affected by this change

Requirements:
1. The subject line MUST:
   - Start with a powerful, precise verb (e.g., restructure, resolve, eliminate, introduce, enforce, migrate, consolidate, decouple)
   - Be under 72 characters
   - Convey intent and outcome, not just action. Avoid vague words like "update", "modify", "change", "fix" without specificity
   - Example: instead of "make base_url optional", write "simplify vendor config by inferring base_url defaults"

2. The body MUST:
   - Explain the reasoning behind the change
   - Mention consequences, side effects, or follow-up work if any
   - NOT simply repeat what the diff already shows

3. Allowed types: feat, fix, docs, style, refactor, perf, test, chore, ci, build

4. Output ONLY the commit message, no explanations, no markdown code blocks, no quotes.
"###;
pub const PROMPT2: &str = r###"Generate an appropriate conventional commit message based on the output of the git diff --cached command.
There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <subject>

    [optional description]

    [optional footer(s)]

Response must be only commit message, example:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files
"###;
pub const PROMPT3: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
  Keep the commit messages brief, but informative. Use new lines to break apart long sentences.
  Type can be fix, feat, BREAKING CHANGE. Other types of commits are allowed, e.g. build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.

  There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <description>

    [optional body]

  Examples:

  Commit message with description and breaking change footer:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files

  Commit message with ! to draw attention to breaking change:
    feat!: send an email to the customer when a product is shipped

  Commit message with scope and ! to draw attention to breaking change:
    feat(api)!: send an email to the customer when a product is shipped

  Commit message with both ! and BREAKING CHANGE footer:
    chore!: drop support for Node 6

    BREAKING CHANGE: use JavaScript features not available in Node 6.

  Commit message with no body:
    docs: correct spelling of CHANGELOG

  Commit message with scope:
    feat(lang): add Polish language

  Commit message with multi-paragraph body and multiple footers:
    fix: prevent racing of requests

    Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.
  "###;
const PROMPT4: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
  Keep the commit messages brief, but informative. Use new lines to break apart long sentences.
  Type can be fix, feat, BREAKING CHANGE. Other types of commits are allowed, e.g. build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.

  There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <description>

    [optional body]

  Examples:

  Commit message with description and breaking change footer:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files

  Commit message with ! to draw attention to breaking change:
    feat!: send an email to the customer when a product is shipped

  Commit message with scope and ! to draw attention to breaking change:
    feat(api)!: send an email to the customer when a product is shipped

  Commit message with both ! and BREAKING CHANGE footer:
    chore!: drop support for Node 6

    BREAKING CHANGE: use JavaScript features not available in Node 6.

  Commit message with no body:
    docs: correct spelling of CHANGELOG

  Commit message with scope:
    feat(lang): add Polish language

  Commit message with multi-paragraph body and multiple footers:
    fix: prevent racing of requests

    Introduce a request id and a reference to latest request. Dismiss
    incoming responses other than from latest request.

    Remove timeouts which were used to mitigate the racing issue but are
    obsolete now.

  No think in response!"###;
const PROMPT5: &str = "Generate a concise commit message based on \
            the following git difference content. The generated message is plain text,\
             does not contain identifiers such as markdown \"`\", \
             and the generated content does not exceed 100 tokens. \
             Depending on the nature of the change, it starts with one of the following prefixes:\
              'build' (build system), 'chore' (chores), 'ci' (continuous integration), \
              'docs' (documentation), 'feat' (new feature), 'fix' (fix), 'perf' (performance),\
               'refactor' (refactoring), 'style' (style), 'test' (test):";
