//! `theta add`

mod rule;
mod skill;
mod subagent;
mod system;
mod tool;

use std::path::Path;

use anyhow::Result;
use theta_cli::{AddCommand, AddNamespace};
use theta_settings::ThetaSettings;

pub(crate) fn dispatch(
    ns: AddNamespace,
    manifest_path: &Path,
    settings: &ThetaSettings,
) -> Result<()> {
    match ns.command {
        AddCommand::Rule(args) => rule::execute(args, manifest_path, settings),
        AddCommand::System(args) => system::execute(args, manifest_path, settings),
        AddCommand::Tool(args) => tool::execute(args, manifest_path),
        AddCommand::Skill(args) => skill::execute(args, manifest_path),
        AddCommand::Subagent(args) => subagent::execute(args, manifest_path),
    }
}
