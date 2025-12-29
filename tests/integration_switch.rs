use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn make_mock_git(output: &str, exit_code: i32) -> std::path::PathBuf {
    let dir = tempdir().unwrap();
    let git_path = dir.path().join("git");
    let mut f = File::create(&git_path).unwrap();
    writeln!(f, "#!/usr/bin/env bash").unwrap();
    // print to stdout then exit with provided code
    writeln!(f, "echo -n '{}'", output.replace('\'', "'\\''")).unwrap();
    writeln!(f, "exit {}", exit_code).unwrap();
    drop(f);
    // make executable
    let mut perms = std::fs::metadata(&git_path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        std::fs::set_permissions(&git_path, perms).unwrap();
    }
    git_path
}

#[test]
fn switch_success_prints_path_and_exit_0() {
    let output = "worktree /tmp/path/to/feature\nHEAD def456\nbranch refs/heads/feature-branch\n";
    let git_mock = make_mock_git(output, 0);

    let bin = std::env::var_os("CARGO_BIN_EXE_gwtree").expect("CARGO_BIN_EXE_gwtree not set");
    let mut cmd = Command::new(bin);
    cmd.env("GWT_GIT", git_mock)
        .arg("switch")
        .arg("feature-branch")
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp/path/to/feature"));
}

#[test]
fn switch_not_found_exits_1_and_prints_error() {
    let output = "worktree /tmp/path/to/main\nHEAD abc123\nbranch refs/heads/main\n";
    let git_mock = make_mock_git(output, 0);

    let bin = std::env::var_os("CARGO_BIN_EXE_gwtree").expect("CARGO_BIN_EXE_gwtree not set");
    let mut cmd = Command::new(bin);
    cmd.env("GWT_GIT", git_mock)
        .arg("switch")
        .arg("feature-branch")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Worktree for branch feature-branch doesn't exist.",
        ));
}

#[test]
fn git_error_propagates_and_exits_1() {
    let output = "fatal: repository not found";
    let git_mock = make_mock_git(output, 1);

    let bin = std::env::var_os("CARGO_BIN_EXE_gwtree").expect("CARGO_BIN_EXE_gwtree not set");
    let mut cmd = Command::new(bin);
    cmd.env("GWT_GIT", git_mock)
        .arg("switch")
        .arg("feature-branch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Git error"));
}

#[test]
fn init_prints_shell_function_and_exits_0() {
    let bin = std::env::var_os("CARGO_BIN_EXE_gwtree").expect("CARGO_BIN_EXE_gwtree not set");
    let mut cmd = Command::new(bin);
    cmd.arg("init")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("gwt() {"));
}
