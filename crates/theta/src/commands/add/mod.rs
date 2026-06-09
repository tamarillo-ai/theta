//! `theta add`

mod rule;
mod skill;
mod subagent;
mod system;
mod tool;

use std::path::Path;

use anyhow::Result;
use theta_args::{AddCommand, AddNamespace, OutputFormat};
use theta_settings::ThetaSettings;

pub(crate) fn dispatch(
    ns: AddNamespace,
    output_format: OutputFormat,
    manifest_path: &Path,
    settings: &ThetaSettings,
) -> Result<()> {
    match ns.command {
        AddCommand::Rule(args) => rule::execute(args, output_format, manifest_path, settings),
        AddCommand::System(args) => system::execute(args, output_format, manifest_path, settings),
        AddCommand::Tool(args) => tool::execute(args, output_format, manifest_path),
        AddCommand::Skill(args) => skill::execute(args, output_format, manifest_path),
        AddCommand::Subagent(args) => subagent::execute(args, output_format, manifest_path),
    }
}
