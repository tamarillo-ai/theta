use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
/// A validated SHA-256 content hash stored as 32 raw bytes.
///
/// On disk (TOML), serialized as `"sha256:<64 hex chars>"` via `DisplayFromStr`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Wrap raw SHA-256 bytes into a `ContentHash`.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{}", hex::encode(self.0))
    }
}

/// Parse error for `ContentHash` — input didn't match `"sha256:<64 hex>"`.
#[derive(Debug, thiserror::Error)]
#[error("invalid content hash: expected 'sha256:<64 hex chars>', got '{0}'")]
pub struct InvalidContentHash(String);

impl FromStr for ContentHash {
    type Err = InvalidContentHash;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_str = s
            .strip_prefix("sha256:")
            .ok_or_else(|| InvalidContentHash(s.to_string()))?;
        let bytes: Vec<u8> = hex::decode(hex_str).map_err(|_| InvalidContentHash(s.to_string()))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| InvalidContentHash(s.to_string()))?;
        Ok(Self(arr))
    }
}

impl From<ContentHash> for String {
    fn from(h: ContentHash) -> String {
        h.to_string()
    }
}

/// The full `theta.lock` file — manifest hash plus per-resource locks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct LockFile {
    /// Schema version and manifest hash
    pub meta: LockMeta,
    /// Locked instructions (system prompt + rules)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<InstructionsLock>,
    /// Locked skills, keyed by skill name
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub skills: BTreeMap<String, ResourceLock>,
    /// Locked subagents, keyed by subagent name
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subagents: BTreeMap<String, SubagentLock>,
}

impl LockFile {
    /// Construct a `LockFile` from its parts.
    pub fn new(
        meta: LockMeta,
        instructions: Option<InstructionsLock>,
        skills: BTreeMap<String, ResourceLock>,
        subagents: BTreeMap<String, SubagentLock>,
    ) -> Self {
        Self {
            meta,
            instructions,
            skills,
            subagents,
        }
    }
}

/// Locked state for a single subagent — either a full child manifest (`ref`)
/// or a local prompt file (`prompt_path`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SubagentLock {
    /// `ref` subagent — a full child agent manifest
    Ref {
        /// The subagent's own manifest lock
        #[serde(flatten)]
        resource: ResourceLock,
        /// Child instructions, if any
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instructions: Option<InstructionsLock>,
        /// Child skills, if any
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        skills: BTreeMap<String, ResourceLock>,
    },
    /// `prompt_path` subagent — system prompt in a local `.md` file
    Inline {
        /// The prompt file lock
        prompt: ResourceLock,
    },
}

impl SubagentLock {
    /// The underlying resource lock, regardless of subagent shape.
    pub fn as_resource_lock(&self) -> &ResourceLock {
        match self {
            Self::Ref { resource, .. } => resource,
            Self::Inline { prompt } => prompt,
        }
    }
}

/// Top-level metadata: schema version and a hash of the manifest that
/// produced this lock.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct LockMeta {
    /// theta-spec schema version (e.g. `"2026-04"`)
    pub schema: String,
    /// Canonical hash of the `theta.toml` that produced this lock
    #[serde_as(as = "DisplayFromStr")]
    pub manifest_hash: ContentHash,
}

/// Locked state for the `[instructions]` section — system prompt and rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct InstructionsLock {
    /// Locked system prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<ResourceLock>,
    /// Locked rules, keyed by rule name
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rules: BTreeMap<String, ResourceLock>,
}

/// A single locked resource: where it came from and its content hash
/// at lock time.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResourceLock {
    /// Where the resource was resolved from
    pub source: LockedSource,
    /// SHA-256 of the content at lock time
    #[serde_as(as = "DisplayFromStr")]
    pub content_hash: ContentHash,
}

/// Where a locked resource lives — local path, git repo, URL, registry,
/// or system store.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum LockedSource {
    /// Local path relative to project root
    Path {
        /// Project-relative path
        path: String,
    },
    /// Git repository, pinned to a resolved commit
    Git {
        /// Repository URL
        git: String,
        /// The ref that was requested (branch, tag, or SHA)
        #[serde(rename = "ref")]
        git_ref: String,
        /// The exact commit SHA the ref resolved to
        #[serde_as(as = "DisplayFromStr")]
        resolved_commit: theta_git::CommitSha,
        /// Subdirectory within the repo, if any
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subdirectory: Option<String>,
        /// Specific file within the subdirectory, if any
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file: Option<String>,
    },
    /// Remote URL (direct download)
    Url {
        /// The URL
        url: String,
    },
    /// Registry-hosted resource
    Registry {
        /// Registry base URL
        registry: String,
        /// Package name
        name: String,
        /// Resolved version
        version: String,
    },
    /// System store resource (installed via `theta store`)
    System {
        /// Name in the system store
        system: String,
    },
}
