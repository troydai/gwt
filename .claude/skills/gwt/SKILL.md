---
name: gwt
description: Manage git worktrees using gwt for branch-based workflow. Use when switching branches, creating worktrees, listing worktrees, or cleaning up after merging PRs.
---

# GWT Worktree Management Skill

You have access to `gwt`, a Git worktree management tool that simplifies working with multiple branches simultaneously. Use gwt instead of raw `git worktree` commands for a streamlined workflow.

## Available Commands

### Switch to a Branch Worktree
```bash
gwt sw <branch>           # Switch to existing branch (creates worktree if needed)
gwt sw -b <branch>        # Create new branch and switch to it
gwt sw -m                 # Switch to main/master branch
gwt sw <branch> --remote <remote>  # Switch to branch from specific remote
```

### List Worktrees
```bash
gwt ls                    # List all worktrees (truncated branch names)
gwt ls --full             # List with full branch names
```

### Remove Worktrees
```bash
gwt rm <branch>           # Remove worktree for branch (prompts for confirmation)
gwt rm <branch> -y        # Remove without confirmation
gwt rm <branch> -b        # Remove worktree and delete the branch
gwt rm <branch> -B        # Remove worktree and force-delete branch
gwt rm --this             # Remove current worktree (switches to home first)
```

### Navigation
```bash
gwt home                  # Return to the main repository worktree
gwt current               # Show current branch and worktree path (alias: gwt c)
```

## Workflow Patterns

### Starting Work on a Feature
1. Use `gwt sw -b feature/name` to create a new branch and worktree
2. Work on the feature in the isolated worktree
3. The original repository remains on its branch

### Switching Between Tasks
1. Use `gwt sw other-branch` to switch to another worktree
2. No need to stash changes - each worktree is independent
3. Use `gwt ls` to see all available worktrees

### Cleaning Up After PR Merge
1. Use `gwt home` to return to main repository
2. Use `gwt rm feature/name -b` to remove worktree and delete merged branch
3. Or use `gwt rm --this -b` if still in the feature worktree

### Checking Current Location
Use `gwt current` to see which branch and worktree you're in.

## Key Benefits

- **Automatic directory switching**: gwt changes your shell's working directory automatically
- **Centralized storage**: Worktrees are stored in `~/.gwt_store`, keeping repos clean
- **Just-in-time creation**: Worktrees are created automatically when switching to new branches
- **Smart remote lookup**: Automatically finds and tracks remote branches

## When to Use gwt

- Switching between branches for code review or testing
- Working on multiple features in parallel
- Isolating experimental changes without affecting main work
- Quick context switching without stashing changes
