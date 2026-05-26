//! System resource store for theta.
//!
//! Owns all operations on the system store at `$XDG_DATA_HOME/theta/store/`:
//! registration, resolution, index management, and init-from-agent.
//! Commands delegate to `StoreHandle` methods and handle only user output.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use theta_static::{
    StoreEntry, StoreIndex, StoreIndexRuleEntry, StoreResourceKind, SystemStoreLayout,
};

pub struct StoreHandle {
    layout: SystemStoreLayout,
}

#[derive(Debug)]
pub struct InitFromAgentResult {
    pub agent_name: String,
}

impl StoreHandle {
    pub fn open() -> Result<Self> {
        let data_dir =
            theta_dirs::data_dir().context("could not determine theta data directory")?;
        let handle = Self {
            layout: SystemStoreLayout::new(&data_dir),
        };
        handle.ensure_builtins()?;
        Ok(handle)
    }

    // open at an explicit root - used in tests
    pub fn open_at(root: &Path) -> Self {
        Self {
            layout: SystemStoreLayout::new(root),
        }
    }

    pub fn layout(&self) -> &SystemStoreLayout {
        &self.layout
    }

    /// Seed any missing builtin skills into the system store.
    ///
    /// Called automatically by `open()` callers that want builtins available.
    /// Skips skills that already exist on disk (never overwrites user modifications).
    pub fn ensure_builtins(&self) -> Result<()> {
        for builtin in theta_static::BUILTIN_SKILLS {
            let skill_dir = self.layout.skill(builtin.name);
            if skill_dir.exists() {
                continue;
            }

            fs_err::create_dir_all(&skill_dir).with_context(|| {
                format!(
                    "failed to create builtin skill directory: {}",
                    skill_dir.display()
                )
            })?;

            for file in builtin.files {
                let dest = skill_dir.join(file.path);
                if let Some(parent) = dest.parent() {
                    fs_err::create_dir_all(parent)?;
                }
                fs_err::write(&dest, file.content)
                    .with_context(|| format!("failed to write builtin file: {}", dest.display()))?;
            }

            self.upsert_index(
                "skills",
                builtin.name,
                builtin.description,
                Path::new("builtin"),
                None,
            )
            .with_context(|| format!("failed to index builtin skill '{}'", builtin.name))?;
        }
        Ok(())
    }

    pub fn load_index(&self) -> Result<StoreIndex> {
        let index_path = self.layout.index();
        if !index_path.exists() {
            return Ok(StoreIndex::default());
        }

        let content = fs_err::read_to_string(&index_path)
            .with_context(|| format!("failed to read {}", index_path.display()))?;
        let doc = content
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| "failed to parse store index")?;

