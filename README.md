# GWT: Git Worktree Management

## Overview

GWT is a CLI tool designed to simplify Git worktree management and reduce operational overhead. As AI-driven development becomes more common, the need for multiple concurrent work environments within a single repository has increased. Git worktrees provide a lightweight alternative to full repository clones, but the native command set can be cumbersome, requiring manual directory navigation and lacking centralized organization.

GWT streamlines this workflow by providing a unified interface for creating, navigating, and managing worktrees, optimizing the experience for both developers and AI agents.

## Status

This project is currently under active development.

## Installation

The tool consists of two components:
- `gwtree` - the compiled binary
- `gwt` - a shell function that wraps `gwtree`

The shell function is required because changing the current working directory must be done by the shell itself (a subprocess cannot change the parent shell's directory).

### Build and Install

```bash
make install
```

This builds the release binary and installs `gwtree` to your Cargo bin directory (typically `~/.cargo/bin`).

### Shell Integration

Add the following to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`):

```bash
# gwt - Git Worktree Management
gwt() {
    gwtree "$@"
}
```

## User Manual

### Quickstart ✅

Install the binary and enable shell integration:

1. Install the binary:

```bash
make install
```

2. Add shell integration by evaluating the init output in your shell startup file (e.g., `~/.bashrc`, `~/.zshrc`):

```bash
# For bash or zsh
eval "$(gwtree init bash)"

# For fish
eval (gwtree init fish)
```

Reload your shell (or open a new terminal) and you're ready to go.

---

### Commands

- `gwtree switch <branch>` — If a worktree already exists for `<branch>`, prints the worktree path to stdout and exits 0. If no worktree exists for that branch, creates a new worktree and prints its path to stdout (exit 0). If the branch does not exist, prints an error message to stderr and exits 1.

Example (existing worktree):

```bash
$ gwt switch feature-branch
/path/to/worktrees/feature-branch
$ echo $?
0
```

Example (missing worktree: created):

```bash
$ gwt switch feature-branch
$HOME/.gwt_store/<repo>/<hash>
$ echo $?
0
```

- `gwtree init <shell>` — Print the shell code that provides the `gwt` wrapper function for the specified shell (`bash`, `zsh`, or `fish`). The wrapper calls the `gwtree` binary and performs `cd` on success for `switch` commands.

Example:

```bash
$ gwtree init bash
# prints shell function which you can `eval` in your shell
```

- `gwtree config view` — Display the configuration file path and contents. This command shows where your config file is located and its current contents without triggering an interactive setup process.

Example:

```bash
$ gwt config view
Config file path: /home/user/.gwt/config.toml

Config file contents:
worktree_root = "/home/user/.gwt_store"
```

---

### Shell Integration Details

Why a shell wrapper?

A process cannot change the working directory of its parent shell. To get the 'cd' behavior shown in usage, the `gwt` wrapper must run in the current shell. `gwtree init <shell>` prints a shell function that the user should `eval` in their shell startup files.

Bash / Zsh example (what `gwtree init bash` prints):

```bash
gwt() {
  if [ "$1" = "switch" ]; then
    local result
    result=$(command gwtree switch "${@:2}")
    local exit_code=$?
    if [ $exit_code -eq 0 ]; then
      cd "$result" || return 1
    else
      echo "$result" >&2
      return $exit_code
    fi
  else
    command gwtree "$@"
  fi
}
```

Fish example (what `gwtree init fish` prints):

```fish
function gwt
  if test "$argv[1]" = "switch"
    set result (command gwtree switch (echo $argv | sed 's/^switch //'))
    set exit_code $status
    if test $exit_code -eq 0
      cd "$result"; or return 1
    else
      echo $result >&2
      return $exit_code
    end
  else
    command gwtree $argv
  end
end
```

---

### Configuration

`gwt` uses a central configuration file to manage settings.

- **Location**: `~/.gwt/config.toml`
- **Format**: TOML

#### First-time Setup

When you run `gwt` for the first time, it will prompt you to create a configuration file interactively.

1.  **Create Configuration File**: Confirm to create the file.
2.  **Worktree Root**: Specify the directory where you want to store your worktrees. The default is `~/.gwt_store`.

#### Configuration Options

Currently, the configuration supports the following option:

- `worktree_root`: The absolute path to the directory where all git worktrees will be stored.

**Example `config.toml`:**

```toml
worktree_root = "/Users/username/.gwt_store"
```

You can manually edit this file to change the worktree root location.

---

### Testing & Development Tips 🧪

- Run unit & integration tests:

```bash
cargo test
```

- Integration tests require `git` available on PATH and will create temporary repositories and worktrees; they run in temporary directories and do not affect your local repos.

- There is a testing hook to mock the `git` executable in integration tests by setting the `GWT_GIT` environment variable to point to a mock script. This is used in `tests/integration_switch.rs`.

- Use `cargo run -- <command>` to try the binary locally without installing.

---

If you'd like, we can add more examples (creating and removing worktrees, listing, etc.) as those commands are implemented.

## Planned Features

- **Centralized Configuration**: Define a root directory for all worktrees.
- **Automated Navigation**: Seamlessly switch between worktrees with automatic directory changing.
- **Rapid Lifecycle Management**: Efficient commands for creating and deleting worktrees.
- **AI Integration**: Streamlined workflow to initialize a worktree and invoke an AI agent in a single step.

## Development Setup

This project uses [pre-commit](https://pre-commit.com/) to ensure code quality.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [pre-commit](https://pre-commit.com/#install)

### Installation

1. Install the pre-commit framework:
   ```bash
   pip install pre-commit
   # or
   brew install pre-commit
   ```

2. Install the git hooks:
   ```bash
   make setup-pre-commit
   ```

   Alternatively, you can run:
   ```bash
   pre-commit install
   ```

### Manual Usage

You can run the checks manually at any time:
```bash
pre-commit run --all-files
```
