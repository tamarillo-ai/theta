use super::*;
use theta_manifest::{ensure_table, write_document};

#[test]
fn rm_tool_removes_entry() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    // add a tool
    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    let mut arr = toml_edit::Array::new();
    arr.push("npx");
    arr.push("server");
    tool["command"] = toml_edit::value(arr);
    tools["filesystem"] = toml_edit::Item::Table(tool);
    write_document(&path, &doc).unwrap();

    // verify it exists
    let manifest = read_manifest(&path).unwrap();
    assert!(manifest.tools.unwrap().contains_key("filesystem"));

    // remove it
    let mut doc = read_document(&path).unwrap();
    let tools = doc["tools"].as_table_mut().unwrap();
    tools.remove("filesystem");
    if tools.is_empty() {
        doc.as_table_mut().remove("tools");
    }
    write_document(&path, &doc).unwrap();

    // verify removed
    let manifest = read_manifest(&path).unwrap();
    assert!(manifest.tools.is_none());
}

#[test]
fn rm_tool_cleans_up_empty_table() {
    let (dir, path) = setup();
    let _ = dir;

    // add two tools
    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();

    let mut t1 = toml_edit::Table::new();
    t1["url"] = toml_edit::value("https://a.com");
    tools["tool-a"] = toml_edit::Item::Table(t1);

    let mut t2 = toml_edit::Table::new();
    t2["url"] = toml_edit::value("https://b.com");
    tools["tool-b"] = toml_edit::Item::Table(t2);
    write_document(&path, &doc).unwrap();

    // remove tool-a - [tools] should remain
    let mut doc = read_document(&path).unwrap();
    let tools = doc["tools"].as_table_mut().unwrap();
    tools.remove("tool-a");
    write_document(&path, &doc).unwrap();

    let content = read_content(&path);
    assert!(content.contains("[tools.tool-b]"));
    assert!(!content.contains("tool-a"));

    // remove tool-b - [tools] should be gone
    let mut doc = read_document(&path).unwrap();
    let tools = doc["tools"].as_table_mut().unwrap();
    tools.remove("tool-b");
    if tools.is_empty() {
        doc.as_table_mut().remove("tools");
    }
    write_document(&path, &doc).unwrap();

    let content = read_content(&path);
    assert!(!content.contains("[tools"));
}

#[test]
fn rm_rule_does_not_delete_source_file_by_default() {
    let (dir, path) = setup();
    let project_dir = dir.path();

    // create a rule file and register it
    let rules_dir = project_dir.join("instructions").join("rules");
    fs_err::create_dir_all(&rules_dir).unwrap();
    fs_err::write(rules_dir.join("safety.md"), "# Safety rule").unwrap();

    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["instructions", "rules"]);
    let rules = doc["instructions"]["rules"].as_table_mut().unwrap();
    let mut rule = toml_edit::Table::new();
    rule["src"] = toml_edit::value("instructions/rules/safety.md");
    rules["safety"] = toml_edit::Item::Table(rule);
    write_document(&path, &doc).unwrap();

    // remove the rule (no --delete)
    let mut doc = read_document(&path).unwrap();
    let rules = doc["instructions"]["rules"].as_table_mut().unwrap();
    rules.remove("safety");
    // clean up empty tables
    let instructions = doc["instructions"].as_table_mut().unwrap();
    if instructions
        .get("rules")
        .and_then(|r| r.as_table())
        .is_some_and(toml_edit::Table::is_empty)
    {
        instructions.remove("rules");
    }
    if instructions.is_empty() {
        doc.as_table_mut().remove("instructions");
    }
    write_document(&path, &doc).unwrap();

    // source file should still exist
    assert!(rules_dir.join("safety.md").exists());

    // manifest should not have the rule
    let manifest = read_manifest(&path).unwrap();
    assert!(manifest.instructions.is_none());
}

#[test]
fn rm_rule_with_delete_removes_source_file() {
    let (dir, path) = setup();
    let project_dir = dir.path();

    let rules_dir = project_dir.join("instructions").join("rules");
    fs_err::create_dir_all(&rules_dir).unwrap();
    let rule_file = rules_dir.join("safety.md");
    fs_err::write(&rule_file, "# Safety rule").unwrap();

    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["instructions", "rules"]);
    let rules = doc["instructions"]["rules"].as_table_mut().unwrap();
    let mut rule = toml_edit::Table::new();
    rule["src"] = toml_edit::value("instructions/rules/safety.md");
    rules["safety"] = toml_edit::Item::Table(rule);
    write_document(&path, &doc).unwrap();

    // read source path before removal
    let src = "instructions/rules/safety.md";
    let full_path = project_dir.join(src);

    // remove from manifest
    let mut doc = read_document(&path).unwrap();
    let rules = doc["instructions"]["rules"].as_table_mut().unwrap();
    rules.remove("safety");
    write_document(&path, &doc).unwrap();

    // simulate --delete
    if full_path.exists() {
        fs_err::remove_file(&full_path).unwrap();
    }

    assert!(!rule_file.exists());
}

#[test]
fn rm_skill_removes_entry_and_cleans_table() {
    let (dir, path) = setup();
    let _ = dir;

    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["skills"]);
    let skills = doc["skills"].as_table_mut().unwrap();
    let mut skill = toml_edit::Table::new();
    let mut source = toml_edit::InlineTable::new();
    source.insert("path", toml_edit::Value::from("skills/test-skill"));
    skill["source"] = toml_edit::value(source);
    skills["test-skill"] = toml_edit::Item::Table(skill);
    write_document(&path, &doc).unwrap();

    // remove
    let mut doc = read_document(&path).unwrap();
    let skills = doc["skills"].as_table_mut().unwrap();
    skills.remove("test-skill");
    if skills.is_empty() {
        doc.as_table_mut().remove("skills");
    }
    write_document(&path, &doc).unwrap();

    let manifest = read_manifest(&path).unwrap();
    assert!(manifest.skills.is_none());
    let content = read_content(&path);
    assert!(!content.contains("[skills"));
}

#[test]
fn rm_subagent_removes_entry() {
    let (dir, path) = setup();
    let _ = dir;

    let mut doc = read_document(&path).unwrap();
    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("reviewer");
    entry["description"] = toml_edit::value("Reviews code");
    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .unwrap()
        .push(entry);
    write_document(&path, &doc).unwrap();

    // verify
    let manifest = read_manifest(&path).unwrap();
    assert_eq!(manifest.subagents.unwrap().len(), 1);

    // remove
    let mut doc = read_document(&path).unwrap();
    let arr = doc["subagents"].as_array_of_tables_mut().unwrap();
    arr.remove(0);
    if arr.is_empty() {
        doc.as_table_mut().remove("subagents");
    }
    write_document(&path, &doc).unwrap();

    let manifest = read_manifest(&path).unwrap();
    assert!(manifest.subagents.is_none());
}
