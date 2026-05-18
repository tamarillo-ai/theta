//! `theta.lock` types, hashing, and staleness detection.
//!
//! Owns the `LockFile` struct and per-resource `ResourceLock` entries.
//! Handles TOML serialization, SHA-256 content hashing, and manifest-to-lock
//! staleness comparison. Does NOT own network fetching.

#![warn(missing_docs)]

mod build;
mod error;
mod hash;
mod io;
mod stale;
mod types;

pub use build::build_lock;
pub use error::{BuildError, LockError, ManifestHashError};
pub use hash::{content_hash, manifest_hash, skill_content_hash};
pub use io::{read_lock, write_lock};
pub use stale::{StaleReason, is_stale};
pub use types::{
    ContentHash, InstructionsLock, LockFile, LockMeta, LockedSource, ResourceLock, SubagentLock,
};

#[cfg(test)]
mod tests;
