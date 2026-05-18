// Cursor caster
//
// cast (theta --> harness):
//   identity + system prompt --> .cursor/rules/system.md (alwaysApply: true)
//   per-rule                 --> .cursor/rules/<name>.mdc
//   harness config           --> .cursor/hooks.json
//   [tools]                  --> .cursor/mcp.json       ({"mcpServers": {...}})
//   [skills]                 --> .cursor/skills/<name>/SKILL.md
//
// import (harness --> theta):
//   .cursor/rules/system.md                --> identity + system prompt
//   .cursor/rules/*.mdc                    --> [instructions.rules]
//   .cursor/hooks.json                     --> [harness.cursor].hooks
//   .cursor/mcp.json mcpServers            --> [tools]
//   .cursor/skills/*/SKILL.md             --> [skills] (primary)
//   .agents/skills/*/SKILL.md             --> [skills] (cross-agent; hint if already in .cursor/skills/)
//
// ref: https://cursor.com/docs/rules
// ref: https://cursor.com/docs/mcp
// ref: https://cursor.com/docs/hooks
// ref: https://cursor.com/docs/skills

mod cast;
mod import;
pub(crate) mod notes;

// .mdc rule frontmatter field names
// ref: cursor.com/docs/rules#rule-anatomy
const MDC_DESCRIPTION: &str = "description";
const MDC_GLOBS: &str = "globs";
const MDC_ALWAYS_APPLY: &str = "alwaysApply";

use std::path::Path;

use crate::Caster;
use crate::common::CastFile;
use crate::harness_config::CursorConfig;
use crate::{ImportOptions, ImportResult, Importer};
use anyhow::Result;
use theta_harness::layout::CursorLayout;
use theta_schema::{Diagnostic, ThetaManifest};

pub struct CursorHarness;

impl Caster for CursorHarness {
    fn cast_files(&self, manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
        cast::cast_files(manifest, theta_dir)
    }

    // ref: https://cursor.com/docs/rules#best-practices - "Keep rules under 500 lines"
    fn validate_output(&self, files: &[CastFile]) -> Vec<Diagnostic> {
        let rules_dir = CursorLayout::rules_dir();
        let mut diags = Vec::new();
        for (path, content) in files {
            if !path.starts_with(&rules_dir) {
                continue;
            }
            let lines = content.lines().count();
            if lines > 500 {
                diags.push(Diagnostic::hint(
                    path.display().to_string(),
                    format!("{lines} lines (Cursor recommends <500 lines per rule)"),
                ));
            }
        }
        // TODO: warn on subagent prompts exceeding ~2000 words (soft recommendation)
        // ref: https://cursor.com/docs/subagents#anti-patterns-to-avoid

        diags
    }

    fn validate_config(&self, manifest: &ThetaManifest) -> Vec<Diagnostic> {
        let key = theta_harness::HarnessTarget::Cursor.toml_key();
        let cfg = match manifest.harness_config::<CursorConfig>(key) {
            Ok(Some(cfg)) => cfg,
            Ok(None) => return Vec::new(),
            Err(e) => {
                return vec![Diagnostic::warn(
                    format!("[harness.{key}]"),
                    format!("failed to parse {key} config: {e}"),
                )];
            }
        };
        crate::harness_config::validate_version(&cfg)
    }
}

impl Importer for CursorHarness {
    fn import(&self, project_dir: &Path, opts: &ImportOptions) -> Result<ImportResult> {
        import::import(project_dir, opts)
    }
}

#[cfg(test)]
mod tests;
