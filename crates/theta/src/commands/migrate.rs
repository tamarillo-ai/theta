//! `theta migrate` — migrate `theta.toml` to the latest schema version.

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{MigrateArgs, OutputFormat};
use theta_manifest::{read_document, read_manifest, schema_version};
use theta_static::SCHEMA_VERSION;

use super::output::present_no_op;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct MigrateOutcome {
    pub from_version: String,
    pub to_version: String,
    pub migrated: bool,
}

pub(crate) fn execute(
    _args: MigrateArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    super::require_manifest(manifest_path)?;

    let doc = read_document(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let version = schema_version(&doc)?;

    // validate the manifest deserializes cleanly
    let _manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to validate {}", manifest_path.display()))?;

    if version == SCHEMA_VERSION {
        let outcome = MigrateOutcome {
            from_version: version.to_string(),
            to_version: SCHEMA_VERSION.to_string(),
            migrated: false,
        };
        return present_no_op(&["migrate"], output_format, outcome, vec![], |_| {
            anstream::eprintln!(
                "{} nothing to migrate - only one schema version exists ({})",
                "ok".green().bold(),
                SCHEMA_VERSION.cyan()
            );
        });
    }

    anyhow::bail!("migration from {version} to {SCHEMA_VERSION} is not yet implemented");
}
