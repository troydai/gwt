---
name: merge-pr
description: Merge a GitHub PR and clean up the worktree. Use when merging a PR from a feature branch worktree that meets merging criteria.
---

# Merge PR Skill

This skill handles the complete workflow for merging a GitHub PR when working in a gwt-managed worktree. It coordinates the merge, worktree cleanup, and branch synchronization.

## When to Use

Use this skill when:
- The user asks to merge a GitHub PR
- You are currently in a feature branch worktree (not the main/home repository)
- The PR has been approved and meets merging criteria

## Why This Skill Exists

When working in a gwt worktree, the standard `gh pr merge -sd` command is insufficient because:
- The `-d` flag cannot delete the local branch while it's checked out in the current worktree
- The worktree directory cannot be removed while you're inside it
- Manual coordination is required to clean up properly

## Workflow Steps

### Step 1: Pre-merge Safety Checks

Before proceeding, verify:
```bash
# Check current branch and worktree
gwt current

# Verify you're not in the home/main worktree
# If gwt current shows you're on main/master in the home repo, STOP - this skill is not needed
```

**STOP and ask for instructions if:**
- You are in the home repository (not a feature worktree)
- There are uncommitted changes (`git status` shows modifications)
- The PR is not ready to merge (check with `gh pr view`)

### Step 2: Merge the PR

Merge the PR with squash, deleting only the remote branch:
```bash
# Merge with squash, delete remote branch only (not local)
gh pr merge --squash --delete-branch
```

Note: `--delete-branch` only deletes the remote branch. The local branch remains because it's checked out.

### Step 3: Clean Up Worktree and Local Branch

Remove the current worktree and its branch, automatically switching to home:
```bash
# Remove current worktree and delete the local branch
# This automatically switches to the home repository
gwt rm --this -b -y
```

The `-b` flag deletes the local branch after removing the worktree.
The `-y` flag skips confirmation since we've already merged.

### Step 4: Sync Main Branch

Pull the latest changes including the merged PR:
```bash
# Sync main branch and prune deleted remote branches
git pull -p
```

## Complete Example

```bash
# 1. Verify current state
gwt current
# Output: Branch feature/my-feature @ Worktree /Users/me/.gwt_store/abc123

# 2. Merge the PR (squash merge, delete remote branch)
gh pr merge --squash --delete-branch

# 3. Clean up worktree and local branch (switches to home automatically)
gwt rm --this -b -y

# 4. Sync main branch
git pull -p
```

## Error Handling

**If `gh pr merge` fails:**
- Check PR status with `gh pr view`
- Verify CI checks have passed
- Ensure PR has required approvals
- Do NOT proceed with cleanup steps

**If `gwt rm --this` fails:**
- Check for uncommitted changes
- Verify the worktree state with `gwt ls`
- Ask user for instructions before attempting manual cleanup

**If `git pull -p` fails:**
- Check network connectivity
- Verify remote is accessible
- This is non-critical; the merge is already complete

## Important Notes

- Always verify the PR is ready to merge before starting
- Never force-delete branches with unmerged changes
- If any step fails unexpectedly, stop and ask for user guidance
- This workflow assumes a squash merge strategy; adjust if using merge commits or rebase