        parse_store_index(&doc)
    }

    pub fn skill_path(&self, name: &str) -> Option<PathBuf> {
        let p = self.layout.skill(name);
        if p.exists() { Some(p) } else { None }
    }

    pub fn rule_path(&self, name: &str) -> Option<PathBuf> {
        let p = self.layout.rule(name);
        if p.exists() { Some(p) } else { None }
    }

    pub fn agent_path(&self, name: &str) -> Option<PathBuf> {
        let p = self.layout.agent(name);
        if p.exists() { Some(p) } else { None }
    }

    pub fn register_skill(
        &self,
        name: &str,
        source_dir: &Path,
        description: &str,
        project_dir: &Path,
        force: bool,
    ) -> Result<PathBuf> {
        if !source_dir.exists() {
            bail!("skill directory not found: {}", source_dir.display());
        }

        let skill_md_path = source_dir.join(theta_static::SKILL_FILE_NAME);
        if !skill_md_path.exists() {
            bail!(
                "skill '{}' is missing SKILL.md at {}",
                name,
                skill_md_path.display()
            );
        }
        let skill_md = fs_err::read_to_string(&skill_md_path)
            .with_context(|| format!("failed to read {}", skill_md_path.display()))?;

        if theta_static::is_skill_template(&skill_md) {
            bail!(
                "skill '{name}' SKILL.md still has template content - replace with real skill documentation before registering"
            );
        }

        let dest = self.layout.skill(name);
        if dest.exists() && !force {
            bail!("skill '{name}' already exists in the system store - use --force to overwrite");
        }

        theta_static::copy_dir_recursive(source_dir, &dest)
            .with_context(|| format!("failed to copy skill '{name}' to store"))?;

        self.upsert_index("skills", name, description, project_dir, None)
            .with_context(|| "failed to update store index")?;

        Ok(dest)
    }

    pub fn register_rule(
        &self,
        name: &str,
        rule_file: &Path,
        description: &str,
        apply: Option<&str>,
        project_dir: &Path,
        force: bool,
    ) -> Result<PathBuf> {
        if !rule_file.exists() {
            bail!("rule file not found: {}", rule_file.display());
        }

        let content = fs_err::read_to_string(rule_file)
            .with_context(|| format!("failed to read {}", rule_file.display()))?;

        if theta_static::is_rule_template(&content) {
            bail!(
                "rule '{name}' still has template content - replace with real rule content before registering"
            );
        }

        let dest = self.layout.rule(name);
        if dest.exists() && !force {
            bail!("rule '{name}' already exists in the system store - use --force to overwrite");
        }

        let rules_dir = self.layout.rules_dir();
        fs_err::create_dir_all(&rules_dir)
            .with_context(|| format!("failed to create {}", rules_dir.display()))?;

        fs_err::copy(rule_file, &dest)
            .with_context(|| format!("failed to copy rule '{name}' to store"))?;

        self.upsert_index("rules", name, description, project_dir, apply)
            .with_context(|| "failed to update store index")?;

        Ok(dest)
    }

    pub fn register_agent(
        &self,
        name: &str,
        manifest: &theta_schema::ThetaManifest,
        manifest_path: &Path,
        project_dir: &Path,
        force: bool,
    ) -> Result<PathBuf> {
        if theta_static::is_placeholder_description(&manifest.agent.description) {
            bail!(
                "agent description is a placeholder - set a real description before registering (run `theta describe \"what your agent does\"`)"
            );
        }

        let dest = self.layout.agent(name);
        if dest.exists() && !force {
            bail!("agent '{name}' already exists in the system store - use --force to overwrite");
        }

        fs_err::create_dir_all(&dest)
            .with_context(|| format!("failed to create {}", dest.display()))?;

        let dest_manifest = self.layout.agent_manifest(name);
        fs_err::copy(manifest_path, &dest_manifest)
            .with_context(|| "failed to copy theta.toml to store")?;

        let lock_path = project_dir.join(theta_static::LOCKFILE);
        if lock_path.exists() {
            let dest_lock = self.layout.agent_lock(name);
            fs_err::copy(&lock_path, &dest_lock)
                .with_context(|| "failed to copy theta.lock to store")?;
        }

        // copy tightly-coupled instruction files
        if let Some(ref instructions) = manifest.instructions {
            if let Some(ref sys) = instructions.system {
                let src = project_dir.join(sys.as_str());
                if src.exists() {
                    let dest_instructions = dest.join(theta_static::INSTRUCTIONS_DIR);
                    fs_err::create_dir_all(&dest_instructions)?;
                    let dest_sys = dest_instructions.join(theta_static::SYSTEM_FILE_NAME);
                    fs_err::copy(&src, &dest_sys)?;
                }
            }
            if let Some(ref rules) = instructions.rules {
                for (rule_name, rule) in rules {
                    if let theta_schema::LocalOrGitRef::Local(path_ref) = &rule.src {
                        let src = project_dir.join(path_ref.as_str());
                        if src.exists() {
                            let dest_rules = dest
                                .join(theta_static::INSTRUCTIONS_DIR)
                                .join(theta_static::RULES_DIR);
                            fs_err::create_dir_all(&dest_rules)?;
                            let dest_rule = dest_rules.join(format!("{rule_name}.md"));
                            fs_err::copy(&src, &dest_rule)?;
                        }
                    }
                }
            }
        }

        self.upsert_index(
            "agents",
            name,
            &manifest.agent.description,
            project_dir,
            None,
        )
        .with_context(|| "failed to update store index")?;

        Ok(dest)
    }

    pub fn init_from_agent(
        &self,
        agent_name: &str,
        target_dir: &Path,
        manifest_path: &Path,
        force: bool,
    ) -> Result<InitFromAgentResult> {
        let stored_manifest = self.layout.agent_manifest(agent_name);
        if !stored_manifest.exists() {
            bail!(
                "agent '{agent_name}' not found in the system store - run `theta list store` to see available agents"
            );
        }

        if manifest_path.exists() && !force {
            bail!(
                "{} already exists - use --force to overwrite",
                manifest_path.display()
            );
        }

        let stored_content = fs_err::read_to_string(&stored_manifest)
            .with_context(|| format!("failed to read {}", stored_manifest.display()))?;
        let mut doc = stored_content
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| format!("failed to parse stored {}", stored_manifest.display()))?;

        let agent_store_dir = self.layout.agent(agent_name);

        let stored_system = agent_store_dir
            .join(theta_static::INSTRUCTIONS_DIR)
            .join(theta_static::SYSTEM_FILE_NAME);
        if stored_system.exists() {
            let dest = target_dir
                .join(theta_static::INSTRUCTIONS_DIR)
                .join(theta_static::SYSTEM_FILE_NAME);
            fs_err::create_dir_all(dest.parent().unwrap())?;
            fs_err::copy(&stored_system, &dest).with_context(|| "failed to copy system prompt")?;
            if let Some(tbl) = doc.get_mut("instructions").and_then(|i| i.as_table_mut()) {
                tbl.insert(
                    "system",
                    toml_edit::value(format!(
                        "{}/{}",
                        theta_static::INSTRUCTIONS_DIR,
                        theta_static::SYSTEM_FILE_NAME
                    )),
                );
            }
        }

        // copy instructions/rules/*.md if stored, rewrite src paths
        let stored_rules_dir = agent_store_dir
            .join(theta_static::INSTRUCTIONS_DIR)
            .join(theta_static::RULES_DIR);
        if stored_rules_dir.exists() {
            let dest_rules = target_dir
                .join(theta_static::INSTRUCTIONS_DIR)
                .join(theta_static::RULES_DIR);
            fs_err::create_dir_all(&dest_rules)?;

            for entry in fs_err::read_dir(&stored_rules_dir)
                .with_context(|| format!("failed to read {}", stored_rules_dir.display()))?
            {
                let entry = entry?;
                let file_name = entry.file_name();
                let Some(rule_name) = file_name.to_str().and_then(|n| n.strip_suffix(".md")) else {
                    continue;
                };

                fs_err::copy(entry.path(), dest_rules.join(&file_name))?;

                let rel_path = format!(
                    "{}/{}/{rule_name}.md",
                    theta_static::INSTRUCTIONS_DIR,
                    theta_static::RULES_DIR,
                );
                if let Some(src) = doc
                    .get_mut("instructions")
                    .and_then(|i| i.get_mut("rules"))
                    .and_then(|r| r.get_mut(rule_name))
                    .and_then(|t| t.get_mut("src"))
                {
                    *src = toml_edit::value(rel_path);
                }
            }
        }

        let stored_lock = self.layout.agent_lock(agent_name);
        if stored_lock.exists() {
            let dest_lock = target_dir.join(theta_static::LOCKFILE);
            fs_err::copy(&stored_lock, &dest_lock).with_context(|| "failed to copy theta.lock")?;
        }

        // write (possibly modified) theta.toml to target
        fs_err::write(manifest_path, doc.to_string())
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;

        Ok(InitFromAgentResult {
            agent_name: agent_name.to_string(),
        })
    }

    pub fn unregister(&self, kind: StoreResourceKind, name: &str) -> Result<()> {
        match kind {
            StoreResourceKind::Skill => {
                let path = self.layout.skill(name);
                if !path.exists() {
                    bail!("skill '{name}' not found in the system store");
                }
                fs_err::remove_dir_all(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
            StoreResourceKind::Rule => {
                let path = self.layout.rule(name);
                if !path.exists() {
                    bail!("rule '{name}' not found in the system store");
                }
                fs_err::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
            StoreResourceKind::Agent => {
                let path = self.layout.agent(name);
                if !path.exists() {
                    bail!("agent '{name}' not found in the system store");
                }
                fs_err::remove_dir_all(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
            _ => bail!("unsupported resource type: {kind}"),
        }

        self.remove_from_index(kind, name)?;

        Ok(())
    }

    fn upsert_index(
        &self,
        section: &str,
        name: &str,
        description: &str,
        project_dir: &Path,
        apply: Option<&str>,
    ) -> Result<()> {
        let index_path = self.layout.index();

        let mut doc = if index_path.exists() {
            let content = fs_err::read_to_string(&index_path)?;
            content
                .parse::<toml_edit::DocumentMut>()
                .unwrap_or_default()
        } else {
            toml_edit::DocumentMut::new()
        };

        doc.as_table_mut()
            .entry(section)
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));

        let now = epoch_now();
        doc[section][name]["registered"] = toml_edit::value(&now);
        doc[section][name]["source_project"] = toml_edit::value(project_dir.display().to_string());
        doc[section][name]["description"] = toml_edit::value(description);
        if let Some(apply_mode) = apply {
            doc[section][name]["apply"] = toml_edit::value(apply_mode);
        }

        fs_err::create_dir_all(self.layout.root())?;
        fs_err::write(&index_path, doc.to_string())?;
        Ok(())
    }

    fn remove_from_index(&self, kind: StoreResourceKind, name: &str) -> Result<()> {
        let index_path = self.layout.index();
        if !index_path.exists() {
            return Ok(());
        }

        let content = fs_err::read_to_string(&index_path)?;
        let mut doc = content
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_default();

        let section = match kind {
            StoreResourceKind::Skill => "skills",
            StoreResourceKind::Rule => "rules",
            StoreResourceKind::Agent => "agents",
            _ => bail!("unsupported resource type: {kind}"),
        };

        if let Some(tbl) = doc.get_mut(section).and_then(|v| v.as_table_mut()) {
            tbl.remove(name);
        }

        fs_err::write(&index_path, doc.to_string())?;
        Ok(())
    }
}

pub fn extract_skill_description(skill_md: &str) -> String {
    let Some(yaml_str) = theta_static::split_frontmatter(skill_md).0 else {
        return String::new();
    };
    theta_schema::SkillFrontmatter::parse(yaml_str)
        .ok()
        .and_then(|fm| fm.description)
        .unwrap_or_default()
}

#[allow(clippy::unnecessary_wraps)] // may become fallible
fn parse_store_index(doc: &toml_edit::DocumentMut) -> Result<StoreIndex> {
    let mut index = StoreIndex::default();

    if let Some(tbl) = doc.get("skills").and_then(|v| v.as_table()) {
        for (name, entry) in tbl {
            index.skills.insert(
                name.to_string(),
                StoreEntry {
                    registered: entry
                        .get("registered")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    source_project: entry
                        .get("source_project")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
            );
        }
    }

    if let Some(tbl) = doc.get("rules").and_then(|v| v.as_table()) {
        for (name, entry) in tbl {
            index.rules.insert(
                name.to_string(),
                StoreIndexRuleEntry {
                    registered: entry
                        .get("registered")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    source_project: entry
                        .get("source_project")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    apply: entry
                        .get("apply")
                        .and_then(|v| v.as_str())
                        .map(std::string::ToString::to_string),
                },
            );
        }
    }

    if let Some(tbl) = doc.get("agents").and_then(|v| v.as_table()) {
        for (name, entry) in tbl {
            index.agents.insert(
                name.to_string(),
                StoreEntry {
                    registered: entry
                        .get("registered")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    source_project: entry
                        .get("source_project")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
            );
        }
    }

    Ok(index)
}

fn epoch_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "0".to_string(), |d| d.as_secs().to_string())
}
#[cfg(test)]
mod tests {
    use super::*;

    fn store_in_tmp() -> (tempfile::TempDir, StoreHandle) {
        let dir = tempfile::tempdir().unwrap();
        let handle = StoreHandle::open_at(dir.path());
        (dir, handle)
    }

    fn make_skill_dir(root: &Path, name: &str, description: &str) -> PathBuf {
        let skill_dir = root.join(name);
        fs_err::create_dir_all(&skill_dir).unwrap();
        let content = format!(
            "---\nname: {name}\ndescription: \"{description}\"\n---\n\n# {name}\n\nReal content.\n"
        );
        fs_err::write(skill_dir.join("SKILL.md"), content).unwrap();
        skill_dir
    }

    fn make_rule_file(root: &Path, name: &str) -> PathBuf {
        fs_err::create_dir_all(root).unwrap();
        let path = root.join(format!("{name}.md"));
        fs_err::write(
            &path,
            "# Real rule content\n\nDo not produce harmful output.\n",
        )
        .unwrap();
        path
    }

    #[test]
    fn empty_store_returns_empty_index() {
        let (_dir, store) = store_in_tmp();
        let index = store.load_index().unwrap();
        assert!(index.skills.is_empty());
        assert!(index.rules.is_empty());
        assert!(index.agents.is_empty());
    }

    #[test]
    fn register_skill_copies_dir_and_updates_index() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        fs_err::create_dir_all(&project).unwrap();
        let skill_dir = make_skill_dir(&project.join("skills"), "osint", "OSINT skill");

        let dest = store
            .register_skill("osint", &skill_dir, "OSINT skill", &project, false)
            .unwrap();

        assert!(dest.join("SKILL.md").exists());
        let index = store.load_index().unwrap();
        assert!(index.skills.contains_key("osint"));
        assert_eq!(index.skills["osint"].description, "OSINT skill");
    }

    #[test]
    fn register_skill_rejects_template() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let skill_dir = project.join("skills").join("bad");
        fs_err::create_dir_all(&skill_dir).unwrap();
        fs_err::write(
            skill_dir.join("SKILL.md"),
            theta_static::skill_template("bad", theta_static::DEFAULT_SKILL_DESCRIPTION),
        )
        .unwrap();

        let err = store
            .register_skill("bad", &skill_dir, "placeholder", &project, false)
            .unwrap_err();
        assert!(err.to_string().contains("template content"));
    }

    #[test]
    fn register_skill_rejects_clash_without_force() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let skill_dir = make_skill_dir(&project.join("skills"), "dup", "A skill");

        store
            .register_skill("dup", &skill_dir, "A skill", &project, false)
            .unwrap();
        let err = store
            .register_skill("dup", &skill_dir, "A skill", &project, false)
            .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn register_skill_force_overwrites() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let skill_dir = make_skill_dir(&project.join("skills"), "overwrite", "version 1");

        store
            .register_skill("overwrite", &skill_dir, "version 1", &project, false)
            .unwrap();
        store
            .register_skill("overwrite", &skill_dir, "version 2", &project, true)
            .unwrap();

        let index = store.load_index().unwrap();
        assert_eq!(index.skills["overwrite"].description, "version 2");
    }

    #[test]
    fn register_rule_copies_file_and_updates_index() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let rule_file = make_rule_file(&project.join("instructions/rules"), "safety");

        let dest = store
            .register_rule(
                "safety",
                &rule_file,
                "Safety rule",
                Some("always"),
                &project,
                false,
            )
            .unwrap();

        assert!(dest.exists());
        let index = store.load_index().unwrap();
        assert!(index.rules.contains_key("safety"));
        assert_eq!(index.rules["safety"].apply.as_deref(), Some("always"));
    }

    #[test]
    fn register_rule_rejects_template() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let rule_dir = project.join("instructions/rules");
        fs_err::create_dir_all(&rule_dir).unwrap();
        let path = rule_dir.join("template.md");
        fs_err::write(&path, theta_static::DEFAULT_RULE_TEMPLATE).unwrap();

        let err = store
            .register_rule("template", &path, "template", None, &project, false)
            .unwrap_err();
        assert!(err.to_string().contains("template content"));
    }

    #[test]
    fn unregister_skill_removes_dir_and_index_entry() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let skill_dir = make_skill_dir(&project.join("skills"), "gone", "Bye");

        store
            .register_skill("gone", &skill_dir, "Bye", &project, false)
            .unwrap();
        assert!(store.skill_path("gone").is_some());

        store.unregister(StoreResourceKind::Skill, "gone").unwrap();

        assert!(store.skill_path("gone").is_none());
        let index = store.load_index().unwrap();
        assert!(!index.skills.contains_key("gone"));
    }

    #[test]
    fn unregister_rule_removes_file_and_index_entry() {
        let (dir, store) = store_in_tmp();
        let project = dir.path().join("project");
        let rule_file = make_rule_file(&project.join("rules"), "rm-me");

        store
            .register_rule("rm-me", &rule_file, "Remove me", None, &project, false)
            .unwrap();

        store.unregister(StoreResourceKind::Rule, "rm-me").unwrap();

        assert!(store.rule_path("rm-me").is_none());
        let index = store.load_index().unwrap();
        assert!(!index.rules.contains_key("rm-me"));
    }

    #[test]
    fn unregister_nonexistent_is_error() {
        let (_dir, store) = store_in_tmp();
        let err = store
            .unregister(StoreResourceKind::Skill, "nope")
            .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn skill_path_returns_none_for_unregistered() {
        let (_dir, store) = store_in_tmp();
        assert!(store.skill_path("nope").is_none());
    }

    #[test]
    fn rule_path_returns_none_for_unregistered() {
        let (_dir, store) = store_in_tmp();
        assert!(store.rule_path("nope").is_none());
    }

    #[test]
    fn extract_description_from_frontmatter() {
        let md = "---\nname: test\ndescription: \"My skill\"\n---\n\n# test\n";
        assert_eq!(extract_skill_description(md), "My skill");
    }

    #[test]
    fn extract_description_returns_empty_for_no_frontmatter() {
        assert_eq!(extract_skill_description("# just markdown"), "");
    }

    #[test]
    fn init_from_agent_copies_manifest_and_files() {
        let (dir, store) = store_in_tmp();

        // manually create a stored agent
        let agent_dir = store.layout().agent("test-agent");
        fs_err::create_dir_all(agent_dir.join("instructions")).unwrap();
        fs_err::write(
            store.layout().agent_manifest("test-agent"),
            "[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test-agent\"\n\
             description = \"A test agent\"\nversion = \"0.1.0\"\n\n[model]\ndefault = \"claude\"\n\
             \n[instructions]\nsystem = \"instructions/system.md\"\n",
        )
        .unwrap();
        fs_err::write(
            agent_dir.join("instructions").join("system.md"),
            "You are a test agent.",
        )
        .unwrap();

        // init into target
        let target = dir.path().join("target-project");
        fs_err::create_dir_all(&target).unwrap();
        let manifest_path = target.join("theta.toml");

        store
            .init_from_agent("test-agent", &target, &manifest_path, false)
            .unwrap();

        assert!(manifest_path.exists());
        assert!(target.join("instructions/system.md").exists());
        let sys = fs_err::read_to_string(target.join("instructions/system.md")).unwrap();
        assert_eq!(sys, "You are a test agent.");
    }

    #[test]
    fn init_from_agent_refuses_overwrite_without_force() {
        let (dir, store) = store_in_tmp();
        let agent_dir = store.layout().agent("blocked");
        fs_err::create_dir_all(&agent_dir).unwrap();
        fs_err::write(
            store.layout().agent_manifest("blocked"),
            "[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"blocked\"\n\
             description = \"Blocked\"\nversion = \"0.1.0\"\n\n[model]\ndefault = \"x\"\n",
        )
        .unwrap();

        let target = dir.path().join("blocked-target");
        fs_err::create_dir_all(&target).unwrap();
        let manifest_path = target.join("theta.toml");
        fs_err::write(&manifest_path, "existing").unwrap();

        let err = store
            .init_from_agent("blocked", &target, &manifest_path, false)
            .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn init_from_nonexistent_agent_fails() {
        let (_dir, store) = store_in_tmp();
        let target = std::env::temp_dir().join("nonexistent-test");
        let manifest = target.join("theta.toml");
        let err = store
            .init_from_agent("ghost", &target, &manifest, false)
            .unwrap_err();
        assert!(err.to_string().contains("not found"));
        // error message should reference correct command
        assert!(err.to_string().contains("theta list store"));
    }

    // builtin skill validation

    #[test]
    fn builtin_skills_have_valid_skill_md() {
        // Validates that every builtin skill shipped in the binary has:
        // - A SKILL.md file in its file list
        // - Parseable YAML frontmatter
        // - A `name` field matching the builtin's declared name
        // - A non-empty `description`
        // - Is not detected as a scaffold template
        for builtin in theta_static::BUILTIN_SKILLS {
            let skill_md_file = builtin
                .files
                .iter()
                .find(|f| f.path == "SKILL.md")
                .unwrap_or_else(|| {
                    panic!(
                        "builtin skill '{}' is missing SKILL.md in its file list",
                        builtin.name
                    )
                });

            let content = skill_md_file.content;

            assert!(
                !theta_static::is_skill_template(content),
                "builtin skill '{}' SKILL.md is still a scaffold template",
                builtin.name,
            );

            let (yaml_str, _body) = theta_static::split_frontmatter(content);
            let yaml_str = yaml_str.unwrap_or_else(|| {
                panic!(
                    "builtin skill '{}' SKILL.md has no YAML frontmatter",
                    builtin.name
                )
            });

            let fm = theta_schema::SkillFrontmatter::parse(yaml_str).unwrap_or_else(|e| {
                panic!(
                    "builtin skill '{}' SKILL.md has unparsable frontmatter: {e}",
                    builtin.name
                )
            });

            assert_eq!(
                fm.name.as_deref(),
                Some(builtin.name),
                "builtin skill '{}' frontmatter `name` does not match declared name",
                builtin.name,
            );

            assert!(
                fm.description.as_ref().is_some_and(|d| !d.is_empty()),
                "builtin skill '{}' frontmatter has empty or missing `description`",
                builtin.name,
            );

            assert!(
                !theta_static::is_placeholder_skill_description(
                    fm.description.as_deref().unwrap_or("")
                ),
                "builtin skill '{}' still has placeholder description",
                builtin.name,
            );
        }
    }

    #[test]
    fn ensure_builtins_seeds_missing_skills() {
        let (_dir, store) = store_in_tmp();

        // Fresh store — no skills yet
        assert!(store.skill_path("use-theta").is_none());

        store.ensure_builtins().unwrap();

        // After seeding, the skill should exist on disk
        let skill_path = store.skill_path("use-theta");
        assert!(
            skill_path.is_some(),
            "use-theta not found after ensure_builtins"
        );

        let skill_md = skill_path.unwrap().join("SKILL.md");
        assert!(skill_md.exists(), "SKILL.md not written for use-theta");

        // And be present in the index
        let index = store.load_index().unwrap();
        assert!(index.skills.contains_key("use-theta"));
        assert_eq!(index.skills["use-theta"].source_project, "builtin");
    }

    #[test]
    fn ensure_builtins_does_not_overwrite_existing() {
        let (_dir, store) = store_in_tmp();

        // Seed once
        store.ensure_builtins().unwrap();

        // User modifies the skill
        let skill_md = store.layout().skill_md("use-theta");
        fs_err::write(&skill_md, "# user-modified content\n").unwrap();

        // Re-seed should NOT overwrite
        store.ensure_builtins().unwrap();

        let content = fs_err::read_to_string(&skill_md).unwrap();
        assert_eq!(content, "# user-modified content\n");
    }

    #[test]
    fn ensure_builtins_is_idempotent() {
        let (_dir, store) = store_in_tmp();

        store.ensure_builtins().unwrap();
        store.ensure_builtins().unwrap();

        let index = store.load_index().unwrap();
        assert_eq!(index.skills.len(), theta_static::BUILTIN_SKILLS.len());
    }
}
