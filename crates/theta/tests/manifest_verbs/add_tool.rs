use super::*;
use theta_manifest::{ensure_table, write_document};

fn add_tool_stdio(doc: &mut toml_edit::DocumentMut, name: &str, command: &[&str]) {
    ensure_table(doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    let mut arr = toml_edit::Array::new();
    for part in command {
        arr.push(*part);
    }
    tool["command"] = toml_edit::value(arr);
    tools[name] = toml_edit::Item::Table(tool);
}

#[test]
fn add_tool_stdio_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    add_tool_stdio(
        &mut doc,
        "filesystem",
        &["npx", "-y", "@modelcontextprotocol/server-filesystem", "./"],
    );

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let tools = manifest.tools.unwrap();
    let fs_tool = &tools["filesystem"];
    assert_eq!(
        fs_tool.command.as_ref().unwrap(),
        &vec![
            "npx".to_string(),
            "-y".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
            "./".to_string(),
        ]
    );
    assert!(fs_tool.enabled);
    assert!(fs_tool.url.is_none());
}

#[test]
fn add_tool_http_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    tool["url"] = toml_edit::value("https://api.example.com/mcp");
    tools["remote-api"] = toml_edit::Item::Table(tool);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let tools = manifest.tools.unwrap();
    let api = &tools["remote-api"];
    assert_eq!(api.url.as_deref(), Some("https://api.example.com/mcp"));
    assert!(api.command.is_none());
}

#[test]
fn add_tool_with_env_writes_correct_toml() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    let mut arr = toml_edit::Array::new();
    arr.push("uvx");
    arr.push("osint-mcp");
    tool["command"] = toml_edit::value(arr);
    let mut env = toml_edit::InlineTable::new();
    env.insert(
        "OSINT_API_KEY",
        toml_edit::Value::from("${env:OSINT_API_KEY}"),
    );
    tool["env"] = toml_edit::value(env);
    tools["osint-mcp"] = toml_edit::Item::Table(tool);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let tools = manifest.tools.unwrap();
    let osint = &tools["osint-mcp"];
    assert_eq!(
        osint.env.as_ref().unwrap().get("OSINT_API_KEY").unwrap(),
        "${env:OSINT_API_KEY}"
    );
}

#[test]
fn add_tool_disabled_writes_enabled_false() {
    let (dir, path) = setup();
    let _ = dir;
    let mut doc = read_document(&path).unwrap();

    ensure_table(&mut doc, &["tools"]);
    let tools = doc["tools"].as_table_mut().unwrap();
    let mut tool = toml_edit::Table::new();
    let mut arr = toml_edit::Array::new();
    arr.push("npx");
    arr.push("server");
    tool["command"] = toml_edit::value(arr);
    tool["enabled"] = toml_edit::value(false);
    tools["disabled-tool"] = toml_edit::Item::Table(tool);

    write_document(&path, &doc).unwrap();
    let manifest = read_manifest(&path).unwrap();
    let tools = manifest.tools.unwrap();
    let dt = &tools["disabled-tool"];
    assert!(!dt.enabled);
}
