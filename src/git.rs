use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::{DateTime, Utc};

pub const REPO_PATH: &str = "repo";
pub const DEFAULT_BRANCH_NAME: &str = "mojmap";

fn repo_path() -> PathBuf {
    let repo_path = Path::new(REPO_PATH);
    fs::create_dir_all(repo_path)
        .unwrap_or_else(|_| panic!("repo at {repo_path:?} couldn't be created"));
    repo_path
        .canonicalize()
        .unwrap_or_else(|_| panic!("repo path {repo_path:?} is invalid"))
}

pub fn ensure_repo_created() {
    Command::new("git")
        .args(["init", "--initial-branch", DEFAULT_BRANCH_NAME])
        .current_dir(repo_path())
        .output()
        .unwrap();
}

pub fn push() {
    Command::new("git")
        .args(["push", "--all"])
        .current_dir(repo_path())
        .output()
        .unwrap();
}

pub fn checkout_branch(name: &str) {
    Command::new("git")
        .args(["checkout", "--orphan", name])
        .current_dir(repo_path())
        // .stdout(io::stdout())
        // .stderr(io::stderr())
        .output()
        .unwrap();
}
pub fn commit(message: &str, author_name: &str, author_email: &str, timestamp: DateTime<Utc>) {
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "--message", message])
        .current_dir(repo_path())
        .env("GIT_AUTHOR_DATE", &timestamp.to_rfc3339())
        .env("GIT_COMMITTER_DATE", &timestamp.to_rfc3339())
        .env("GIT_AUTHOR_NAME", author_name)
        .env("GIT_AUTHOR_EMAIL", author_email)
        .env("GIT_COMMITTER_NAME", author_name)
        .env("GIT_COMMITTER_EMAIL", author_email)
        .output()
        .unwrap();
}

pub fn commit_messages() -> Vec<String> {
    // git rev-list --all --oneline
    let output = Command::new("git")
        .args(["rev-list", "--oneline", "HEAD"])
        .current_dir(repo_path())
        .output()
        .unwrap();
    let mut messages = Vec::new();
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let message = line.split_once(' ').unwrap().1;
        messages.push(message.to_string());
    }
    messages
}

pub fn move_decomp_output_into_repo(out_path: &Path, exclude_paths: &[&str]) {
    // clear repo first
    let repo_path = repo_path();
    let entries_in_repo_path = repo_path
        .read_dir()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    if !entries_in_repo_path.iter().any(|e| e.file_name() == ".git") {
        panic!("tried to copy files into a directory without a .git!")
    }

    println!("deleting everything in {repo_path:?} (except the .git)");
    for entry in entries_in_repo_path {
        if entry.file_name() == ".git" {
            continue;
        };
        if entry.path().is_dir() {
            if let Err(e) = fs::remove_dir_all(entry.path()) {
                println!("couldn't remove directory {entry:?}! {e}");
            }
        } else if let Err(e) = fs::remove_file(entry.path()) {
            println!("couldn't remove file {entry:?}! {e}");
        }
    }

    println!("copying stuff from {out_path:?} to {repo_path:?}");
    // copy everything from tmp/out except assets and data
    for path_name_to_delete in exclude_paths {
        let mut path_to_delete = out_path.to_owned();
        path_to_delete.push(path_name_to_delete);
        let _ = fs::remove_dir_all(path_to_delete);
    }
    for entry in out_path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let mut new_path = PathBuf::from(REPO_PATH);
        new_path.push(entry.path().file_name().unwrap());
        fs::rename(entry.path(), new_path).unwrap();
    }
}
