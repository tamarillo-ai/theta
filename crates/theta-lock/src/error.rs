use std::path::PathBuf;

/// I/O and serialization errors for lock file operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LockError {
    /// Failed to read the lock file from disk
    #[error("failed to read {path}: {source}")]
    Read {
        /// File that couldn't be read
        path: PathBuf,
        /// Underlying I/O error
        source: std::io::Error,
    },
    /// Failed to write the lock file to disk
    #[error("failed to write {path}: {source}")]
    Write {
        /// File that couldn't be written
        path: PathBuf,
        /// Underlying I/O error
        source: std::io::Error,
    },
    /// Lock file exists but contains invalid TOML
    #[error("failed to parse {path}: {source}")]
    Parse {
        /// File that couldn't be parsed
        path: PathBuf,
        /// TOML parse error
        source: toml::de::Error,
    },
    /// Failed to serialize the lock struct to TOML
    #[error("failed to serialize lock: {source}")]
    Serialize {
        /// TOML serialization error
        source: toml::ser::Error,
    },
}

/// Failures when computing the canonical manifest hash — the manifest
/// must be valid UTF-8 and valid TOML to produce a stable hash.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestHashError {
    /// Raw bytes aren't valid UTF-8
    #[error("theta.toml is not valid UTF-8")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    /// Valid UTF-8 but not valid TOML (or not a valid `ThetaManifest`)
    #[error("theta.toml has invalid TOML syntax: {0}")]
    InvalidToml(#[from] toml::de::Error),
    /// Parsed fine but couldn't be re-serialized (shouldn't happen)
    #[error("failed to re-serialize manifest for canonical hashing: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// A single error encountered while resolving sources during `build_lock`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildError {
    /// `theta.toml` itself couldn't be hashed
    #[error("{0}")]
    ManifestHash(
        /// The underlying hash error
        #[from]
        ManifestHashError,
    ),
    /// A local file or directory couldn't be read
    #[error("{resource}: {path} - {source}")]
    SourceNotFound {
        /// Which manifest entry (e.g. `"instructions.rules.safety"`)
        resource: String,
        /// The resolved path that was missing
        path: PathBuf,
        /// Underlying I/O error
        source: std::io::Error,
    },
    /// A system store resource isn't installed
    #[error(
        "{resource}: system store {kind} '{name}' not found - run `theta register {kind} {name}` to add it"
    )]
    SystemStoreNotFound {
        /// Which manifest entry
        resource: String,
        /// `"rule"` or `"skill"`
        kind: &'static str,
        /// The store name
        name: String,
    },
    /// Couldn't determine the system data directory
    #[error("{resource}: could not determine system store directory")]
    NoDataDir {
        /// Which manifest entry
        resource: String,
    },
    /// `git fetch` failed
    #[error("{resource}: {source}")]
    GitFetch {
        /// Which manifest entry
        resource: String,
        /// The underlying git error
        source: theta_git::GitError,
    },
    /// Git fetch succeeded but the expected file wasn't in the checkout
    #[error("{resource}: {file} not found in {url} at {commit}")]
    GitFileNotFound {
        /// Which manifest entry
        resource: String,
        /// The URL that was fetched
        url: String,
        /// Short commit SHA
        commit: String,
        /// File or directory that was expected
        file: String,
    },
    /// Hashing a skill directory failed (dir exists but can't be walked)
    #[error("{resource}: failed to hash: {source}")]
    HashFailed {
        /// Which manifest entry
        resource: String,
        /// Underlying I/O error
        source: std::io::Error,
    },
    /// A subagent's child manifest couldn't be parsed
    #[error("{resource}: failed to parse child manifest: {source}")]
    ChildManifestParse {
        /// Which manifest entry
        resource: String,
        /// TOML parse error
        source: toml::de::Error,
    },
    /// Catch-all for source types that aren't yet implemented
    #[error("{resource}: unsupported source type")]
    UnsupportedSource {
        /// Which manifest entry
        resource: String,
    },
}
