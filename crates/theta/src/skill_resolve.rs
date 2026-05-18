//! Shared skill resolution logic used by `add skill` and `register skill`.
//!
//! Owns: GitHub shorthand parsing, git checkout fetching, and the
//! `ResolvedSkill` struct that both commands consume.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use theta_store::extract_skill_description;

pub(crate) struct ResolvedSkill {
    pub name: String,
    pub description: String,
    pub dir: PathBuf,
}

pub(crate) struct ParsedGitHubRef {
    pub owner: String,
    pub repo: String,
    pub subdirectory: Option<String>,
    pub git_ref: Option<String>,
    pub inferred_name: String,
}

impl ParsedGitHubRef {
    pub(crate) fn git_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repo)
    }
}

pub(crate) fn parse_github_ref(input: &str) -> Result<ParsedGitHubRef> {
    let (path, git_ref) = match input.rsplit_once('@') {
        Some((p, r)) => (p, Some(r.to_string())),
        None => (input, None),
    };
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 {
        bail!("github reference must have at least 2 segments (owner/repo), got: {input}");
    }
    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let subdirectory = if segments.len() > 2 {
        Some(segments[2..].join("/"))
    } else {
        None
    };
    let inferred_name = subdirectory
        .as_ref()
        .and_then(|s| s.rsplit('/').next())
        .unwrap_or(&repo)
        .to_string();
    Ok(ParsedGitHubRef {
        owner,
        repo,
        subdirectory,
        git_ref,
        inferred_name,
    })
}

pub(crate) fn fetch_git_checkout(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    subdirectory: Option<&str>,
) -> Result<PathBuf> {
    let cache = theta_git::cache_dir()?;
    let fetcher = theta_git::GitFetcher::new(cache);
    let reference = theta_git::GitRef::from_manifest(branch, tag, rev);
    let result = fetcher
        .fetch(git_url, &reference, None)
        .with_context(|| format!("failed to fetch {git_url}"))?;
    match subdirectory {
        Some(sub) => Ok(result.path.join(sub)),
        None => Ok(result.path),
    }
}

/// Read `SKILL.md` from a directory and extract the description field.
pub(crate) fn read_skill_description(dir: &Path) -> String {
    let skill_md_path = dir.join(theta_static::SKILL_FILE_NAME);
    match fs_err::read_to_string(&skill_md_path) {
        Ok(content) => extract_skill_description(&content),
        Err(e) => {
            tracing::debug!(path = %skill_md_path.display(), error = %e, "could not read skill description");
            String::new()
        }
    }
}
