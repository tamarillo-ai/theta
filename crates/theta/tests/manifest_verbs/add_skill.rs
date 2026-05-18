use super::*;
use theta_manifest::{ensure_table, write_document};

#[test]
fn add_skill_scaffold_creates_directory_and_skill_md() {
    let (dir, path) = setup();
    let project_dir = dir.path();

    // scaffold skill directory
    let skill_dir = project_dir.join("skills").join("my-analysis");
    fs_err::create_dir_all(&skill_dir).unwrap();

    let description = "Analyzes code for performance issues";
    let content = theta_static::skill_template("my-analysis", description);
    fs_err::write(skill_dir.join(theta_static::SKILL_FILE_NAME), &content).unwrap();
    fs_err::create_dir_all(skill_dir.join("scripts")).unwrap();
    fs_err::create_dir_all(skill_dir.join("references")).unwrap();
    fs_err::create_dir_all(skill_dir.join("assets")).unwrap();

    // register in theta.toml
    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["skills"]);
    let skills = doc["skills"].as_table_mut().unwrap();
    let mut skill = toml_edit::Table::new();
    let mut source = toml_edit::InlineTable::new();
    source.insert("path", toml_edit::Value::from("skills/my-analysis"));
    skill["source"] = toml_edit::value(source);
    skills["my-analysis"] = toml_edit::Item::Table(skill);
    write_document(&path, &doc).unwrap();

    // verify
    let manifest = read_manifest(&path).unwrap();
    let skills = manifest.skills.unwrap();
    let analysis = &skills["my-analysis"];
    assert_eq!(
        analysis.source,
        theta_schema::SourceRef::Path {
            path: "skills/my-analysis".to_string()
        }
    );

    // verify SKILL.md content
    let skill_md = fs_err::read_to_string(skill_dir.join(theta_static::SKILL_FILE_NAME)).unwrap();
    assert!(skill_md.contains("name: my-analysis"));
    assert!(skill_md.contains(description));
    assert!(skill_md.contains("## when to use"));

    // verify subdirectories
    assert!(skill_dir.join("scripts").is_dir());
    assert!(skill_dir.join("references").is_dir());
    assert!(skill_dir.join("assets").is_dir());
}

#[test]
fn add_skill_git_source_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    ensure_table(&mut doc, &["skills"]);
    let skills = doc["skills"].as_table_mut().unwrap();
    let mut skill = toml_edit::Table::new();
    let mut source = toml_edit::InlineTable::new();
    source.insert(
        "git",
        toml_edit::Value::from("https://github.com/tamarillo-ai/skills"),
    );
    source.insert("branch", toml_edit::Value::from("main"));
    skill["source"] = toml_edit::value(source);
    skills["osint"] = toml_edit::Item::Table(skill);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let skills = manifest.skills.unwrap();
    let osint = &skills["osint"];
    match &osint.source {
        theta_schema::SourceRef::Git {
            git,
            branch,
            subdirectory,
            ..
        } => {
            assert_eq!(git, "https://github.com/tamarillo-ai/skills");
            assert_eq!(branch.as_deref(), Some("main"));
            assert!(subdirectory.is_none());
        }
        other => panic!("expected Git source, got: {other:?}"),
    }
}

#[test]
fn skill_name_validation() {
    assert!(theta_schema::is_valid_skill_name("my-skill"));
    assert!(theta_schema::is_valid_skill_name("osint-investigation"));
    assert!(theta_schema::is_valid_skill_name("a"));
    assert!(theta_schema::is_valid_skill_name("a1b2c3"));

    assert!(!theta_schema::is_valid_skill_name(""));
    assert!(!theta_schema::is_valid_skill_name("-leading"));
    assert!(!theta_schema::is_valid_skill_name("trailing-"));
    assert!(!theta_schema::is_valid_skill_name("double--hyphen"));
    assert!(!theta_schema::is_valid_skill_name("UPPER"));
    assert!(!theta_schema::is_valid_skill_name("has space"));
    assert!(!theta_schema::is_valid_skill_name(
        &"a".repeat(65) // over 64 chars
    ));
}

#[test]
fn skill_template_generates_valid_frontmatter() {
    let content = theta_static::skill_template("test-skill", "A test skill");
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: test-skill"));
    assert!(content.contains("description: \"A test skill\""));
    assert!(content.contains("---\n\n#"));
}

#[test]
fn skill_template_detection() {
    let template = theta_static::skill_template("foo", "bar");
    assert!(theta_static::is_skill_template(&template));
    assert!(!theta_static::is_skill_template("# Custom content"));
}

#[test]
fn placeholder_skill_description_detection() {
    assert!(theta_static::is_placeholder_skill_description(
        theta_static::DEFAULT_SKILL_DESCRIPTION
    ));
    assert!(!theta_static::is_placeholder_skill_description(
        "A real description"
    ));
}
