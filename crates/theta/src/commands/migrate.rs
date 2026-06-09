//! `theta migrate` — migrate `theta.toml` to the latest schema version.

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use theta_args::MigrateArgs;
use theta_manifest::{read_document, read_manifest, schema_version};
use theta_static::SCHEMA_VERSION;

pub(crate) fn execute(_args: MigrateArgs, manifest_path: &Path) -> Result<()> {
    super::require_manifest(manifest_path)?;

    let doc = read_document(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let version = schema_version(&doc)?;

    // validate the manifest deserializes cleanly
    let _manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to validate {}", manifest_path.display()))?;

    if version == SCHEMA_VERSION {
        anstream::eprintln!(
            "{} nothing to migrate - only one schema version exists ({})",
            "ok".green().bold(),
            SCHEMA_VERSION.cyan()
        );
        return Ok(());
    }

    anyhow::bail!("migration from {version} to {SCHEMA_VERSION} is not yet implemented");
}
