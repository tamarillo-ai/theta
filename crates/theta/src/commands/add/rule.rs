//! `theta add rule` — register an instruction rule in the manifest.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_args::{AddRuleArgs, OutputFormat};
use theta_manifest::{ensure_table, parse_manifest, read_document, write_document};
use theta_schema::{ApplyMode, Validate};
use theta_settings::ThetaSettings;

use crate::commands::output::{
    EntityKind, MutationKind, MutationOutput, MutationSource, MutationSourceKind, present,
};
use crate::commands::{project_dir, report_diagnostics, require_manifest};

pub(super) fn execute(
    args: AddRuleArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
    settings: &ThetaSettings,
) -> Result<()> {
    require_manifest(manifest_path)?;

    if !theta_schema::is_valid_rule_name(&args.name) {
        bail!(
            "\"{}\" is not a valid rule name (kebab-case segments separated by `/`, no leading/trailing `/`)",
            args.name
        );
    }

    // --system: write a system store ref, no file scaffolding
    if let Some(ref store_name_arg) = args.system {
        // explicit store name, or derive from the leaf segment of the rule name
        let store_name = if store_name_arg.is_empty() {
            args.name.rsplit('/').next().unwrap_or(&args.name)
        } else {
            store_name_arg.as_str()
        };
        return add_rule_from_store(&args.name, store_name, output_format, manifest_path);
    }

    // --git: write a git ref, no file scaffolding
    if let Some(ref git) = args.git {
        let default_file = format!("{}.md", args.name);
        let file = args.file.as_deref().unwrap_or(&default_file);
        return add_rule_from_git(
            &args.name,
            git,
            args.branch.as_deref(),
            args.tag.as_deref(),
            args.rev.as_deref(),
            file,
            args.sync,
            output_format,
            manifest_path,
        );
    }

    let apply = args.apply;
    let project_dir = project_dir(manifest_path)?;

    let rule_path = match args.path {
        Some(ref p) => p.clone(),
        None => project_dir
            .join(settings.rules_path())
            .join(format!("{}.md", args.name)),
    };

    let rule_path_rel = theta_static::rel_string(&rule_path, project_dir);

    // single document read - used for all checks and mutation
    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if theta_manifest::has_rule(&doc, &args.name) {
        bail!(
            "rule \"{}\" is already registered in {}",
            args.name,
            manifest_path.display()
        );
    }

    let scaffolded = scaffold_rule_file(&args, &rule_path)?;

    let rules_table = ensure_table(&mut doc, &["instructions", "rules"]);

    let mut rule_table = toml_edit::Table::new();
    rule_table["src"] = toml_edit::value(&rule_path_rel);

    if let Some(ref summary) = args.summary {
        rule_table["summary"] = toml_edit::value(summary.as_str());
    }
    if let Some(ref desc) = args.description {
        rule_table["description"] = toml_edit::value(desc.as_str());
    }
    if apply != ApplyMode::Always {
        rule_table["apply"] = toml_edit::value(apply.as_str());
    }
    if let Some(ref patterns) = args.apply_to {
        let mut arr = toml_edit::Array::new();
        for p in patterns {
            arr.push(p.as_str());
        }
        rule_table["apply_to"] = toml_edit::value(arr);
    }

    rules_table[&args.name] = toml_edit::Item::Table(rule_table);

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;

    let mut diags = Vec::new();
    manifest.validate(&mut diags);

    let rule_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path == "[instructions.rules]")
        .collect();

    let (errors, _) = report_diagnostics(&rule_diags);
    if errors > 0 {
        if scaffolded {
            let _ = fs_err::remove_file(&rule_path);
        }
        bail!("rule rejected - manifest not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let files_written = if scaffolded {
        vec![rule_path.clone()]
    } else {
        vec![]
    };
    let outcome = MutationOutput {
        kind: MutationKind::Add,
        entity: EntityKind::Rule,
        name: Some(args.name.clone()),
        source: Some(MutationSource {
            kind: MutationSourceKind::Local,
            detail: rule_path_rel.clone(),
        }),
        files_written,
        files_deleted: vec![],
    };
    let path_provided = args.path.is_some();
    let name = args.name.clone();
    present(
        &["add", "rule"],
        output_format,
        outcome,
        vec![],
        move |_| {
            if path_provided {
                anstream::eprintln!(
                    "{} rule \"{}\" from {}",
                    "registered".green().bold(),
                    name.cyan(),
                    rule_path_rel.cyan(),
                );
            } else {
                anstream::eprintln!(
                    "{} {} - edit it to define the rule",
                    "created".green().bold(),
                    rule_path_rel.cyan(),
                );
            }
        },
    )
}

