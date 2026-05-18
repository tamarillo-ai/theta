//! theta CLI — entry point and command dispatch.

use anyhow::{Context, Result};
use clap::Parser;

use theta_cli::{Cli, Commands};
use theta_settings::{CliOverrides, ThetaSettings};
use theta_static::MANIFEST_FILE_NAME;

mod commands;
mod resolve;
mod skill_resolve;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref dir) = cli.global.directory {
        std::env::set_current_dir(dir)
            .with_context(|| format!("failed to change directory to {}", dir.display()))?;
    }

    let manifest_path = match cli.global.manifest {
        Some(p) => p,
        None => find_manifest().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join(MANIFEST_FILE_NAME)
        }),
    };

    let overrides = CliOverrides {
        instructions_dir: cli.global.instructions_dir,
        rules_dir: cli.global.rules_dir,
    };
    let settings = ThetaSettings::resolve(&overrides);

    match cli.command {
        Commands::Init(args) => commands::init::execute(args, &manifest_path),
        Commands::Check(args) => commands::check::execute(args, &manifest_path),
        Commands::Migrate(args) => commands::migrate::execute(args, &manifest_path),
        Commands::Describe(args) => commands::describe::execute(args, &manifest_path),
        Commands::Add(ns) => commands::add::dispatch(ns, &manifest_path, &settings),
        Commands::Rm(ns) => commands::rm::dispatch(ns, &manifest_path),
        Commands::List(ns) => commands::list::execute(ns, &manifest_path),
        Commands::Lock(args) => commands::lock::execute(args, &manifest_path),
        Commands::Sync(args) => commands::sync::execute(args, &manifest_path),
        Commands::Cast(ns) => commands::cast::dispatch(ns, &manifest_path),
        Commands::Register(ns) => commands::register::dispatch(ns, &manifest_path),
        Commands::Tree(args) => commands::tree::execute(args, &manifest_path),
        Commands::Schema(args) => commands::schema::execute(args),
    }
}

/// Walk up from the current directory looking for `theta.toml`.
fn find_manifest() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(MANIFEST_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
