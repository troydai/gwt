# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Implemented `gwtree switch <branch>` command which prints the path of an existing worktree for the specified branch and exits with 0; prints an error and exits 1 if not found. 🔧
- Implemented `gwtree init <shell>` which emits shell-integration code for `bash`, `zsh`, and `fish` to allow `gwt` wrapper to `cd` into worktrees on success. 🔧
- Added `make install` command for convenient local installation (macOS/Linux). 📦

### Tests

- Added unit tests for parsing `git worktree list --porcelain` output and branch matching. ✅
- Added integration tests:
  - Mock-based tests that substitute `git` via `GWT_GIT` env var to assert CLI behavior. ✅
  - Real `git` repository tests that create branches and worktrees and assert `gwtree switch` behavior. ✅

### Internal

- Reduced the public crate surface and encapsulated the `Worktree` struct fields behind accessors. 🔒
- Moved unit tests inline to their corresponding source files for clearer locality. 🧪
- Added testing helpers and dev-dependencies (`assert_cmd`, `predicates`) for robust integration testing. 🧰


---

*For more details, see commit history.*
