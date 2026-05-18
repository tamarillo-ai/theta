//! `theta cast` — cast `theta.toml` to/from harness-native config.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_cast::{CastFile, cast_notes, caster_for, import_notes, importer_for, write_cast_output};
use theta_cli::{CastCommand, CastFromArgs, CastNamespace, CastToArgs};
use theta_manifest::read_manifest;
use theta_schema::DiagLevel;
use theta_static::{DOT_THETA_DIR, MANIFEST_FILE_NAME};

pub(crate) fn dispatch(ns: CastNamespace, manifest_path: &Path) -> Result<()> {
    match ns.command {
        CastCommand::To(args) => execute_to(args, manifest_path),
        CastCommand::From(args) => execute_from(args, manifest_path),
    }
}

fn execute_to(args: CastToArgs, manifest_path: &Path) -> Result<()> {
    let target = args.target;

    if args.notes {
        anstream::print!("{}", cast_notes(target));
        return Ok(());
    }

    super::require_manifest(manifest_path)?;

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let project_dir = manifest_path.parent().unwrap();

    // auto-sync: ensures that .theta/ is materialized and up to date
    super::sync::execute(theta_cli::SyncArgs { force: false }, manifest_path)?;

    let theta_dir = project_dir.join(DOT_THETA_DIR);
    let output_dir = args.output.unwrap_or_else(|| project_dir.to_path_buf());

    guard_user_home_output(&output_dir, args.force)?;

    let caster = caster_for(target);

    let config_diags = caster.validate_config(&manifest);
    for d in &config_diags {
        match d.level {
            DiagLevel::Error => {
                anstream::eprintln!("{} {} {}", "error".red().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Warn => {
                anstream::eprintln!("{} {} {}", "warn".yellow().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Hint => {
                anstream::eprintln!("{} {} {}", "hint".blue().bold(), d.path.cyan(), d.message);
            }
            _ => {}
        }
    }

    // merge with on-disk files at `output_dir` when the harness shares config
    // files with the editor (e.g. `.vscode/settings.json`, `.vscode/mcp.json`);
    // the default trait impl delegates to `cast_files` so from-scratch harnesses
    // are unaffected
    let files = caster
        .cast_files_with_output(&manifest, &theta_dir, &output_dir)
        .with_context(|| format!("failed to cast to {target}"))?;

    if !args.force {
        let existing: Vec<_> = files
            .iter()
            .map(|(rel, _)| output_dir.join(rel))
            .filter(|p| p.exists())
            .collect();
        if !existing.is_empty() {
            for p in &existing {
                anstream::eprintln!(
                    "{} {} already exists",
                    "conflict".red().bold(),
                    p.display().cyan(),
                );
            }
            bail!(
                "cast would overwrite {} existing file(s) - use --force to overwrite",
                existing.len()
            );
        }
    }

    let diags = caster.validate_output(&files);
    for d in &diags {
        match d.level {
            DiagLevel::Error => {
                anstream::eprintln!("{} {} {}", "error".red().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Warn => {
                anstream::eprintln!("{} {} {}", "warn".yellow().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Hint => {
                anstream::eprintln!("{} {} {}", "hint".blue().bold(), d.path.cyan(), d.message);
            }
            _ => {}
        }
    }

    let written = write_cast_output(&files, &output_dir)
        .with_context(|| format!("failed to write cast output to {}", output_dir.display()))?;

    for path in &written {
        anstream::eprintln!("{} {}", "wrote".green().bold(), path.display().cyan());
    }

    warn_coexisting_hook_files(&output_dir, &files);

    anstream::eprintln!(
        "{} cast to {} ({} file(s))",
        "done".green().bold(),
        target.as_str().cyan(),
        written.len(),
    );
    anstream::eprintln!(
        "{} round-trip has known YAML/JSON cosmetic differences - run `theta cast to {} --notes` for details",
        "info".blue().bold(),
        target.as_str(),
    );

    Ok(())
}

/// Refuse to cast into a known user-level harness directory under `$HOME`.
/// The set of reserved directories is enumerated from `HarnessTarget::all()`
/// via `user_home_dir()`, so adding a new harness automatically extends the
/// guard. Hand-curated user-level config must not be silently overwritten by
/// project-scoped `theta cast`. `--force` bypasses the guard.
fn guard_user_home_output(output_dir: &Path, force: bool) -> Result<()> {
    let Some(home) = theta_dirs::home_dir() else {
        return Ok(());
    };
    let canonical = output_dir
        .canonicalize()
        .unwrap_or_else(|_| output_dir.to_path_buf());
    let home_canonical = home.canonicalize().unwrap_or(home);
    for target in theta_harness::HarnessTarget::all() {
        let reserved = home_canonical.join(target.user_home_dir());
        if canonical == reserved || canonical.starts_with(&reserved) {
            if force {
                anstream::eprintln!(
                    "{} cast output {} lands under a user-level harness directory ({}); --force in effect",
                    "warn".yellow().bold(),
                    canonical.display().cyan(),
                    target.as_str().cyan(),
                );
                return Ok(());
            }
            bail!(
                "refusing to cast into user-level {} directory {} - this would overwrite hand-curated global config; pass --force to override",
                target.as_str(),
                canonical.display()
            );
        }
    }
    Ok(())
}

/// Warn if `theta-hooks.json` coexists with other hook JSON files.
/// VS Code merges all `*.json` in `.github/hooks/`, so duplicates cause hooks to fire twice.
fn warn_coexisting_hook_files(output_dir: &Path, files: &[CastFile]) {
    let hooks_dir = output_dir.join(".github/hooks");
    let wrote_hooks = files
        .iter()
        .any(|(p, _): &CastFile| p.starts_with(".github/hooks/"));
    if !wrote_hooks || !hooks_dir.is_dir() {
        return;
    }
    let Ok(rd) = fs_err::read_dir(&hooks_dir) else {
        return;
    };
    let other_jsons: Vec<_> = rd
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let p = e.path();
            p.extension().is_some_and(|ext| ext == "json") && e.file_name() != "theta-hooks.json"
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    if !other_jsons.is_empty() {
        anstream::eprintln!(
            "{} .github/hooks/ contains {} alongside theta-hooks.json - VS Code merges all *.json files, so hooks may fire twice. consider removing the originals after importing into theta.",
            "warn".yellow().bold(),
            other_jsons.join(", ").cyan(),
        );
    }
}

fn execute_from(args: CastFromArgs, manifest_path: &Path) -> Result<()> {
    let target = args.source;

    if args.notes {
        anstream::print!("{}", import_notes(target));
        return Ok(());
    }

    // cast-from creates a NEW manifest - use cwd as default, not the
    // walked-up manifest path (which may point to an ancestor project)
    // Only honor manifest_path when --manifest was explicitly provided
    let cwd = std::env::current_dir().unwrap_or_default();
    let cwd_default = cwd.join(MANIFEST_FILE_NAME);
    let has_explicit_manifest =
        manifest_path != cwd_default && manifest_path != Path::new(MANIFEST_FILE_NAME);

    let project_dir = args.input.unwrap_or_else(|| cwd.clone());

    let theta_toml = if has_explicit_manifest {
        manifest_path.to_path_buf()
    } else {
        project_dir.join(MANIFEST_FILE_NAME)
    };

    if theta_toml.exists() && !args.force {
        bail!(
            "{} already exists - use --force to overwrite",
            theta_toml.display()
        );
    }

    let import_opts = theta_cast::ImportOptions::new(
        &project_dir,
        args.subagent_prompts.clone(),
        args.force_overwrite,
    )
    .with_cross_read(args.cross_read);
    let importer = importer_for(target);
    let result = importer
        .import(&project_dir, &import_opts)
        .with_context(|| format!("failed to import from {target}"))?;

    for d in &result.diagnostics {
        match d.level {
            DiagLevel::Error => {
                anstream::eprintln!("{} {} {}", "error".red().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Warn => {
                anstream::eprintln!("{} {} {}", "warn".yellow().bold(), d.path.cyan(), d.message);
            }
            DiagLevel::Hint => {
                anstream::eprintln!("{} {} {}", "hint".blue().bold(), d.path.cyan(), d.message);
            }
            _ => {}
        }
    }

    if !args.force {
        let conflicts: Vec<_> = result
            .extracted_files
            .iter()
            .map(|(rel, _)| project_dir.join(rel))
            .filter(|p| p.exists())
            .collect();
        if !conflicts.is_empty() {
            for p in &conflicts {
                anstream::eprintln!(
                    "{} {} already exists",
                    "conflict".red().bold(),
                    p.display().cyan(),
                );
            }
            bail!(
                "import would overwrite {} existing file(s) - use --force to overwrite",
                conflicts.len()
            );
        }
    }

    // write extracted source files (system.md, rules/*.md, etc.)
    let mut written_count = 0usize;
    for (rel_path, content) in &result.extracted_files {
        let dst = project_dir.join(rel_path);
        if let Some(parent) = dst.parent() {
            fs_err::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let bytes = match content {
            theta_cast::CastContent::Text(s) => {
                if s.ends_with('\n') {
                    s.as_bytes().to_vec()
                } else {
                    format!("{s}\n").into_bytes()
                }
            }
            theta_cast::CastContent::Binary(b) => b.clone(),
        };
        fs_err::write(&dst, bytes).with_context(|| format!("failed to write {}", dst.display()))?;
        anstream::eprintln!("{} {}", "wrote".green().bold(), dst.display().cyan());
        written_count += 1;
    }

    let toml_content = result.document.to_string();
    if let Some(parent) = theta_toml.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs_err::write(&theta_toml, &toml_content)
        .with_context(|| format!("failed to write {}", theta_toml.display()))?;
    anstream::eprintln!("{} {}", "wrote".green().bold(), theta_toml.display().cyan());
    written_count += 1;

    for src in &result.sources_read {
        anstream::eprintln!(
            "{} {}",
            "read".blue().bold(),
            project_dir.join(src).display().cyan()
        );
    }

    anstream::eprintln!(
        "{} imported from {} ({} file(s) written)",
        "done".green().bold(),
        target.as_str().cyan(),
        written_count,
    );
    anstream::eprintln!(
        "{} round-trip has known YAML/JSON cosmetic differences - run `theta cast from {} --notes` for details",
        "info".blue().bold(),
        target.as_str(),
    );

    Ok(())
}
