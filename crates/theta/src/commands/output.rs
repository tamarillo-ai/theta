//! Shared exhaust shapes reused across CLI verbs.

use std::path::PathBuf;

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::OutputFormat;
use theta_schema::{CommandFailure, CommandOutput, Diagnostic};

/// Wrap a verb outcome in the canonical envelope and route it based on
/// `output_format`. JSON mode prints the envelope to stdout; Human mode
/// invokes `render`. This is the single place verbs decide which output
/// channel to use.
///
/// `render` is only called in human mode, so it is free to write to stderr
/// via `anstream::eprintln!`.
pub(crate) fn present<T, F>(
    verb: &[&str],
    output_format: OutputFormat,
    outcome: T,
    diagnostics: Vec<Diagnostic>,
    render: F,
) -> Result<()>
where
    T: Serialize + JsonSchema,
    F: FnOnce(&T),
{
    match output_format {
        OutputFormat::Json => {
            let mut env = CommandOutput::ok(verb.iter().copied(), outcome);
            env.diagnostics = diagnostics;
            env.print_json()?;
            Ok(())
        }
        OutputFormat::Human => {
            render(&outcome);
            Ok(())
        }
    }
}

/// Variant of [`present`] that emits a `noop` envelope in JSON mode.
pub(crate) fn present_no_op<T, F>(
    verb: &[&str],
    output_format: OutputFormat,
    outcome: T,
    diagnostics: Vec<Diagnostic>,
    render: F,
) -> Result<()>
where
    T: Serialize + JsonSchema,
    F: FnOnce(&T),
{
    match output_format {
        OutputFormat::Json => {
            let mut env = CommandOutput::no_op(verb.iter().copied(), outcome);
            env.diagnostics = diagnostics;
            env.print_json()?;
            Ok(())
        }
        OutputFormat::Human => {
            render(&outcome);
            Ok(())
        }
    }
}

/// Variant of [`present`] for error outcomes. JSON mode emits an `error`
/// envelope and returns [`CommandFailure`] so the CLI boundary exits non-zero
/// without printing the human banner. Human mode renders the outcome and
/// returns the supplied [`anyhow::Error`].
pub(crate) fn present_error<T, F>(
    verb: &[&str],
    output_format: OutputFormat,
    outcome: T,
    diagnostics: Vec<Diagnostic>,
    render: F,
    human_error: anyhow::Error,
) -> Result<()>
where
    T: Serialize + JsonSchema,
    F: FnOnce(&T),
{
    match output_format {
        OutputFormat::Json => {
            CommandOutput::error(verb.iter().copied(), outcome, diagnostics).print_json()?;
            Err(CommandFailure.into())
        }
        OutputFormat::Human => {
            render(&outcome);
            Err(human_error)
        }
    }
}

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
