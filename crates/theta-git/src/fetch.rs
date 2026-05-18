//! High-level fetch: clone/fetch + ref resolution + checkout.

use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::cache::{CachePaths, url_digest};
use crate::git;
use crate::{CommitSha, GitError, GitRef};

pub struct FetchResult {
    pub commit: CommitSha,
    pub path: PathBuf,
}

pub struct GitFetcher {
    cache_dir: PathBuf,
}

impl GitFetcher {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn fetch(
        &self,
        url: &str,
        git_ref: &GitRef,
        locked_commit: Option<&CommitSha>,
    ) -> Result<FetchResult, GitError> {
        let digest = url_digest(url);
        let paths = CachePaths::new(&self.cache_dir);
        let db_path = paths.db(&digest);

        let lock_path = paths.lock_file(&digest);
        let _lock = acquire_lock(&lock_path)?;

        if let Some(locked) = locked_commit {
            let checkout_path = paths.checkout(&digest, locked.short());
            if checkout_path.exists() {
                if let Ok(head) = git::head_sha(&checkout_path) {
                    if head == *locked {
                        debug!(url, commit = locked.short(), "cache hit (locked commit)");
                        return Ok(FetchResult {
                            commit: locked.clone(),
                            path: checkout_path,
                        });
                    }
                }
            }
        }

        if !db_path.join("HEAD").exists() {
            info!(url, "initializing git database");
            git::init_bare(&db_path)?;
        }

        info!(url, git_ref = %git_ref, "fetching");
        let refspecs = git::refspecs_for(git_ref);
        match git::fetch_remote(&db_path, url, &refspecs) {
            Ok(()) => {}
            Err(_) if !matches!(git_ref, GitRef::DefaultBranch) => {
                debug!(url, "targeted fetch failed, falling back to fetch all");
                git::fetch_all(&db_path, url)?;
            }
            Err(e) => return Err(e),
        }

        let commit = if let Some(locked) = locked_commit {
            if !git::has_commit(&db_path, locked) {
                return Err(GitError::RefNotFound {
                    url: url.to_string(),
                    reference: locked.to_string(),
                });
            }
            locked.clone()
        } else {
            let refspec = git::resolve_refspec(git_ref);
            git::rev_parse(&db_path, &refspec).map_err(|_| GitError::RefNotFound {
                url: url.to_string(),
                reference: git_ref.to_string(),
            })?
        };

        let checkout_path = paths.checkout(&digest, commit.short());

        if checkout_path.exists() {
            if let Ok(head) = git::head_sha(&checkout_path) {
                if head == commit {
                    debug!(url, commit = commit.short(), "cache hit (checkout exists)");
                    return Ok(FetchResult {
                        commit,
                        path: checkout_path,
                    });
                }
            }
            fs_err::remove_dir_all(&checkout_path)?;
        }

        info!(url, commit = commit.short(), "checking out");
        git::clone_local(&db_path, &checkout_path)?;
        git::checkout(&checkout_path, &commit)?;

        Ok(FetchResult {
            commit,
            path: checkout_path,
        })
    }

    pub fn is_cached(&self, url: &str, commit: &CommitSha) -> bool {
        let digest = url_digest(url);
        let paths = CachePaths::new(&self.cache_dir);
        let checkout = paths.checkout(&digest, commit.short());
        if !checkout.exists() {
            return false;
        }
        git::head_sha(&checkout)
            .ok()
            .is_some_and(|head| head == *commit)
    }

    pub fn evict(&self, url: &str) -> Result<(), GitError> {
        let digest = url_digest(url);
        let paths = CachePaths::new(&self.cache_dir);
        let db = paths.db(&digest);
        if db.exists() {
            fs_err::remove_dir_all(&db)?;
        }
        let checkouts_dir = self.cache_dir.join("checkouts").join(&digest);
        if checkouts_dir.exists() {
            fs_err::remove_dir_all(&checkouts_dir)?;
        }
        let lock_file = paths.lock_file(&digest);
        if lock_file.exists() {
            fs_err::remove_file(&lock_file)?;
        }
        Ok(())
    }

    pub fn clean(&self) -> Result<(), GitError> {
        if self.cache_dir.exists() {
            fs_err::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
}

// per-URL advisory file lock via fs2 flock().
// the lock is held for the duration of the fetch (clone + checkout)
// safe across threads (rayon) and processes (concurrent theta runs)
// on drop, the File closes and the OS releases the lock automatically

fn acquire_lock(lock_path: &Path) -> Result<std::fs::File, GitError> {
    use fs2::FileExt;

    if let Some(parent) = lock_path.parent() {
        fs_err::create_dir_all(parent)?;
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(lock_path)?;
    file.lock_exclusive().map_err(|e| GitError::LockFailed {
        path: lock_path.to_path_buf(),
        source: e,
    })?;
    Ok(file)
}
