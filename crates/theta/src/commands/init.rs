//! `theta init` — create and set defaults on a new `theta.toml`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{InitArgs, OutputFormat};
use theta_manifest::create_manifest;
use theta_schema::{Diagnostic, minimal_manifest, normalize_agent_name};
use theta_static::MANIFEST_FILE_NAME;
use theta_store::StoreHandle;

use super::output::{present, present_error};

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InitSource {
    Scaffold,
    Store,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct InitOutput {
    pub manifest_path: PathBuf,
    pub agent_name: Option<String>,
    pub source: InitSource,
    pub gitignore_appended: bool,
}

pub(crate) fn execute(
    args: InitArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    let cwd = super::project_dir(manifest_path)?;

    if let Some(ref agent_name) = args.from {
        return init_from_store(agent_name, output_format, manifest_path, args.force);
    }

    if manifest_path.exists() {
        let outcome = InitOutput {
            manifest_path: manifest_path.to_path_buf(),
            agent_name: None,
            source: InitSource::Scaffold,
            gitignore_appended: false,
        };
        let diag = Diagnostic::error(
            "[manifest]",
            format!("{} already exists", manifest_path.display()),
        );
        return present_error(
            &["init"],
            output_format,
            outcome,
            vec![diag],
            |_| {},
            anyhow!("{} already exists in {}", MANIFEST_FILE_NAME, cwd.display()),
        );
    }

    let name = args.name.unwrap_or_else(|| name_from_directory(cwd));

    let mut manifest = minimal_manifest(&name);
    manifest.agent.authors = detect_authors();

    create_manifest(manifest_path, &manifest)
        .with_context(|| format!("failed to create {}", manifest_path.display()))?;

    let gitignore_appended = append_gitignore(cwd);

    let outcome = InitOutput {
        manifest_path: manifest_path.to_path_buf(),
        agent_name: Some(name),
        source: InitSource::Scaffold,
        gitignore_appended,
    };

    present(&["init"], output_format, outcome, vec![], |o| {
        anstream::eprintln!(
            "{} {} in {}",
            "initialized".green().bold(),
            MANIFEST_FILE_NAME.cyan(),
            o.manifest_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
                .cyan()
        );
        if o.gitignore_appended {
            anstream::eprintln!(
                "{} .theta/ to {}",
                "appended".green().bold(),
                ".gitignore".cyan(),
            );
        }
    })
}

fn init_from_store(
    agent_name: &str,
    output_format: OutputFormat,
    manifest_path: &Path,
    force: bool,
) -> Result<()> {
    let target_dir = super::project_dir(manifest_path)?;
    let store = StoreHandle::open()?;

    store.init_from_agent(agent_name, target_dir, manifest_path, force)?;

    let outcome = InitOutput {
        manifest_path: manifest_path.to_path_buf(),
        agent_name: Some(agent_name.to_string()),
        source: InitSource::Store,
        gitignore_appended: false,
    };

    present(&["init"], output_format, outcome, vec![], |o| {
        anstream::eprintln!(
            "{} from '{}' into {}",
            "initialized".green().bold(),
            o.agent_name.as_deref().unwrap_or("").cyan(),
            o.manifest_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
                .cyan()
        );
    })
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
fn append_gitignore(project_dir: &Path) -> bool {
    let path = project_dir.join(".gitignore");
    let Ok(existing) = fs::read_to_string(&path) else {
        return false;
    };

    let dominated = |l: &str| matches!(l.trim(), ".theta" | ".theta/" | "/.theta" | "/.theta/");
    if existing.lines().any(dominated) {
        return false;
    }

    let Ok(mut f) = fs::OpenOptions::new().append(true).open(&path) else {
        return false;
    };
    if !existing.ends_with('\n') {
        let _ = writeln!(f);
    }
    writeln!(f, ".theta/").is_ok()
}
