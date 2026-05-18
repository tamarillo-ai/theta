//! Integration tests for manifest verbs: add tool/skill/subagent, rm, list.

#[path = "manifest_verbs/add_skill.rs"]
mod add_skill;
#[path = "manifest_verbs/add_subagent.rs"]
mod add_subagent;
#[path = "manifest_verbs/add_tool.rs"]
mod add_tool;
#[path = "manifest_verbs/list.rs"]
mod list;
#[path = "manifest_verbs/rm.rs"]
mod rm;

use theta_manifest::{create_manifest, read_document, read_manifest};
use theta_schema::minimal_manifest;
use theta_static::MANIFEST_FILE_NAME;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);
    let mut manifest = minimal_manifest("test-agent");
    manifest.agent.description = "test agent".to_string();
    manifest.agent.model = Some("claude-sonnet-4-20250514".to_string());
    create_manifest(&path, &manifest).unwrap();
    (dir, path)
}

fn read_content(path: &std::path::Path) -> String {
    fs_err::read_to_string(path).unwrap()
}
