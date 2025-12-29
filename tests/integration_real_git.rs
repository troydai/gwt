use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use std::path::Path;

fn run_git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {:?} failed", args);
}

#[test]
fn switch_real_worktree_success() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();

    // init repo and set user
    run_git(&repo_dir, &["init", "-b", "main"]);
    run_git(&repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(&repo_dir, &["config", "user.name", "Test"]);

    // initial commit
    let file_path = repo_dir.join("README.md");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "hello").unwrap();
    run_git(&repo_dir, &["add", "README.md"]);
    run_git(&repo_dir, &["commit", "-m", "initial"]);

    // create branch and commit
    run_git(&repo_dir, &["checkout", "-b", "feature-branch"]);
    let mut f2 = File::create(&repo_dir.join("feat.txt")).unwrap();
    writeln!(f2, "feat").unwrap();
    run_git(&repo_dir, &["add", "feat.txt"]);
    run_git(&repo_dir, &["commit", "-m", "feature commit"]);

    // create worktree for feature branch
    let wt_dir = tmp.path().join("wt_feature");
    run_git(&repo_dir, &["worktree", "add", wt_dir.to_str().unwrap(), "feature-branch"]);

    // run gwtree switch from repo dir - should print the worktree path
    let mut cmd = Command::cargo_bin("gwtree").unwrap();
    cmd.current_dir(&repo_dir)
        .arg("switch")
        .arg("feature-branch")
        .assert()
        .success()
        .stdout(predicate::str::contains(wt_dir.to_str().unwrap()));
}

#[test]
fn switch_real_worktree_not_found() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().join("repo2");
    fs::create_dir_all(&repo_dir).unwrap();

    // init repo and set user
    run_git(&repo_dir, &["init", "-b", "main"]);
    run_git(&repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(&repo_dir, &["config", "user.name", "Test"]);

    // initial commit
    let file_path = repo_dir.join("README.md");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "hello").unwrap();
    run_git(&repo_dir, &["add", "README.md"]);
    run_git(&repo_dir, &["commit", "-m", "initial"]);

    let mut cmd = Command::cargo_bin("gwtree").unwrap();
    cmd.current_dir(&repo_dir)
        .arg("switch")
        .arg("no-such-branch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Worktree for branch no-such-branch doesn't exist."));
}

#[test]
fn init_real_prints_shell_function() {
    let mut cmd = Command::cargo_bin("gwtree").unwrap();
    cmd.arg("init").arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("gwt() {"));
}
