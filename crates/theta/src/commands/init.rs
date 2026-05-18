//! `theta init` — create and set defaults on a new `theta.toml`.

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_cli::InitArgs;
use theta_manifest::create_manifest;
use theta_schema::{minimal_manifest, normalize_agent_name};
use theta_static::MANIFEST_FILE_NAME;
use theta_store::StoreHandle;

pub(crate) fn execute(args: InitArgs, manifest_path: &Path) -> Result<()> {
    let cwd = super::project_dir(manifest_path)?;

    if let Some(ref agent_name) = args.from {
        return init_from_store(agent_name, manifest_path, args.force);
    }

    if manifest_path.exists() {
        bail!("{} already exists in {}", MANIFEST_FILE_NAME, cwd.display());
    }

    let name = args.name.unwrap_or_else(|| name_from_directory(cwd));

    let mut manifest = minimal_manifest(&name);
    manifest.agent.authors = detect_authors();

    create_manifest(manifest_path, &manifest)
        .with_context(|| format!("failed to create {}", manifest_path.display()))?;

    append_gitignore(cwd);

    anstream::eprintln!(
        "{} {} in {}",
        "initialized".green().bold(),
        MANIFEST_FILE_NAME.cyan(),
        cwd.display().cyan()
    );

    Ok(())
}

fn init_from_store(agent_name: &str, manifest_path: &Path, force: bool) -> Result<()> {
    let target_dir = super::project_dir(manifest_path)?;
    let store = StoreHandle::open()?;

    store.init_from_agent(agent_name, target_dir, manifest_path, force)?;

    anstream::eprintln!(
        "{} from '{}' into {}",
        "initialized".green().bold(),
        agent_name.cyan(),
        target_dir.display().cyan()
    );

    Ok(())
}

fn detect_authors() -> Option<Vec<String>> {
    let name = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8(o.stdout).ok()?;
            let s = s.trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

    let email = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8(o.stdout).ok()?;
            let s = s.trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

    match (name, email) {
        (Some(n), Some(e)) => Some(vec![format!("{n} <{e}>")]),
        (Some(n), None) => Some(vec![n]),
        _ => None,
    }
}

fn name_from_directory(path: &std::path::Path) -> String {
    let raw = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("my-agent");
    normalize_agent_name(raw)
}

// best-effort: append .theta/ to .gitignore if present and not already listed.
// modeled after cargo's write_ignore_file: read --> dedup --> append.
// see: https://github.com/rust-lang/cargo/blob/bf2f4a4a56d6b857eca7657839b51a90d50e20ae/src/cargo/ops/cargo_new.rs#L709
fn append_gitignore(project_dir: &Path) {
    let path = project_dir.join(".gitignore");
    let Ok(existing) = fs::read_to_string(&path) else {
        return;
    };

    let dominated = |l: &str| matches!(l.trim(), ".theta" | ".theta/" | "/.theta" | "/.theta/");
    if existing.lines().any(dominated) {
        return;
    }

    let Ok(mut f) = fs::OpenOptions::new().append(true).open(&path) else {
        return;
    };
    if !existing.ends_with('\n') {
        let _ = writeln!(f);
    }
    if writeln!(f, ".theta/").is_ok() {
        anstream::eprintln!(
            "{} .theta/ to {}",
            "appended".green().bold(),
            ".gitignore".cyan(),
        );
    }
}
