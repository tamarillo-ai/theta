//! `theta register` — register a project resource into the system store.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_cli::{
    LockArgs, RegisterAgentArgs, RegisterCommand, RegisterNamespace, RegisterRuleArgs,
    RegisterSkillArgs,
};
use theta_manifest::read_manifest;
use theta_schema::{LocalOrGitRef, SourceRef};
use theta_store::StoreHandle;

use crate::skill_resolve::{
    ResolvedSkill, fetch_git_checkout, parse_github_ref, read_skill_description,
};

pub(crate) fn dispatch(ns: RegisterNamespace, manifest_path: &Path) -> Result<()> {
    let store = StoreHandle::open()?;

    match ns.command {
        RegisterCommand::Skill(args) => register_skill(args, manifest_path, &store),
        RegisterCommand::Rule(args) => register_rule(args, manifest_path, &store),
        RegisterCommand::Agent(args) => register_agent(args, manifest_path, &store),
    }
}

fn resolve_manifest_skill(name: &str, manifest_path: &Path) -> Result<ResolvedSkill> {
    super::require_manifest(manifest_path)?;
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let project_dir = super::project_dir(manifest_path)?;

    let skills = manifest
        .skills
        .as_ref()
        .with_context(|| "no skills defined in theta.toml")?;
    let skill = skills
        .get(name)
        .with_context(|| format!("skill '{name}' not found in [skills]"))?;

    let skill_dir = match &skill.source {
        SourceRef::Path { path } => project_dir.join(path),
        SourceRef::Git {
            git,
            branch,
            tag,
            rev,
            subdirectory,
            ..
        } => fetch_git_checkout(
            git,
            branch.as_deref(),
            tag.as_deref(),
            rev.as_deref(),
            subdirectory.as_deref(),
        )?,
        SourceRef::System { .. } => {
            bail!("skill '{name}' is already from the system store")
        }
        _ => bail!("skill '{name}' has an unsupported source type"),
    };

    let description = read_skill_description(&skill_dir);
    Ok(ResolvedSkill {
        name: name.to_string(),
        description,
        dir: skill_dir,
    })
}

fn resolve_standalone_skill(args: &RegisterSkillArgs) -> Result<ResolvedSkill> {
    let name = args.name.as_deref().unwrap_or(&args.name_or_ref);

    if let Some(ref path) = args.path {
        let description = args
            .description
            .clone()
            .unwrap_or_else(|| read_skill_description(path));
        return Ok(ResolvedSkill {
            name: name.to_string(),
            description,
            dir: path.clone(),
        });
    }

    if let Some(ref git) = args.git {
        let dir = fetch_git_checkout(
            git,
            args.branch.as_deref(),
            args.tag.as_deref(),
            args.rev.as_deref(),
            args.subdirectory.as_deref(),
        )?;
        let description = args
            .description
            .clone()
            .unwrap_or_else(|| read_skill_description(&dir));
        return Ok(ResolvedSkill {
            name: name.to_string(),
            description,
            dir,
        });
    }

    // bare owner/repo[/path][@ref] syntax
    if args.name_or_ref.contains('/') {
        let parsed = parse_github_ref(&args.name_or_ref)?;
        let dir = fetch_git_checkout(
            &parsed.git_url(),
            None,
            None,
            parsed.git_ref.as_deref(),
            parsed.subdirectory.as_deref(),
        )?;
        let effective_name = args.name.as_deref().unwrap_or(&parsed.inferred_name);
        let description = args
            .description
            .clone()
            .unwrap_or_else(|| read_skill_description(&dir));
        return Ok(ResolvedSkill {
            name: effective_name.to_string(),
            description,
            dir,
        });
    }

    bail!(
        "no source specified -- use `theta register skill <name>` (from manifest) \
         or provide --git, --path, or owner/repo shorthand"
    )
}

fn register_skill(
    args: RegisterSkillArgs,
    manifest_path: &Path,
    store: &StoreHandle,
) -> Result<()> {
    // dispatch: explicit source flags ---> standalone mode
    //           bare name (no /) ---> manifest mode
    //           name_or_ref with / ---> github shorthand (standalone)
    let has_explicit_source = args.git.is_some() || args.path.is_some();
    let is_github_shorthand = !has_explicit_source && args.name_or_ref.contains('/');

    let resolved = if has_explicit_source || is_github_shorthand {
        resolve_standalone_skill(&args)?
    } else {
        resolve_manifest_skill(&args.name_or_ref, manifest_path)?
    };

    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    let dest = store.register_skill(
        &resolved.name,
        &resolved.dir,
        &resolved.description,
        project_dir,
        args.force,
    )?;

    anstream::eprintln!(
        "{} skill '{}' --> {}",
        "registered".green().bold(),
        resolved.name.cyan(),
        dest.display()
    );
    Ok(())
}

fn register_rule(args: RegisterRuleArgs, manifest_path: &Path, store: &StoreHandle) -> Result<()> {
    super::require_manifest(manifest_path)?;
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let project_dir = super::project_dir(manifest_path)?;

    let rules = manifest
        .instructions
        .as_ref()
        .and_then(|i| i.rules.as_ref())
        .with_context(|| "no rules defined in theta.toml")?;
    let rule = rules
        .get(&args.name)
        .with_context(|| format!("rule '{}' not found in [instructions.rules]", args.name))?;

    let rule_file = match &rule.src {
        LocalOrGitRef::Local(path_ref) => project_dir.join(path_ref.as_str()),
        LocalOrGitRef::Git { .. } | LocalOrGitRef::System { .. } => {
            bail!(
                "rule '{}' source type is '{}' - only local path rules can be registered",
                args.name,
                rule.src.display_compact()
            )
        }
        _ => bail!("rule '{}' has an unsupported source type", args.name),
    };

    let description = rule
        .summary
        .clone()
        .or_else(|| rule.description.clone())
        .unwrap_or_else(|| format!("rule {}", args.name));

    let dest = store.register_rule(
        &args.name,
        &rule_file,
        &description,
        Some(rule.apply.as_str()),
        project_dir,
        args.force,
    )?;

    anstream::eprintln!(
        "{} rule '{}' --> {}",
        "registered".green().bold(),
        args.name.cyan(),
        dest.display()
    );
    Ok(())
}

fn register_agent(
    args: RegisterAgentArgs,
    manifest_path: &Path,
    store: &StoreHandle,
) -> Result<()> {
    super::require_manifest(manifest_path)?;
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let project_dir = super::project_dir(manifest_path)?;

    let agent_name = args.name.as_deref().unwrap_or(&manifest.agent.name);

    // optionally lock first
    if !args.no_lock {
        super::lock::execute(LockArgs { force: false }, manifest_path).with_context(
            || "theta lock failed before registration - fix errors above and retry",
        )?;
    }

    let dest = store.register_agent(
        agent_name,
        &manifest,
        manifest_path,
        project_dir,
        args.force,
    )?;

    anstream::eprintln!(
        "{} agent '{}' --> {}",
        "registered".green().bold(),
        agent_name.cyan(),
        dest.display()
    );
    Ok(())
}
