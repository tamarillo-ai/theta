//! Shared exhaust shapes reused across CLI verbs.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Serialize;

/// Verb that performed a mutation against either the manifest or the system store.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MutationKind {
    Add,
    Remove,
    Register,
    Unregister,
}

/// Manifest or store entity that the mutation targets.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EntityKind {
    Rule,
    System,
    Tool,
    Skill,
    Subagent,
    Agent,
}

/// Origin description for a newly-added entry. `kind` describes which
/// constructor of the manifest's `src`/`source` union was used; `detail`
/// carries the human-meaningful identifier (path, URL, store name).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct MutationSource {
    pub kind: MutationSourceKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MutationSourceKind {
    Local,
    Git,
    Store,
    Inline,
    Description,
}

/// Uniform payload for every `add` / `rm` / `register` / unregister variant.
///
/// `files_written` and `files_deleted` are project-relative when the operation
/// targets a project, absolute when targeting the system store.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct MutationOutput {
    pub kind: MutationKind,
    pub entity: EntityKind,
    pub name: Option<String>,
    pub source: Option<MutationSource>,
    pub files_written: Vec<PathBuf>,
    pub files_deleted: Vec<PathBuf>,
}