// create the rule file on disk if it doesn't exist. returns true if a file was created
fn scaffold_rule_file(args: &AddRuleArgs, rule_path: &Path) -> Result<bool> {
    if args.path.is_some() && rule_path.exists() {
        return Ok(false); // registering existing file
    }
    if rule_path.exists() {
        bail!(
            "file already exists: {} - use --path to register it instead",
            rule_path.display()
        );
    }
    if let Some(parent) = rule_path.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let content = args
        .content
        .as_deref()
        .unwrap_or(theta_static::DEFAULT_RULE_TEMPLATE);
    fs_err::write(rule_path, content)
        .with_context(|| format!("failed to write {}", rule_path.display()))?;
    Ok(true)
}

// write `[instructions.rules.<key>] src = { system = "<store_name>" }` to theta.toml
fn add_rule_from_store(
    key: &str,
    store_name: &str,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    // verify the store entry exists
    let store = theta_store::StoreHandle::open()?;
    if store.rule_path(store_name).is_none() {
        bail!(
            "rule '{store_name}' not found in the system store - run `theta register rule {store_name}` first"
        );
    }

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if theta_manifest::has_rule(&doc, key) {
        bail!(
            "rule \"{}\" is already registered in {}",
            key,
            manifest_path.display()
        );
    }

    let rules_table = ensure_table(&mut doc, &["instructions", "rules"]);

    let mut rule_table = toml_edit::Table::new();
    let mut src = toml_edit::InlineTable::new();
    src.insert("system", toml_edit::Value::from(store_name));
    rule_table["src"] = toml_edit::value(src);
    rules_table[key] = toml_edit::Item::Table(rule_table);

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let outcome = MutationOutput {
        kind: MutationKind::Add,
        entity: EntityKind::Rule,
        name: Some(key.to_string()),
        source: Some(MutationSource {
            kind: MutationSourceKind::Store,
            detail: store_name.to_string(),
        }),
        files_written: vec![],
        files_deleted: vec![],
    };
    let key_owned = key.to_string();
    let store_owned = store_name.to_string();
    present(
        &["add", "rule"],
        output_format,
        outcome,
        vec![],
        move |_| {
            if key_owned == store_owned {
                anstream::eprintln!(
                    "{} rule \"{}\" from system store",
                    "registered".green().bold(),
                    key_owned.cyan(),
                );
            } else {
                anstream::eprintln!(
                    "{} rule \"{}\" from system store (store: {})",
                    "registered".green().bold(),
                    key_owned.cyan(),
                    store_owned.cyan(),
                );
            }
        },
    )
}

fn add_rule_from_git(
    name: &str,
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    file: &str,
    sync: bool,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if theta_manifest::has_rule(&doc, name) {
        bail!(
            "rule \"{}\" is already registered in {}",
            name,
            manifest_path.display()
        );
    }

    let rules_table = ensure_table(&mut doc, &["instructions", "rules"]);

    let mut rule_table = toml_edit::Table::new();
    let mut src = toml_edit::InlineTable::new();
    src.insert("git", toml_edit::Value::from(git_url));
    if let Some(b) = branch {
        src.insert("branch", toml_edit::Value::from(b));
    }
    if let Some(t) = tag {
        src.insert("tag", toml_edit::Value::from(t));
    }
    if let Some(r) = rev {
        src.insert("rev", toml_edit::Value::from(r));
    }
    src.insert("file", toml_edit::Value::from(file));
    rule_table["src"] = toml_edit::value(src);
    rules_table[name] = toml_edit::Item::Table(rule_table);

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let outcome = MutationOutput {
        kind: MutationKind::Add,
        entity: EntityKind::Rule,
        name: Some(name.to_string()),
        source: Some(MutationSource {
            kind: MutationSourceKind::Git,
            detail: git_url.to_string(),
        }),
        files_written: vec![],
        files_deleted: vec![],
    };
    let name_owned = name.to_string();
    present(
        &["add", "rule"],
        output_format,
        outcome,
        vec![],
        move |_| {
            anstream::eprintln!(
                "{} rule \"{}\" from git",
                "registered".green().bold(),
                name_owned.cyan(),
            );
        },
    )?;

    if sync {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }

    Ok(())
}
