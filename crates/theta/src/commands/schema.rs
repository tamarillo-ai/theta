//! `theta schema` — print the `theta.toml` JSON Schema.

#![allow(clippy::print_stdout)]

use anyhow::Result;
use theta_cli::SchemaArgs;
use theta_schema::ThetaManifest;

pub(crate) fn execute(_args: SchemaArgs) -> Result<()> {
    let schema = schemars::schema_for!(ThetaManifest);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
