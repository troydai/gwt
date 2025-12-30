# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Added `gwt config setup` command to allow users to interactively set up or reset the configuration.

### Internal

- Refactored configuration loading logic to separate interactive prompting from the main loading flow.
- Simplified `setup` function return type and improved modularity in `src/config/mod.rs`.

## [0.1.0] - 2025-12-30

### Added

- Added `-b` / `--create-branch` flag to `gwt sw` command to create a new branch if it doesn't exist.
- Improved playground environment by automatically creating a sample git repository (`test_repo`) on startup.
- `gwtree sw <branch>` now notifies the user via stderr when a new worktree is created.

### Fixed

- Redirected informational messages (e.g., "Branch created", "Configuration saved", "Created directory") to `stderr` to ensure `stdout` only contains the worktree path, keeping shell integration clean.

- `gwtree sw <branch>` now checks if the current branch matches the requested branch. If so, it prints a warning in yellow and exits with status 1 (to prevent shell cd).
- Implemented `gwtree sw <branch>` command which prints the path of an existing worktree for the specified branch and exits with 0; prints an error and exits 1 if not found.
- Updated `gwtree sw <branch>` to create a worktree when missing (under `<worktree_root>/<repo>/<hash>`), and only error when the branch does not exist.
- Implemented `gwtree init <shell>` which emits shell-integration code for `bash`, `zsh`, and `fish` to allow `gwt` wrapper to `cd` into worktrees on success.
- Added `make install` command for convenient local installation (macOS/Linux).
- Implemented `gwtree config view` command which displays the configuration file path and contents with colored output. This allows users to inspect their configuration without triggering an interactive setup process.

### Tests

- Added unit tests for parsing `git worktree list --porcelain` output and branch matching.
- Added integration tests:
  - Mock-based tests that substitute `git` via `GWT_GIT` env var to assert CLI behavior.
  - Real `git` repository tests that create branches and worktrees and assert `gwtree sw` behavior.

### Internal

- Reduced the public crate surface and encapsulated the `Worktree` struct fields behind accessors.
- Moved unit tests inline to their corresponding source files for clearer locality.
- Added testing helpers and dev-dependencies (`assert_cmd`, `predicates`) for robust integration testing.
- Refactored config command logic into its own module (`src/command/config/mod.rs`) for better code organization.
- Added comprehensive unit tests for the config command module (7 new tests).


---

*For more details, see commit history.*
