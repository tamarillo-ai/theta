//! `theta add skill` — scaffold or register a skill in the manifest.

use std::path::Path;

use crate::commands::{project_dir, report_diagnostics, require_manifest};
use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_args::{AddSkillArgs, OutputFormat};
use theta_manifest::{ensure_table, parse_manifest, read_document, write_document};
use theta_schema::Validate;
use theta_static::is_default_manifest;

/// What the user intends to do with `theta add skill`.
enum SkillIntent {
    /// No source flags --> scaffold a new skill directory + register it.
    CreateAndRegister { name: String },
    /// --path --> validate an existing local dir + register it
    RegisterLocal {
        name: String,
        path: std::path::PathBuf,
    },
    /// --git (+ optional --branch/--tag/--rev / --subdirectory), or bare owner/repo syntax
    RegisterGit {
        name: String,
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
        subdirectory: Option<String>,
    },
    /// --system: write `source = { system = "<name>" }`, no scaffolding
    RegisterSystem { name: String },
}

impl SkillIntent {
    fn name(&self) -> &str {
        match self {
            Self::CreateAndRegister { name }
            | Self::RegisterLocal { name, .. }
            | Self::RegisterGit { name, .. }
            | Self::RegisterSystem { name } => name,
        }
    }
}

use crate::skill_resolve::parse_github_ref;

fn resolve_intent(args: &AddSkillArgs) -> Result<SkillIntent> {
    let effective_name =
        |fallback: &str| -> String { args.name.clone().unwrap_or_else(|| fallback.to_string()) };

    if args.system {
        return Ok(SkillIntent::RegisterSystem {
            name: effective_name(&args.name_or_ref),
        });
    }
    if let Some(ref git) = args.git {
        return Ok(SkillIntent::RegisterGit {
            name: effective_name(&args.name_or_ref),
            url: git.clone(),
            branch: args.branch.clone(),
            tag: args.tag.clone(),
            rev: args.rev.clone(),
            subdirectory: args.subdirectory.clone(),
        });
    }
    if let Some(ref path) = args.path {
        return Ok(SkillIntent::RegisterLocal {
            name: effective_name(&args.name_or_ref),
            path: path.clone(),
        });
    }
    // no flags --> check if name_or_ref contains '/' (github shorthand)
    // expands owner/repo[/path][@ref] into a full git URL
    if args.name_or_ref.contains('/') {
        let parsed = parse_github_ref(&args.name_or_ref)?;
        let name = effective_name(&parsed.inferred_name);
        return Ok(SkillIntent::RegisterGit {
            name,
            url: parsed.git_url(),
            branch: args.branch.clone(),
            tag: args.tag.clone(),
            rev: args.rev.clone().or(parsed.git_ref),
            subdirectory: args.subdirectory.clone().or(parsed.subdirectory),
        });
    }

    // --subdirectory without any git source is invalid
    if args.subdirectory.is_some() {
        bail!("--subdirectory requires --git or a github shorthand (owner/repo)");
    }

    // bare name --> scaffold
    Ok(SkillIntent::CreateAndRegister {
        name: effective_name(&args.name_or_ref),
    })
}

