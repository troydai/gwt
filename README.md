# GWT: Git Worktree Management

[![CI](https://github.com/troydai/gwt/actions/workflows/ci.yml/badge.svg)](https://github.com/troydai/gwt/actions/workflows/ci.yml)

GWT is a CLI tool designed to streamline Git worktree management. It provides a centralized, organized way to handle multiple concurrent work environments within a single repository, reducing the operational overhead of the native `git worktree` command set.

## Why GWT?

As AI-driven development and complex feature branching become more common, developers often need to work across multiple branches simultaneously. While Git worktrees are a powerful alternative to full repository clones, they can be cumbersome to manage:

- **Clutter**: Creating worktrees inside your main repository can clutter your file system.
- **Navigation**: Manually navigating between worktree directories is slow.
- **Lifecycle**: Cleaning up or tracking worktrees spread across different paths is difficult.

GWT solves this by:
- **Centralizing Storage**: All worktrees are stored in a dedicated root directory (e.g., `~/.gwt_store`), keeping your repositories clean.
- **Automatic Navigation**: A shell wrapper allows you to switch branches and directories in a single command.
- **Deterministic Paths**: Worktree locations are computed using a stable hash of the repository path and branch name.

## Installation

### 1. Build and Install the Binary

The core logic resides in the `gwtree` binary. You can build and install it using Cargo:

```bash
make install
```

This installs `gwtree` to your Cargo bin directory (typically `~/.cargo/bin`).

### 2. Shell Integration

Because a subprocess cannot change the parent shell's working directory, GWT uses a shell function named `gwt` as a wrapper.

Add the following to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish`):

**Bash / Zsh:**
```bash
eval "$(gwtree init bash)"
```

**Fish:**
```fish
eval (gwtree init fish)
```

## Quick Start

1. **Initialize**: Run `gwt` for the first time to set up your configuration.
   ```bash
   gwt config view
   ```
   If it's your first time, it will prompt you to create a config file at `~/.gwt/config.toml`.

2. **Switch to a Branch**:
   ```bash
   gwt sw feature-x
   ```
   If a worktree for `feature-x` already exists, GWT will jump to it. If not, it will create one in your worktree store and then move you there.

## Try it in the Playground

If you want to try GWT without affecting your local system or installing dependencies, you can use the provided Docker-based playground:

```bash
make playground
```

This will build a container with GWT pre-installed and drop you into a shell where you can experiment with worktree management safely.

## User Manual

### Commands

#### `gwt sw <branch> [-b|--create-branch] [-m|--main]` (Switch)

The `sw` (switch) command is the heart of GWT. It combines worktree discovery, creation, and directory navigation into a single seamless operation.

- **Automatic Directory Switching**: Unlike the native `git worktree` command, `gwt sw` automatically changes your shell's current working directory to the target worktree. This eliminates the need to manually `cd` after switching branches.
- **Just-in-Time Worktree Creation**: If a worktree for the specified branch doesn't exist yet, GWT handles the heavy lifting:
    - It creates a new worktree in your centralized `worktree_root` (default `~/.gwt_store`).
    - It uses a deterministic hashing algorithm to ensure the worktree path is stable and unique to that repository/branch combination.
    - Once created, it immediately moves your shell into that new directory.
- **New Branch Creation**: With the `-b` or `--create-branch` flag, GWT will create the branch for you if it doesn't already exist.
- **Main Branch Shortcut**: Use the `-m` or `--main` flag to quickly switch to the primary branch without specifying its name. GWT will automatically detect and use `main` if it exists, falling back to `master` if only the latter is present.
- **Safe Transitions**: GWT checks if you are already on the target branch or if the branch exists before making any changes, preventing accidental state issues. Informational messages are printed to `stderr` to keep `stdout` clean for path-based navigation.

**Example:**
```bash
# Switch to a feature branch, creating it if necessary
$ gwt sw -b feature-api-v2
Branch 'feature-api-v2' created.
Created directory: /Users/me/.gwt_store
Created worktree for branch 'feature-api-v2' at '/Users/me/.gwt_store/a1b2c3d4e5f6g7h8'

# You are now automatically navigated to the worktree directory
$ pwd
/Users/me/.gwt_store/a1b2c3d4e5f6g7h8

# Quickly switch to the main branch (main or master)
$ gwt sw -m
# Automatically switches to 'main' if it exists, or 'master' as fallback
```

---

#### `gwt rm <branch> [-b|--delete-branch] [-B|--force-delete-branch]` (Remove)

The `rm` (remove) command simplifies worktree removal by allowing you to specify branches instead of directory paths.

- **Branch-Centric Workflow**: Specify the branch name to identify and remove its associated worktree, eliminating the need to remember or look up worktree directory paths.
- **Automatic Directory Switching**: If you're currently working inside the worktree being removed, GWT automatically switches you to the main worktree before deletion, preventing errors from deleting your current directory.
- **Interactive Confirmation**: Prompts for confirmation before removing the worktree to prevent accidental deletions.
- **Optional Branch Deletion**: Use `-b` or `--delete-branch` to delete the branch after removing the worktree. Use `-B` or `--force-delete-branch` for force deletion (equivalent to `git branch -D`).

**Example:**
```bash
# Remove a worktree for a feature branch
$ gwt rm feature-api-v2
Remove worktree at '/Users/me/.gwt_store/a1b2c3d4e5f6g7h8' for branch 'feature-api-v2'? [y/N] y
Worktree for branch 'feature-api-v2' removed.

# Remove a worktree and delete the branch
$ gwt rm feature-api-v2 -b
Remove worktree at '/Users/me/.gwt_store/a1b2c3d4e5f6g7h8' for branch 'feature-api-v2'? [y/N] y
Worktree for branch 'feature-api-v2' removed.
Branch 'feature-api-v2' deleted.

# Force delete a branch with unmerged changes
$ gwt rm feature-api-v2 -B
```

---

#### `gwt current` (alias: `gwt c`)

Displays information about the current Git worktree and branch. This is useful for quickly checking which branch you're on and which worktree directory you're working in.

The command outputs:
- The current branch name (or `(detached)` if HEAD is detached) - highlighted in green (yellow for detached)
- The absolute path to the current worktree root directory - highlighted in cyan

**Example:**
```bash
$ gwt current
Branch main @ Worktree /home/user/my-repo

$ gwt c  # Using the alias
Branch feature/my-feature @ Worktree /home/user/.gwt_store/a1b2c3d4e5f6g7h8
```

---

#### `gwt config view`
Displays the location and current contents of your configuration file. This is useful for verifying where your worktrees are being stored.

---

#### `gwt config setup`
Interactively sets up or resets your configuration. It will prompt you for the `worktree_root` directory.

---

#### `gwt ls [--full]`
Lists all worktrees managed by Git. The output is color-coded and aligned for readability:
- **Marker**: An asterisk (`*`) indicates the current active worktree.
- **Hash**: Shortened commit hash (yellow).
- **Branch**: Branch name (green). Long names are truncated to 32 characters by default.
- **Path**: Absolute path to the worktree (cyan).

Use the `--full` flag to prevent truncation of long branch names.

**Example:**
```bash
$ gwt ls
* 3fdfaf9 main     /home/user/repo
  86ee136 feat/api /home/user/.gwt_store/a1b2c3d4
```

---

#### `gwtree init <shell>`
Generates the shell integration code required for the `gwt` wrapper to function. This is typically used once during initial setup in your `.bashrc`, `.zshrc`, or `config.fish`. Supported shells: `bash`, `zsh`, `fish`.

## Configuration

GWT uses a TOML configuration file located at `~/.gwt/config.toml`.

| Option | Description | Default |
|--------|-------------|---------|
| `worktree_root` | Absolute path where worktrees are stored. | `~/.gwt_store` |

Example `config.toml`:
```toml
worktree_root = "/Users/username/.gwt_store"
```

## Development

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- [pre-commit](https://pre-commit.com/)

### Setup
```bash
make setup-pre-commit
```

### Running Tests
```bash
cargo test
```

### Manual Build
```bash
cargo build --release
```

## License
Distributed under the MIT License. See `LICENSE` for more information.