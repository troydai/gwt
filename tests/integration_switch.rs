use assert_cmd::prelude::*;
use predicates::prelude::*;
use sha1::{Digest, Sha1};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

fn setup_home_and_config(root: &TempDir) -> (PathBuf, PathBuf) {
    let home = root.path().join("home");
    let config_dir = home.join(".gwt");
    fs::create_dir_all(&config_dir).unwrap();

    let worktree_root = root.path().join("worktrees");
    fs::create_dir_all(&worktree_root).unwrap();

    fs::write(
        config_dir.join("config.toml"),
        format!(
            r#"worktree_root = "{}"
"#,
            worktree_root.display()
        ),
    )
    .unwrap();

    (home, worktree_root)
}

fn setup_mock_git(root: &TempDir) -> (PathBuf, PathBuf) {
    let mock_git = root.path().join("mock-git");
    let log = root.path().join("git.log");

    let script = r#"#!/usr/bin/env bash
set -euo pipefail

echo "$@" >> "${MOCK_GIT_LOG}"

if [[ "${1:-}" == "worktree" && "${2:-}" == "list" && "${3:-}" == "--porcelain" ]]; then
  printf "%s" "${MOCK_GIT_WORKTREE_LIST:-}"
  exit 0
fi

if [[ "${1:-}" == "rev-parse" && "${2:-}" == "--show-toplevel" ]]; then
  echo "${MOCK_GIT_TOPLEVEL:?MOCK_GIT_TOPLEVEL is required}"
  exit 0
fi

if [[ "${1:-}" == "show-ref" && "${2:-}" == "--verify" && "${3:-}" == "--quiet" ]]; then
  ref="${4:-}"
  branch="${ref#refs/heads/}"
  IFS=',' read -ra branches <<< "${MOCK_GIT_BRANCHES:-}"
  for b in "${branches[@]}"; do
    if [[ "$b" == "$branch" ]]; then
      exit 0
    fi
  done
  exit 1
fi

if [[ "${1:-}" == "worktree" && "${2:-}" == "add" ]]; then
  exit "${MOCK_GIT_WORKTREE_ADD_EXIT:-0}"
fi

echo "Unhandled git args: $*" >&2
exit 2
"#;

    write_executable(&mock_git, script);
    fs::write(&log, "").unwrap();

    (mock_git, log)
}

#[test]
fn switch_prints_existing_worktree_path_when_present() {
    let root = TempDir::new().unwrap();
    let (home, _worktree_root) = setup_home_and_config(&root);
    let (mock_git, log) = setup_mock_git(&root);

    let worktree_list = r#"worktree /existing/feature
HEAD abc123
branch refs/heads/feature-branch
"#;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("gwtree"));
    cmd.env("HOME", &home)
        .env("GWT_GIT", &mock_git)
        .env("MOCK_GIT_LOG", &log)
        .env("MOCK_GIT_WORKTREE_LIST", worktree_list)
        .arg("switch")
        .arg("feature-branch");

    cmd.assert()
        .success()
        .stdout(predicate::eq("/existing/feature\n"));

    let log_contents = fs::read_to_string(&log).unwrap();
    assert!(!log_contents.contains("worktree add"));
    assert!(!log_contents.contains("rev-parse --show-toplevel"));
    assert!(!log_contents.contains("show-ref --verify --quiet"));
}

#[test]
fn switch_creates_worktree_when_missing() {
    let root = TempDir::new().unwrap();
    let (home, worktree_root) = setup_home_and_config(&root);
    let (mock_git, log) = setup_mock_git(&root);

    // No existing worktree for the branch.
    let worktree_list = r#"worktree /existing/main
HEAD abc123
branch refs/heads/main
"#;

    let repo_toplevel = PathBuf::from("/tmp/repo");
    let repo_name = "repo";
    let branch = "new-branch";
    let digest = Sha1::digest(format!("{repo_name}|{branch}"));
    let hash = format!("{digest:x}")[0..8].to_string();
    let expected = worktree_root.join(repo_name).join(hash);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("gwtree"));
    cmd.env("HOME", &home)
        .env("GWT_GIT", &mock_git)
        .env("MOCK_GIT_LOG", &log)
        .env("MOCK_GIT_WORKTREE_LIST", worktree_list)
        .env("MOCK_GIT_TOPLEVEL", repo_toplevel.display().to_string())
        .env("MOCK_GIT_BRANCHES", branch)
        .arg("switch")
        .arg(branch);

    cmd.assert()
        .success()
        .stdout(predicate::eq(format!("{}\n", expected.display())));

    let log_contents = fs::read_to_string(&log).unwrap();
    assert!(log_contents.contains("worktree add"));
    assert!(log_contents.contains(&format!("worktree add {} {}", expected.display(), branch)));
}

#[test]
fn switch_errors_when_branch_does_not_exist() {
    let root = TempDir::new().unwrap();
    let (home, _worktree_root) = setup_home_and_config(&root);
    let (mock_git, log) = setup_mock_git(&root);

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("gwtree"));
    cmd.env("HOME", &home)
        .env("GWT_GIT", &mock_git)
        .env("MOCK_GIT_LOG", &log)
        .env("MOCK_GIT_WORKTREE_LIST", "")
        .env("MOCK_GIT_TOPLEVEL", "/tmp/repo")
        .env("MOCK_GIT_BRANCHES", "main")
        .arg("switch")
        .arg("missing-branch");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Branch missing-branch doesn't exist.",
    ));
}