pub(super) fn execute(
    args: AddSkillArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

    let intent = resolve_intent(&args)?;
    let skill_name = intent.name().to_string();
    let wants_sync = !args.no_sync;

    if !theta_schema::is_valid_skill_name(&skill_name) {
        bail!(
            "\"{skill_name}\" is not a valid skill name (lowercase alphanumeric + hyphens, 1–64 chars, no leading/trailing/consecutive hyphens)"
        );
    }

    let project_dir = project_dir(manifest_path)?;

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if theta_manifest::has_skill(&doc, &skill_name) {
        bail!(
            "skill \"{}\" is already registered in {}",
            skill_name,
            manifest_path.display()
        );
    }

    if let SkillIntent::RegisterLocal { path, .. } = &intent {
        validate_local_skill_dir(path, &skill_name)?;
    }

    if let SkillIntent::RegisterSystem { name } = &intent {
        let store = theta_store::StoreHandle::open()?;
        if store.skill_path(name).is_none() {
            bail!(
                "skill '{name}' not found in the system store - run `theta register skill {name}` first"
            );
        }
    }

    let mut scaffolded = false;
    let skill_dir_rel = match &intent {
        SkillIntent::CreateAndRegister { name } => {
            let skill_dir = project_dir.join(theta_static::SKILLS_DIR).join(name);
            if skill_dir.exists() {
                bail!(
                    "directory already exists: {} - use --path to register it",
                    skill_dir.display()
                );
            }
            scaffold_skill_dir(&skill_dir, name, args.description.as_deref())?;
            scaffolded = true;
            format!("{}/{}", theta_static::SKILLS_DIR, name)
        }
        SkillIntent::RegisterLocal { path, .. } => theta_static::rel_string(path, project_dir),
        _ => String::new(), // remote sources don't use a local relative path
    };

    let skills_table = ensure_table(&mut doc, &["skills"]);
    let mut skill_table = toml_edit::Table::new();

    let source = match &intent {
        SkillIntent::RegisterGit {
            url,
            branch,
            tag,
            rev,
            subdirectory,
            ..
        } => {
            let mut s = toml_edit::InlineTable::new();
            s.insert("git", toml_edit::Value::from(url.as_str()));
            if let Some(b) = branch {
                s.insert("branch", toml_edit::Value::from(b.as_str()));
            }
            if let Some(t) = tag {
                s.insert("tag", toml_edit::Value::from(t.as_str()));
            }
            if let Some(r) = rev {
                s.insert("rev", toml_edit::Value::from(r.as_str()));
            }
            if let Some(d) = subdirectory {
                s.insert("subdirectory", toml_edit::Value::from(d.as_str()));
            }
            s
        }
        SkillIntent::CreateAndRegister { .. } | SkillIntent::RegisterLocal { .. } => {
            let mut s = toml_edit::InlineTable::new();
            s.insert("path", toml_edit::Value::from(skill_dir_rel.as_str()));
            s
        }
        SkillIntent::RegisterSystem { name } => {
            let mut s = toml_edit::InlineTable::new();
            s.insert("system", toml_edit::Value::from(name.as_str()));
            s
        }
    };
    skill_table["source"] = toml_edit::value(source);
    if let Some(ref tags) = args.tags {
        let arr = tags
            .iter()
            .map(|t| toml_edit::Value::from(t.as_str()))
            .collect::<toml_edit::Array>();
        skill_table["tags"] = toml_edit::value(arr);
    }
    if let Some(ref goal) = args.goal {
        skill_table["goal"] = toml_edit::value(goal.as_str());
    }
    skills_table[&skill_name] = toml_edit::Item::Table(skill_table);

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;

    let mut diags = Vec::new();
    manifest.validate(&mut diags);

    let skill_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path.contains("[skills"))
        .collect();

    let (errors, _) = report_diagnostics(&skill_diags);
    if errors > 0 {
        if scaffolded {
            let skill_dir = project_dir.join(theta_static::SKILLS_DIR).join(&skill_name);
            let _ = fs_err::remove_dir_all(&skill_dir);
        }
        bail!("skill rejected - manifest not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    if json {
        use crate::commands::output::{
            EntityKind, MutationKind, MutationOutput, MutationSource, MutationSourceKind,
        };
        use theta_schema::CommandOutput;
        let (source_kind, source_detail) = match &intent {
            SkillIntent::CreateAndRegister { .. } | SkillIntent::RegisterLocal { .. } => {
                (MutationSourceKind::Local, skill_dir_rel.clone())
            }
            SkillIntent::RegisterGit { url, .. } => (MutationSourceKind::Git, url.clone()),
            SkillIntent::RegisterSystem { name } => (MutationSourceKind::Store, name.clone()),
        };
        let files_written = if scaffolded {
            vec![project_dir.join(theta_static::SKILLS_DIR).join(&skill_name)]
        } else {
            vec![]
        };
        CommandOutput::ok(
            ["add", "skill"],
            MutationOutput {
                kind: MutationKind::Add,
                entity: EntityKind::Skill,
                name: Some(skill_name.clone()),
                source: Some(MutationSource {
                    kind: source_kind,
                    detail: source_detail,
                }),
                files_written,
                files_deleted: vec![],
            },
        )
        .print_json()?;
    } else {
        match &intent {
            SkillIntent::CreateAndRegister { .. } => {
                anstream::eprintln!(
                    "{} {} - edit {} to define the skill",
                    "created".green().bold(),
                    skill_dir_rel.cyan(),
                    format!("{}/{}", skill_dir_rel, theta_static::SKILL_FILE_NAME).cyan(),
                );
            }
            SkillIntent::RegisterLocal { .. } => {
                anstream::eprintln!(
                    "{} skill \"{}\" from {}",
                    "registered".green().bold(),
                    skill_name.cyan(),
                    skill_dir_rel.cyan(),
                );
            }
            _ => {
                let source_desc = match &intent {
                    SkillIntent::RegisterGit { .. } => "git",
                    SkillIntent::RegisterSystem { .. } => "system store",
                    _ => unreachable!(),
                };
                anstream::eprintln!(
                    "{} skill \"{}\" ({})",
                    "registered".green().bold(),
                    skill_name.cyan(),
                    source_desc,
                );
            }
        }
    }

    // pre-fetch git sources into cache so they're ready for sync/cast
    // fetch failure is not fatal at add time -- sync will catch it
    if !args.no_sync
        && let SkillIntent::RegisterGit {
            url,
            branch,
            tag,
            rev,
            subdirectory,
            ..
        } = &intent
    {
        let _ = crate::skill_resolve::fetch_git_checkout(
            url,
            branch.as_deref(),
            tag.as_deref(),
            rev.as_deref(),
            subdirectory.as_deref(),
        );
    }

    // sync (lock + materialize everything) unless --no-sync or non-root manifest
    // same as uv: `uv add` always syncs, `uv add --no-sync` opts out
    if wants_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }

    Ok(())
}

