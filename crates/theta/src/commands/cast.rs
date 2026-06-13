//! `theta cast` — cast `theta.toml` to/from harness-native config.

use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use std::path::PathBuf;
use theta_args::{CastCommand, CastFromArgs, CastNamespace, CastToArgs, OutputFormat};
use theta_cast::{CastFile, cast_notes, caster_for, import_notes, importer_for, write_cast_output};
use theta_manifest::read_manifest;
use theta_schema::{DiagLevel, Diagnostic};
use theta_static::{DOT_THETA_DIR, MANIFEST_FILE_NAME};

use super::output::{present, present_error};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct CastToOutput {
    pub target: String,
    pub output_dir: PathBuf,
    pub files_written: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct CastFromOutput {
    pub source: String,
    pub manifest_path: PathBuf,
    pub files_written: Vec<PathBuf>,
    pub sources_read: Vec<PathBuf>,
}

fn render_diagnostic(d: &Diagnostic) {
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

pub(crate) fn dispatch(
    ns: CastNamespace,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    match ns.command {
        CastCommand::To(args) => execute_to(args, output_format, manifest_path),
        CastCommand::From(args) => execute_from(args, output_format, manifest_path),
    }
}

fn execute_to(args: CastToArgs, output_format: OutputFormat, manifest_path: &Path) -> Result<()> {
    let target = args.target;

    if args.notes {
        let notes = cast_notes(target);
        let outcome = CastToOutput {
            target: target.as_str().to_string(),
            output_dir: PathBuf::new(),
            files_written: vec![],
        };
        return present(&["cast", "to"], output_format, outcome, vec![], move |_| {
            anstream::print!("{notes}");
        });
    }

    super::require_manifest(manifest_path)?;

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let project_dir = manifest_path.parent().unwrap();

    // auto-sync: ensures that .theta/ is materialized and up to date
    super::sync::execute(
        theta_args::SyncArgs { force: false },
        OutputFormat::Human,
        manifest_path,
    )?;

    let theta_dir = project_dir.join(DOT_THETA_DIR);
    let output_dir = args.output.unwrap_or_else(|| project_dir.to_path_buf());

    guard_user_home_output(&output_dir, args.force)?;

    let caster = caster_for(target);

    let config_diags = caster.validate_config(&manifest);

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
            let diags: Vec<Diagnostic> = existing
                .iter()
                .map(|p| Diagnostic::error("[cast.to]", format!("{} already exists", p.display())))
                .collect();
            let n = existing.len();
            let outcome = CastToOutput {
                target: target.as_str().to_string(),
                output_dir: output_dir.clone(),
                files_written: vec![],
            };
            let existing_for_render = existing;
            return present_error(
                &["cast", "to"],
                output_format,
                outcome,
                diags,
                move |_| {
                    for p in &existing_for_render {
                        anstream::eprintln!(
                            "{} {} already exists",
                            "conflict".red().bold(),
                            p.display().cyan(),
                        );
                    }
                },
                anyhow!("cast would overwrite {n} existing file(s) - use --force to overwrite"),
            );
        }
    }

    let output_diags = caster.validate_output(&files);
    let written = write_cast_output(&files, &output_dir)
        .with_context(|| format!("failed to write cast output to {}", output_dir.display()))?;

    let mut all_diags = config_diags;
    all_diags.extend(output_diags);

    let outcome = CastToOutput {
        target: target.as_str().to_string(),
        output_dir: output_dir.clone(),
        files_written: written,
    };
    let target_str = target.as_str().to_string();
    let files_for_render = files;
    let output_dir_for_render = output_dir;
    let diags_for_render = all_diags.clone();
    present(
        &["cast", "to"],
        output_format,
        outcome,
        all_diags,
        move |o| {
            for d in &diags_for_render {
                render_diagnostic(d);
            }
            for path in &o.files_written {
                anstream::eprintln!("{} {}", "wrote".green().bold(), path.display().cyan());
            }
            warn_coexisting_hook_files(&output_dir_for_render, &files_for_render);
            anstream::eprintln!(
                "{} cast to {} ({} file(s))",
                "done".green().bold(),
                target_str.cyan(),
                o.files_written.len(),
            );
            anstream::eprintln!(
                "{} round-trip has known YAML/JSON cosmetic differences - run `theta cast to {} --notes` for details",
                "info".blue().bold(),
                target_str,
            );
        },
    )
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

fn execute_from(
    args: CastFromArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    let target = args.source;

    if args.notes {
        let notes = import_notes(target);
        let outcome = CastFromOutput {
            source: target.as_str().to_string(),
            manifest_path: PathBuf::new(),
            files_written: vec![],
            sources_read: vec![],
        };
        return present(
            &["cast", "from"],
            output_format,
            outcome,
            vec![],
            move |_| anstream::print!("{notes}"),
        );
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

    if !args.force {
        let conflicts: Vec<_> = result
            .extracted_files
            .iter()
            .map(|(rel, _)| project_dir.join(rel))
            .filter(|p| p.exists())
            .collect();
        if !conflicts.is_empty() {
            let mut diags: Vec<Diagnostic> = result.diagnostics.clone();
            for p in &conflicts {
                diags.push(Diagnostic::error(
                    "[cast.from]",
                    format!("{} already exists", p.display()),
                ));
            }
            let n = conflicts.len();
            let outcome = CastFromOutput {
                source: target.as_str().to_string(),
                manifest_path: theta_toml.clone(),
                files_written: vec![],
                sources_read: vec![],
            };
            let import_diags = result.diagnostics.clone();
            return present_error(
                &["cast", "from"],
                output_format,
                outcome,
                diags,
                move |_| {
                    for d in &import_diags {
                        render_diagnostic(d);
                    }
                    for p in &conflicts {
                        anstream::eprintln!(
                            "{} {} already exists",
                            "conflict".red().bold(),
                            p.display().cyan(),
                        );
                    }
                },
                anyhow!("import would overwrite {n} existing file(s) - use --force to overwrite"),
            );
        }
    }

    // write extracted source files (system.md, rules/*.md, etc.)
    let mut files_written: Vec<PathBuf> = Vec::new();
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
        files_written.push(dst);
    }

    let toml_content = result.document.to_string();
    if let Some(parent) = theta_toml.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs_err::write(&theta_toml, &toml_content)
        .with_context(|| format!("failed to write {}", theta_toml.display()))?;
    files_written.push(theta_toml.clone());

    let sources_read: Vec<PathBuf> = result
        .sources_read
        .iter()
        .map(|p| project_dir.join(p))
        .collect();
    let outcome = CastFromOutput {
        source: target.as_str().to_string(),
        manifest_path: theta_toml,
        files_written,
        sources_read,
    };
    let target_str = target.as_str().to_string();
    let diags_for_render = result.diagnostics.clone();
    present(
        &["cast", "from"],
        output_format,
        outcome,
        result.diagnostics,
        move |o| {
            for d in &diags_for_render {
                render_diagnostic(d);
            }
            for path in &o.files_written {
                anstream::eprintln!("{} {}", "wrote".green().bold(), path.display().cyan());
            }
            for src in &o.sources_read {
                anstream::eprintln!("{} {}", "read".blue().bold(), src.display().cyan());
            }
            anstream::eprintln!(
                "{} imported from {} ({} file(s) written)",
                "done".green().bold(),
                target_str.cyan(),
                o.files_written.len(),
            );
            anstream::eprintln!(
                "{} round-trip has known YAML/JSON cosmetic differences - run `theta cast from {} --notes` for details",
                "info".blue().bold(),
                target_str,
            );
        },
    )
}
