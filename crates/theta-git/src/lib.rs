//! `git` fetch, ref resolution, and content-addressed cache for theta.
//!
//! `git` support is derived from Cargo's implementation via uv.
//! Cargo is dual-licensed under Apache 2.0 or MIT, at the user's choice.
//! uv source: <https://github.com/astral-sh/uv/tree/main/crates/uv-git>.
//! Cargo source: <https://github.com/rust-lang/cargo>.

mod cache;
mod fetch;
mod git;

pub use cache::url_digest;
pub use fetch::{FetchResult, GitFetcher};

use std::fmt;
use std::path::PathBuf;

pub fn cache_dir() -> Result<PathBuf, GitError> {
    theta_dirs::cache_dir()
        .map(|d| d.join("git"))
        .ok_or(GitError::CacheDirNotFound)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRef {
    Branch(String),
    Tag(String),
    Commit(CommitSha),
    DefaultBranch,
}

impl GitRef {
    /// Construct from explicit manifest fields — tries branch, tag, rev in
    /// order, then `DefaultBranch`.
    pub fn from_manifest(branch: Option<&str>, tag: Option<&str>, rev: Option<&str>) -> Self {
        if let Some(b) = branch {
            return Self::Branch(b.to_string());
        }
        if let Some(t) = tag {
            return Self::Tag(t.to_string());
        }
        if let Some(r) = rev {
            return r
                .parse::<CommitSha>()
                .map_or_else(|_| Self::Branch(r.to_string()), Self::Commit);
        }
        Self::DefaultBranch
    }

    /// Classify a bare ref string by heuristic — prefer `from_manifest` when
    /// the manifest distinguishes branch/tag/rev explicitly.
    pub fn from_str_heuristic(s: &str) -> Self {
        if s.is_empty() {
            return Self::DefaultBranch;
        }
        if let Ok(sha) = s.parse::<CommitSha>() {
            return Self::Commit(sha);
        }
        // tags typically look like v1.0, v2.3.1, etc.
        if s.starts_with('v') && s.len() > 1 && s.as_bytes()[1].is_ascii_digit() {
            return Self::Tag(s.to_string());
        }
        Self::Branch(s.to_string())
    }
}

impl fmt::Display for GitRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Branch(b) => write!(f, "branch:{b}"),
            Self::Tag(t) => write!(f, "tag:{t}"),
            Self::Commit(sha) => write!(f, "commit:{}", sha.short()),
            Self::DefaultBranch => write!(f, "HEAD"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitSha(String);

impl CommitSha {
    pub fn new(s: &str) -> Result<Self, GitError> {
        if s.len() != 40 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GitError::InvalidSha(s.to_string()));
        }
        Ok(Self(s.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0[..7]
    }
}

impl fmt::Display for CommitSha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for CommitSha {
    type Err = GitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git not found - install git and ensure it's on PATH")]
    GitNotFound,
    #[error("could not determine git cache directory")]
    CacheDirNotFound,
    #[error("failed to fetch {url}: {stderr}")]
    FetchFailed { url: String, stderr: String },
    #[error("ref '{reference}' not found in {url}")]
    RefNotFound { url: String, reference: String },
    #[error("subdirectory '{subdir}' not found in {url} at {commit}")]
    SubdirNotFound {
        url: String,
        commit: String,
        subdir: String,
    },
    #[error("git {command} failed: {stderr}")]
    CommandFailed { command: String, stderr: String },
    #[error("failed to acquire lock {}: {source}", path.display())]
    LockFailed {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("invalid commit SHA: {0}")]
    InvalidSha(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_sha_validation() {
        let valid = "a".repeat(40);
        assert!(CommitSha::new(&valid).is_ok());
        assert_eq!(CommitSha::new(&valid).unwrap().short(), "aaaaaaa");

        // wrong length
        assert!(CommitSha::new("abc123").is_err());
        // non-hex
        let invalid = format!("{}g", "a".repeat(39));
        assert!(CommitSha::new(&invalid).is_err());
    }

    #[test]
    fn commit_sha_normalizes_case() {
        let upper = "A".repeat(40);
        let sha = CommitSha::new(&upper).unwrap();
        assert_eq!(sha.as_str(), "a".repeat(40));
    }

    #[test]
    fn git_ref_from_str_heuristic() {
        assert_eq!(GitRef::from_str_heuristic(""), GitRef::DefaultBranch);
        assert_eq!(
            GitRef::from_str_heuristic("main"),
            GitRef::Branch("main".to_string())
        );
        assert_eq!(
            GitRef::from_str_heuristic("v1.0"),
            GitRef::Tag("v1.0".to_string())
        );
        assert_eq!(
            GitRef::from_str_heuristic("v2.3.1"),
            GitRef::Tag("v2.3.1".to_string())
        );
        let sha = "a".repeat(40);
        assert!(matches!(
            GitRef::from_str_heuristic(&sha),
            GitRef::Commit(_)
        ));
        // "v" alone is a branch
        assert_eq!(
            GitRef::from_str_heuristic("v"),
            GitRef::Branch("v".to_string())
        );
    }

    #[test]
    fn from_manifest_explicit_branch() {
        assert_eq!(
            GitRef::from_manifest(Some("main"), None, None),
            GitRef::Branch("main".to_string())
        );
    }

    #[test]
    fn from_manifest_explicit_tag() {
        assert_eq!(
            GitRef::from_manifest(None, Some("v1.0"), None),
            GitRef::Tag("v1.0".to_string())
        );
    }

    #[test]
    fn from_manifest_explicit_rev() {
        let sha = "a".repeat(40);
        assert!(matches!(
            GitRef::from_manifest(None, None, Some(&sha)),
            GitRef::Commit(_)
        ));
    }

    #[test]
    fn from_manifest_rev_non_sha_falls_back_to_branch() {
        // rev that isn't a valid SHA - treated as branch (git will resolve it)
        assert_eq!(
            GitRef::from_manifest(None, None, Some("HEAD~3")),
            GitRef::Branch("HEAD~3".to_string())
        );
    }

    #[test]
    fn from_manifest_no_fields_is_default_branch() {
        assert_eq!(
            GitRef::from_manifest(None, None, None),
            GitRef::DefaultBranch
        );
    }

    #[test]
    fn from_manifest_branch_takes_priority() {
        // if multiple are somehow set, branch wins (validated elsewhere)
        assert_eq!(
            GitRef::from_manifest(Some("main"), Some("v1.0"), None),
            GitRef::Branch("main".to_string())
        );
    }
}