/// Validate that a local skill directory exists, is a directory, and contains a valid SKILL.md
fn validate_local_skill_dir(path: &Path, expected_name: &str) -> Result<()> {
    if !path.exists() {
        bail!("skill directory does not exist: {}", path.display());
    }
    if !path.is_dir() {
        bail!("skill path is not a directory: {}", path.display());
    }
    let skill_md = path.join(theta_static::SKILL_FILE_NAME);
    if !skill_md.exists() {
        bail!(
            "{} not found in {} - a valid skill directory must contain {}",
            theta_static::SKILL_FILE_NAME,
            path.display(),
            theta_static::SKILL_FILE_NAME,
        );
    }
    let content = fs_err::read_to_string(&skill_md)
        .with_context(|| format!("failed to read {}", skill_md.display()))?;
    validate_skill_frontmatter(&content, expected_name)?;
    Ok(())
}

fn scaffold_skill_dir(skill_dir: &Path, name: &str, description: Option<&str>) -> Result<()> {
    fs_err::create_dir_all(skill_dir)
        .with_context(|| format!("failed to create {}", skill_dir.display()))?;

    // create SKILL.md
    let description = description.unwrap_or(theta_static::DEFAULT_SKILL_DESCRIPTION);
    let content = theta_static::skill_template(name, description);
    fs_err::write(skill_dir.join(theta_static::SKILL_FILE_NAME), content)
        .with_context(|| format!("failed to write {}", theta_static::SKILL_FILE_NAME))?;

    // create empty subdirectories
    for subdir in &["scripts", "references", "assets"] {
        fs_err::create_dir_all(skill_dir.join(subdir))
            .with_context(|| format!("failed to create {subdir} directory"))?;
    }

    Ok(())
}

fn validate_skill_frontmatter(content: &str, expected_name: &str) -> Result<()> {
    let Some(yaml_str) = theta_static::split_frontmatter(content).0 else {
        bail!(
            "{} missing YAML frontmatter (must start with ---)",
            theta_static::SKILL_FILE_NAME
        );
    };

    let fm = theta_schema::SkillFrontmatter::parse(yaml_str).with_context(|| {
        format!(
            "{} frontmatter is not valid YAML",
            theta_static::SKILL_FILE_NAME
        )
    })?;

    match fm.name.as_deref() {
        Some(n) if n != expected_name => {
            bail!(
                "{} frontmatter name \"{}\" does not match skill name \"{}\"",
                theta_static::SKILL_FILE_NAME,
                n,
                expected_name,
            );
        }
        None => {
            bail!(
                "{} frontmatter is missing required `name` field",
                theta_static::SKILL_FILE_NAME,
            );
        }
        _ => {}
    }
    match fm.description.as_deref() {
        Some("") => {
            bail!(
                "{} frontmatter description is empty",
                theta_static::SKILL_FILE_NAME
            );
        }
        None => {
            bail!(
                "{} frontmatter is missing required `description` field",
                theta_static::SKILL_FILE_NAME,
            );
        }
        _ => {}
    }

    Ok(())
}
