# Rust feature implementing

Implement given feature in Rust.

Input is a github issue with detailed instruction of requirement and implementation plan. Follow the plan.

Required qualities:
- Write idiomatic Rust code.
- Write concise, easy to read, and extensible code.
- Avoid clutter and complicated code.
- Do not add things that are not asked to. Do not over design.
- Implement unit test to provide sufficient code coverage.

Workflow
- Create a new feature branch from last main (or master) branch.
- Create a worktree from that branch and switch current directory to the worktree.
- Read the entire github issue.
- Understand the issue including requirements, design, and implementation plan.
- Read the relevant codebase to form context.
- Implement the feature by producing code. Using a test-driven approach, write unit test first.
- Iterate on the code using a cycle of testing -> coding -> linting.
- Update README.md and CHANGELOG.md.
- Create a pull request. The pull request's title and description are filled with the right amount of details.

Tools available:
- Use `gh` CLI to interact with github.
- Use `gwt` CLI to switch worktree. For example `gwt sw -b <branch>` will create a worktree for `<branch>` and switch to it.
- Use `git` CLI for other git operations.
- Use `cargo` for Rust.
