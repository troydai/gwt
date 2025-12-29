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

### Build

```bash
cargo build --release
```

This produces the `gwtree` binary in `target/release/`.

### Shell Integration

Add the following to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`):

```bash
# gwt - Git Worktree Management
gwt() {
    gwtree "$@"
}
```

## User Manual

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
