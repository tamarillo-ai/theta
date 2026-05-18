//! `git` CLI wrapper -- shells out to system `git` binary.
//!
//! Derived from uv-git/src/git.rs and Cargo's git support (Apache-2.0/MIT).
//! Uses the system git CLI (not libgit2) to avoid build-time C dependencies.

use std::path::Path;
use std::process::{Command, Output};

use crate::GitError;

pub(crate) fn git_cmd(args: &[&str], cwd: Option<&Path>) -> Result<Output, GitError> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_ASKPASS", "echo");

    cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            GitError::GitNotFound
        } else {
            GitError::Io(e)
        }
    })
}

fn git_output(args: &[&str], cwd: Option<&Path>) -> Result<String, GitError> {
    let output = git_cmd(args, cwd)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: args.join(" "),
            stderr: stderr.trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn init_bare(path: &Path) -> Result<(), GitError> {
    fs_err::create_dir_all(path)?;
    let output = git_cmd(&["init", "--bare"], Some(path))?;
    if !output.status.success() {
        return Err(GitError::CommandFailed {
            command: "init --bare".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

pub(crate) fn fetch_remote(db_path: &Path, url: &str, refspecs: &[String]) -> Result<(), GitError> {
    let mut args = vec!["fetch", "--force", "--update-head-ok", url];
    let refspec_strs: Vec<&str> = refspecs.iter().map(std::string::String::as_str).collect();
    args.extend_from_slice(&refspec_strs);

    let output = git_cmd(&args, Some(db_path))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::FetchFailed {
            url: url.to_string(),
            stderr: stderr.trim().to_string(),
        });
    }
    Ok(())
}

pub(crate) fn fetch_all(db_path: &Path, url: &str) -> Result<(), GitError> {
    let refspecs = vec![
        "+refs/heads/*:refs/remotes/origin/*".to_string(),
        "+refs/tags/*:refs/tags/*".to_string(),
    ];
    fetch_remote(db_path, url, &refspecs)
}

pub(crate) fn rev_parse(db_path: &Path, refspec: &str) -> Result<crate::CommitSha, GitError> {
    let spec = format!("{refspec}^0");
    let out = git_output(&["rev-parse", &spec], Some(db_path))?;
    out.parse().map_err(|_| GitError::InvalidSha(out))
}

pub(crate) fn clone_local(db_path: &Path, checkout_path: &Path) -> Result<(), GitError> {
    if checkout_path.exists() {
        fs_err::remove_dir_all(checkout_path)?;
    }
    if let Some(parent) = checkout_path.parent() {
        fs_err::create_dir_all(parent)?;
    }

    // pass paths as &OsStr directly - avoids to_string_lossy() which
    // silently replaces non-UTF-8 bytes with U+FFFD
    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg("--local")
        .arg("--no-checkout")
        .arg(db_path)
        .arg(checkout_path);
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_ASKPASS", "echo");

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            GitError::GitNotFound
        } else {
            GitError::Io(e)
        }
    })?;
    if !output.status.success() {
        return Err(GitError::CommandFailed {
            command: "clone --local".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

pub(crate) fn checkout(checkout_path: &Path, sha: &crate::CommitSha) -> Result<(), GitError> {
    let output = git_cmd(&["checkout", "--force", sha.as_str()], Some(checkout_path))?;
    if !output.status.success() {
        return Err(GitError::CommandFailed {
            command: format!("checkout {sha}"),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(())
}

pub(crate) fn has_commit(db_path: &Path, sha: &crate::CommitSha) -> bool {
    git_cmd(&["cat-file", "-t", sha.as_str()], Some(db_path))
        .ok()
        .is_some_and(|o| o.status.success())
}

pub(crate) fn head_sha(checkout_path: &Path) -> Result<crate::CommitSha, GitError> {
    let out = git_output(&["rev-parse", "HEAD"], Some(checkout_path))?;
    out.parse().map_err(|_| GitError::InvalidSha(out))
}

pub(crate) fn refspecs_for(git_ref: &crate::GitRef) -> Vec<String> {
    match git_ref {
        crate::GitRef::Branch(b) => {
            vec![format!("+refs/heads/{b}:refs/remotes/origin/{b}")]
        }
        crate::GitRef::Tag(t) => {
            vec![format!("+refs/tags/{t}:refs/tags/{t}")]
        }
        crate::GitRef::Commit(_) => {
            vec![
                "+refs/heads/*:refs/remotes/origin/*".to_string(),
                "+refs/tags/*:refs/tags/*".to_string(),
            ]
        }
        crate::GitRef::DefaultBranch => {
            vec!["+HEAD:refs/remotes/origin/HEAD".to_string()]
        }
    }
}

pub(crate) fn resolve_refspec(git_ref: &crate::GitRef) -> String {
    match git_ref {
        crate::GitRef::Branch(b) => format!("refs/remotes/origin/{b}"),
        crate::GitRef::Tag(t) => format!("refs/tags/{t}"),
        crate::GitRef::Commit(sha) => sha.as_str().to_string(),
        crate::GitRef::DefaultBranch => "refs/remotes/origin/HEAD".to_string(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_is_available() {
        let output = git_cmd(&["--version"], None).expect("git should be on PATH");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("git version"));
    }

    #[test]
    fn init_bare_creates_valid_repo() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("test.git");
        init_bare(&db).unwrap();
        assert!(db.join("HEAD").exists());
        assert!(db.join("objects").exists());
    }

    #[test]
    fn refspecs_for_branch() {
        let refs = refspecs_for(&crate::GitRef::Branch("main".into()));
        assert_eq!(refs, vec!["+refs/heads/main:refs/remotes/origin/main"]);
    }

    #[test]
    fn refspecs_for_tag() {
        let refs = refspecs_for(&crate::GitRef::Tag("v1.0".into()));
        assert_eq!(refs, vec!["+refs/tags/v1.0:refs/tags/v1.0"]);
    }

    #[test]
    fn refspecs_for_default_branch() {
        let refs = refspecs_for(&crate::GitRef::DefaultBranch);
        assert_eq!(refs, vec!["+HEAD:refs/remotes/origin/HEAD"]);
    }

    #[test]
    fn resolve_refspec_round_trips() {
        let branch = resolve_refspec(&crate::GitRef::Branch("main".into()));
        assert_eq!(branch, "refs/remotes/origin/main");

        let tag = resolve_refspec(&crate::GitRef::Tag("v2".into()));
        assert_eq!(tag, "refs/tags/v2");

        let sha = crate::CommitSha::new(&"a".repeat(40)).unwrap();
        let commit = resolve_refspec(&crate::GitRef::Commit(sha));
        assert_eq!(commit, "a".repeat(40));
    }
}
