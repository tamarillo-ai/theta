use super::*;
use theta_manifest::write_document;

#[test]
fn add_subagent_inline_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("code-reviewer");
    entry["description"] = toml_edit::value("Reviews code for quality");
    entry["model"] = toml_edit::value("claude-sonnet-4-20250514");

    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].name, "code-reviewer");
    assert_eq!(subs[0].description, Some("Reviews code for quality".into()));
    assert_eq!(subs[0].model.as_deref(), Some("claude-sonnet-4-20250514"));
    assert!(subs[0].agent_ref.is_none());
}

#[test]
fn add_subagent_ref_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;

    // create a child theta.toml for the ref
    let child_manifest = minimal_manifest("researcher");
    let child_path = dir.path().join("agents").join("researcher.theta.toml");
    fs_err::create_dir_all(child_path.parent().unwrap()).unwrap();
    create_manifest(&child_path, &child_manifest).unwrap();

    let mut doc = read_document(&path).unwrap();

    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("researcher");
    entry["description"] = toml_edit::value("");
    entry["ref"] = toml_edit::value("agents/researcher.theta.toml");

    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].name, "researcher");
    assert_eq!(
        subs[0]
            .agent_ref
            .as_ref()
            .map(theta_schema::LocalPathRef::as_str),
        Some("agents/researcher.theta.toml")
    );
}

#[test]
fn add_subagent_with_tools_and_skills() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("planner");
    entry["description"] = toml_edit::value("Plans tasks");

    let mut tools_arr = toml_edit::Array::new();
    tools_arr.push("filesystem");
    tools_arr.push("browser");
    entry["tools"] = toml_edit::value(tools_arr);

    let mut skills_arr = toml_edit::Array::new();
    skills_arr.push("research");
    entry["skills"] = toml_edit::value(skills_arr);

    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs[0].tools.as_ref().unwrap().len(), 2);
    assert_eq!(
        subs[0].skills.as_ref().unwrap(),
        &vec!["research".to_string()]
    );
}
#[test]
fn add_subagent_with_prompt_path_writes_correct_toml() {
    let (dir, path) = setup();
    let project_dir = dir.path();

    // create the prompt file
    fs_err::create_dir_all(project_dir.join("subagents")).unwrap();
    fs_err::write(
        project_dir.join("subagents/reviewer.md"),
        "# reviewer\n\nYou review code.\n",
    )
    .unwrap();

    let mut doc = read_document(&path).unwrap();
    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("reviewer");
    entry["description"] = toml_edit::value("Reviews code");
    entry["prompt_path"] = toml_edit::value("subagents/reviewer.md");

    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs[0].name, "reviewer");
    assert_eq!(
        subs[0].prompt_path.as_deref(),
        Some("subagents/reviewer.md")
    );
    assert!(subs[0].agent_ref.is_none());
}

#[test]
fn prompt_path_round_trips_through_toml() {
    let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"
version = "0.1.0"

[[subagents]]
name = "planner"
description = "Plans work"
prompt_path = "subagents/planner.md"
model = "claude-sonnet-4-20250514"
"#;

    let manifest: theta_schema::ThetaManifest = toml::from_str(toml_str).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs[0].prompt_path.as_deref(), Some("subagents/planner.md"));
    assert!(subs[0].agent_ref.is_none());

    // mode should be inline
    assert_eq!(subs[0].mode(), theta_schema::SubagentMode::Inline);
}

#[test]
fn description_only_subagent_has_no_prompt_path() {
    let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"
version = "0.1.0"

[[subagents]]
name = "helper"
description = "A simple helper"
"#;

    let manifest: theta_schema::ThetaManifest = toml::from_str(toml_str).unwrap();
    let subs = manifest.subagents.unwrap();
    assert!(subs[0].prompt_path.is_none());
    assert!(subs[0].agent_ref.is_none());
    assert_eq!(subs[0].mode(), theta_schema::SubagentMode::DescriptionOnly);
}

#[test]
fn scaffold_subagent_creates_file_and_registers() {
    let (dir, path) = setup();
    let project_dir = dir.path();

    // simulate the CreateAndRegister intent:
    // 1. scaffold subagents/<name>.md
    let subagents_dir = project_dir.join("subagents");
    fs_err::create_dir_all(&subagents_dir).unwrap();
    let prompt_file = subagents_dir.join("planner.md");
    fs_err::write(&prompt_file, "# planner\n\nPlans the work\n").unwrap();
    assert!(prompt_file.exists());

    // 2. register with prompt_path pointing to the scaffolded file
    let mut doc = read_document(&path).unwrap();
    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("planner");
    entry["description"] = toml_edit::value("Plans the work");
    entry["prompt_path"] = toml_edit::value("subagents/planner.md");
    entry["model"] = toml_edit::value("claude-sonnet-4-20250514");

    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);

    write_document(&path, &doc).unwrap();

    // verify the manifest
    let manifest = read_manifest(&path).unwrap();
    let subs = manifest.subagents.unwrap();
    assert_eq!(subs[0].name, "planner");
    assert_eq!(subs[0].description, Some("Plans the work".into()));
    assert_eq!(subs[0].prompt_path.as_deref(), Some("subagents/planner.md"));
    assert_eq!(subs[0].mode(), theta_schema::SubagentMode::Inline);

    // verify the scaffolded file exists with correct content
    let content = fs_err::read_to_string(&prompt_file).unwrap();
    assert!(content.contains("# planner"));
    assert!(content.contains("Plans the work"));
}
