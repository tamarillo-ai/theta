// GitHub Copilot caster
//
// cast (theta --> harness):
//   identity + system prompt --> .github/copilot-instructions.md
//   per-rule                 --> .github/instructions/<name>.instructions.md
//   [skills]                 --> .github/skills/<name>/SKILL.md
//   [tools]                  --> .vscode/mcp.json  ({"servers": {...}})
//   [[subagents]]            --> .github/agents/<name>.agent.md
//   harness config           --> .vscode/settings.json
//
// import (harness --> theta):
//   .github/copilot-instructions.md          --> identity + system prompt
//   .github/instructions/*.instructions.md   --> [instructions.rules]
//   .github/skills/*/SKILL.md                --> [skills]
//   .vscode/mcp.json servers                 --> [tools]
//   .github/agents/*.agent.md                --> [[subagents]]
//   .vscode/settings.json                    --> [harness.github_copilot]
//
// ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions
// ref: https://code.visualstudio.com/docs/copilot/chat/mcp-servers
// ref: https://code.visualstudio.com/docs/copilot/customization/agent-skills
// ref: https://code.visualstudio.com/docs/copilot/customization/custom-agents
// ref: https://code.visualstudio.com/docs/copilot/reference/mcp-configuration

mod cast;
mod import;
pub(super) mod notes;

use std::path::Path;

use crate::Caster;
use crate::common::CastFile;
use crate::{ImportOptions, ImportResult, Importer};
use anyhow::Result;
use theta_schema::{Diagnostic, ThetaManifest};

pub struct Copilot;

impl Caster for Copilot {
    fn cast_files(&self, manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
        cast::cast_files_internal(manifest, theta_dir, None)
    }

    fn cast_files_with_output(
        &self,
        manifest: &ThetaManifest,
        theta_dir: &Path,
        output_dir: &Path,
    ) -> Result<Vec<CastFile>> {
        cast::cast_files_internal(manifest, theta_dir, Some(output_dir))
    }

    fn validate_config(&self, manifest: &ThetaManifest) -> Vec<Diagnostic> {
        cast::validate_config(manifest)
    }
}

impl Importer for Copilot {
    fn import(&self, project_dir: &Path, opts: &ImportOptions) -> Result<ImportResult> {
        import::import(project_dir, opts)
    }
}

#[cfg(test)]
mod tests;
