use super::*;
use theta_manifest::{ensure_table, write_document};

#[test]
fn list_tools_shows_registered_tools() {
    let (dir, path) = setup();
    let _ = dir;

    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    let mut arr = toml_edit::Array::new();
    arr.push("npx");
    arr.push("-y");
    arr.push("@mcp/server-filesystem");
    tool["command"] = toml_edit::value(arr);
    tools["filesystem"] = toml_edit::Item::Table(tool);
    write_document(&path, &doc).unwrap();

    let manifest = read_manifest(&path).unwrap();
    let tools = manifest.tools.unwrap();
    assert!(tools.contains_key("filesystem"));
    let fs = &tools["filesystem"];
    assert!(fs.command.is_some());
    assert!(fs.enabled);
}

#[test]
fn list_skills_shows_registered_skills() {
    let (dir, path) = setup();
    let _ = dir;

    let mut doc = read_document(&path).unwrap();
    ensure_table(&mut doc, &["skills"]);
    let skills = doc["skills"].as_table_mut().unwrap();
    let mut skill = toml_edit::Table::new();
    let mut source = toml_edit::InlineTable::new();
    source.insert(
        "git",
        toml_edit::Value::from("https://github.com/tamarillo/skills"),
    );
    source.insert("ref", toml_edit::Value::from("main"));
    skill["source"] = toml_edit::value(source);
    skills["osint"] = toml_edit::Item::Table(skill);
    write_document(&path, &doc).unwrap();

    let manifest = read_manifest(&path).unwrap();
    let skills = manifest.skills.unwrap();
    assert!(skills.contains_key("osint"));
}

#[test]
fn list_subagents_shows_registered_subagents() {
    let (dir, path) = setup();
    let _ = dir;

    let mut doc = read_document(&path).unwrap();
    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value("planner");
    entry["description"] = toml_edit::value("Plans tasks");
    entry["model"] = toml_edit::value("gpt-4o");
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
    assert_eq!(subs[0].name, "planner");
    assert_eq!(subs[0].model.as_deref(), Some("gpt-4o"));
}
