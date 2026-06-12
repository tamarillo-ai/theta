use owo_colors::OwoColorize;
use theta_schema::{DiagLevel, Diagnostic};

pub(crate) mod add;
pub(crate) mod cast;
pub(crate) mod check;
pub(crate) mod describe;
pub(crate) mod get;
pub(crate) mod init;
pub(crate) mod list;
pub(crate) mod lock;
pub(crate) mod migrate;
pub(crate) mod output;
pub(crate) mod register;
pub(crate) mod rm;
pub(crate) mod schema;
pub(crate) mod sync;
pub(crate) mod tree;

pub(crate) fn require_manifest(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("{} not found (run `theta init` first)", path.display());
    }
    Ok(())
}

pub(crate) fn project_dir(manifest_path: &std::path::Path) -> anyhow::Result<&std::path::Path> {
    manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("manifest path has no parent directory"))
}

// print diagnostics to stderr. returns (error_count, warning_count)
pub(crate) fn report_diagnostics(diags: &[Diagnostic]) -> (usize, usize) {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    for d in diags {
        match d.level {
            DiagLevel::Warn => {
                anstream::eprintln!("{} {} {}", "warn".yellow().bold(), d.path, d.message);
                warnings += 1;
            }
            DiagLevel::Error => {
                anstream::eprintln!("{} {} {}", "error".red().bold(), d.path, d.message);
                errors += 1;
            }
            DiagLevel::Hint => {
                anstream::eprintln!("{} {} {}", "hint".blue().bold(), d.path, d.message);
            }
            _ => {}
        }
    }
    (errors, warnings)
}
