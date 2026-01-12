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

### Step 1: Pre-merge Checks

Verify the current state and PR readiness:
```bash
# Check current branch and worktree
gwt current

# Check PR status - ensure it's ready to merge
gh pr view
```

**STOP and ask for instructions if:**
- You are in the home repository (not a feature worktree)
- There are uncommitted changes (`git status` shows modifications)
- The PR is not ready to merge (failed checks, missing approvals, etc.)

### Step 2: Merge the PR

Merge with squash (do not delete branches yet):
```bash
gh pr merge -s
```

Once the merge succeeds, proceed with cleanup.

### Step 3: Remove Worktree and Switch Home

Remove the current worktree and switch back to home:
```bash
gwt rm --this -y
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

## Important Notes

- Always verify the PR is ready to merge before starting
- Never force-delete branches with unmerged changes
- If any step fails unexpectedly, stop and ask for user guidance
- This workflow assumes a squash merge strategy; adjust if using merge commits or rebase
