//! Cache layout and digest computation.
//!
//! Derived from uv-git cache patterns (Apache-2.0/MIT).

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub fn url_digest(url: &str) -> String {
    let canonical = canonical_url(url);
    let hash = Sha256::digest(canonical.as_bytes());
    hex::encode(hash)
}

// NOTE: adapted from uv's `CanonicalUrl::new()` (Apache-2.0/MIT)
// https://github.com/astral-sh/uv/blob/main/crates/uv-cache-key/src/canonical_url.rs
//
// uv operates on `url::Url`; we operate on raw strings because our URLs
// go directly to the git CLI (which handles all forms natively)
// normalization steps match uv: lowercase, strip creds, strip trailing /,
// strip .git suffix. this ensures that different spellings of the same
// repo produce the same cache digest.
fn canonical_url(url: &str) -> String {
    let mut s = url.to_lowercase();

    // strip credentials (user:pass@) from the authority
    // e.g. https://user:token@github.com/org/repo --> https://github.com/org/repo
    if let Some(scheme_end) = s.find("://") {
        let authority_start = scheme_end + 3;
        if let Some(at_pos) = s[authority_start..].find('@') {
            s = format!(
                "{}{}",
                &s[..authority_start],
                &s[authority_start + at_pos + 1..]
            );
        }
    }

    // strip trailing slashes
    while s.ends_with('/') {
        s.pop();
    }

    // strip .git suffix - repos are accessible with or without it
    // handles both "repo.git" and "repo.git@ref" (ref after .git)
    if let Some(base) = s.strip_suffix(".git") {
        s = base.to_string();
    } else if let Some((prefix, suffix)) = s.rsplit_once('@') {
        if let Some(stripped) = prefix.strip_suffix(".git") {
            s = format!("{stripped}@{suffix}");
        }
    }

    s
}

pub(crate) struct CachePaths<'a> {
    root: &'a Path,
}

impl<'a> CachePaths<'a> {
    pub(crate) fn new(cache_dir: &'a Path) -> Self {
        Self { root: cache_dir }
    }

    pub(crate) fn db(&self, digest: &str) -> PathBuf {
        self.root.join("db").join(digest)
    }

    pub(crate) fn checkout(&self, digest: &str, short_sha: &str) -> PathBuf {
        self.root.join("checkouts").join(digest).join(short_sha)
    }

    pub(crate) fn lock_file(&self, digest: &str) -> PathBuf {
        self.root.join("locks").join(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_deterministic() {
        let a = url_digest("https://github.com/foo/bar");
        let b = url_digest("https://github.com/foo/bar");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_normalizes_git_suffix() {
        let a = url_digest("https://github.com/foo/bar.git");
        let b = url_digest("https://github.com/foo/bar");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_normalizes_case() {
        let a = url_digest("https://GitHub.com/Foo/Bar");
        let b = url_digest("https://github.com/foo/bar");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_normalizes_trailing_slash() {
        let a = url_digest("https://github.com/foo/bar/");
        let b = url_digest("https://github.com/foo/bar");
        assert_eq!(a, b);
    }

    #[test]
    fn cache_paths_layout() {
        let root = Path::new("/tmp/theta-cache/git");
        let paths = CachePaths::new(root);
        let d = "abc123";
        assert_eq!(paths.db(d), PathBuf::from("/tmp/theta-cache/git/db/abc123"));
        assert_eq!(
            paths.checkout(d, "deadbee"),
            PathBuf::from("/tmp/theta-cache/git/checkouts/abc123/deadbee")
        );
        assert_eq!(
            paths.lock_file(d),
            PathBuf::from("/tmp/theta-cache/git/locks/abc123")
        );
    }

    #[test]
    fn digest_strips_credentials() {
        let with_creds = url_digest("https://user:token@github.com/org/repo");
        let without_creds = url_digest("https://github.com/org/repo");
        assert_eq!(with_creds, without_creds);
    }

    #[test]
    fn digest_strips_credentials_with_only_username() {
        let with_user = url_digest("https://user@github.com/org/repo");
        let without = url_digest("https://github.com/org/repo");
        assert_eq!(with_user, without);
    }

    #[test]
    fn digest_strips_git_suffix_before_ref() {
        let with_ref = url_digest("https://github.com/org/repo.git@v1.0");
        let without = url_digest("https://github.com/org/repo@v1.0");
        assert_eq!(with_ref, without);
    }

    #[test]
    fn digest_all_normalizations_combined() {
        let messy = url_digest("https://User:Pass@GitHub.com/Org/Repo.git/");
        let clean = url_digest("https://github.com/org/repo");
        assert_eq!(messy, clean);
    }

    #[test]
    fn digest_double_git_suffix_strips_only_one() {
        // ".git.git" should strip the outer .git, leaving ".git"
        let double = canonical_url("https://github.com/org/repo.git.git");
        assert_eq!(double, "https://github.com/org/repo.git");
    }

    #[test]
    fn digest_git_suffix_with_trailing_slash() {
        let a = url_digest("https://github.com/org/repo.git/");
        let b = url_digest("https://github.com/org/repo");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_multiple_trailing_slashes() {
        let a = url_digest("https://github.com/org/repo///");
        let b = url_digest("https://github.com/org/repo");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_url_with_query_string() {
        // query strings are preserved - different query = different digest
        // (this is correct: we don't strip query params)
        let with_query = canonical_url("https://github.com/org/repo?token=abc");
        assert_eq!(with_query, "https://github.com/org/repo?token=abc");
    }

    #[test]
    fn digest_url_with_fragment() {
        let with_fragment = canonical_url("https://github.com/org/repo#readme");
        assert_eq!(with_fragment, "https://github.com/org/repo#readme");
    }

    #[test]
    fn digest_ssh_url_preserves_git_user() {
        // ssh://git@host is standard - git@ is the username, not credentials
        let ssh = canonical_url("ssh://git@github.com/org/repo");
        assert_eq!(ssh, "ssh://github.com/org/repo");
    }

    #[test]
    fn digest_empty_url() {
        let empty = canonical_url("");
        assert_eq!(empty, "");
    }

    #[test]
    fn digest_no_scheme_passthrough() {
        // SCP-form or bare strings pass through (validation catches these elsewhere)
        let scp = canonical_url("git@github.com:org/repo.git");
        assert_eq!(scp, "git@github.com:org/repo");
    }
}
