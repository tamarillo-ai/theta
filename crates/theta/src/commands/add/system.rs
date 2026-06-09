//! `theta add system` — set the system prompt path in the manifest.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_args::{AddSystemArgs, OutputFormat};
use theta_manifest::{ensure_table, parse_manifest, read_document, write_document};
use theta_schema::Validate;
use theta_settings::ThetaSettings;

use crate::commands::output::{
    EntityKind, MutationKind, MutationOutput, MutationSource, MutationSourceKind, present,
};
use crate::commands::{project_dir, report_diagnostics, require_manifest};

pub(super) fn execute(
    args: AddSystemArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
    settings: &ThetaSettings,
) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    // bail if [instructions].system is already set
    let has_system = doc
        .get("instructions")
        .and_then(|i| i.get("system"))
        .is_some();
    if has_system {
        bail!(
            "[instructions].system is already set in {} - edit it directly",
            manifest_path.display()
        );
    }

    let system_path = match args.path {
        Some(ref p) => p.clone(),
        None => project_dir
            .join(&settings.instructions_dir)
            .join(theta_static::SYSTEM_FILE_NAME),
    };

    let system_path_rel = theta_static::rel_string(&system_path, project_dir);

    let scaffolded = scaffold_system_file(&args, &system_path)?;

    let instructions = ensure_table(&mut doc, &["instructions"]);
    instructions.insert("system", toml_edit::value(&system_path_rel));

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;

    let mut diags = Vec::new();
    manifest.validate(&mut diags);

    let system_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path == "[instructions].system")
        .collect();

    let (errors, _) = report_diagnostics(&system_diags);
    if errors > 0 {
        if scaffolded {
            let _ = fs_err::remove_file(&system_path);
        }
        bail!("system prompt rejected - manifest not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let files_written = if scaffolded {
        vec![system_path.clone()]
    } else {
        vec![]
    };
    let outcome = MutationOutput {
        kind: MutationKind::Add,
        entity: EntityKind::System,
        name: None,
        source: Some(MutationSource {
            kind: MutationSourceKind::Local,
            detail: system_path_rel.clone(),
        }),
        files_written,
        files_deleted: vec![],
    };
    let registered = args.path.is_some();
    present(
        &["add", "system"],
        output_format,
        outcome,
        vec![],
        move |_| {
            if registered {
                anstream::eprintln!(
                    "{} system prompt from {}",
                    "registered".green().bold(),
                    system_path_rel.cyan(),
                );
            } else {
                anstream::eprintln!(
                    "{} {} - edit it to set your system prompt",
                    "created".green().bold(),
                    system_path_rel.cyan(),
                );
            }
        },
    )
}

fn scaffold_system_file(args: &AddSystemArgs, system_path: &Path) -> Result<bool> {
    if args.path.is_some() && system_path.exists() {
        return Ok(false); // registering existing file
    }
    if system_path.exists() {
        bail!(
            "file already exists: {} - use --path to register it instead",
            system_path.display()
        );
    }
    if let Some(parent) = system_path.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let content = args
        .content
        .as_deref()
        .unwrap_or(theta_static::DEFAULT_SYSTEM_TEMPLATE);
    fs_err::write(system_path, content)
        .with_context(|| format!("failed to write {}", system_path.display()))?;
    Ok(true)
}
