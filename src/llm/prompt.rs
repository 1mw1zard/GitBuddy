pub const _PROMPT: &str = r###"Generate an appropriate conventional commit message based on the output of the git diff --cached command.
There MUST be only one type and description line.
  Use this template:
    <type>[optional scope]: <description>

    [optional body]

    [optional footer(s)]

Response must be only commit message, example:
    feat: allow provided config object to extend other configs

    BREAKING CHANGE: `extends` key in config file is now used for extending other config files
"###;

pub const PROMPT: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
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
const PROMPT__: &str = r###"You will receive a git diff. Write a commit message as if you are a senior software engineering.
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
const PROMPT_: &str = "Generate a concise commit message based on \
            the following git difference content. The generated message is plain text,\
             does not contain identifiers such as markdown \"`\", \
             and the generated content does not exceed 100 tokens. \
             Depending on the nature of the change, it starts with one of the following prefixes:\
              'build' (build system), 'chore' (chores), 'ci' (continuous integration), \
              'docs' (documentation), 'feat' (new feature), 'fix' (fix), 'perf' (performance),\
               'refactor' (refactoring), 'style' (style), 'test' (test):";