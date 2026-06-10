//! `theta schema` — print the `theta.toml` JSON Schema or the CLI verb tree.

#![allow(clippy::print_stdout)]

use anyhow::Result;
use clap::CommandFactory;
use schemars::schema::RootSchema;
use serde::Serialize;
use theta_args::{Cli, SchemaArgs};
use theta_schema::ThetaManifest;

use super::output::MutationOutput;

pub(crate) fn execute(args: SchemaArgs) -> Result<()> {
    if args.list_verbs {
        return print_verb_tree();
    }
    let schema = schemars::schema_for!(ThetaManifest);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}

fn print_verb_tree() -> Result<()> {
    let root = Cli::command();
    let tree = walk(&root, &[]);
    println!("{}", serde_json::to_string_pretty(&tree)?);
    Ok(())
}

#[derive(Serialize)]
struct VerbNode {
    path: Vec<String>,
    about: Option<String>,
    args: Vec<ArgNode>,
    /// JSON Schema for the verb's `data` payload inside the
    /// [`theta_schema::CommandOutput`] envelope. `None` for non-leaf nodes and
    /// for verbs that do not produce an envelope (e.g. `schema` itself).
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<RootSchema>,
    subcommands: Vec<VerbNode>,
}

/// Map a leaf verb path to its `data`-payload JSON Schema.
///
/// Returning `None` for a leaf means "this verb does not emit a
/// `CommandOutput` envelope" (currently only `schema` itself). Returning `None`
/// for a non-leaf is expected.
fn output_schema_for(path: &[String]) -> Option<RootSchema> {
    let segments: Vec<&str> = path.iter().map(String::as_str).collect();
    match segments.as_slice() {
        ["init"] => Some(schemars::schema_for!(super::init::InitOutcome)),
        ["check"] => Some(schemars::schema_for!(super::check::CheckOutcome)),
        ["migrate"] => Some(schemars::schema_for!(super::migrate::MigrateOutcome)),
        ["describe"] => Some(schemars::schema_for!(super::describe::DescribeOutcome)),
        ["lock"] => Some(schemars::schema_for!(super::lock::LockOutcome)),
        ["sync"] => Some(schemars::schema_for!(super::sync::SyncOutcome)),
        ["tree"] => Some(schemars::schema_for!(super::tree::TreeOutcome)),
        ["list", _] => Some(schemars::schema_for!(super::list::ListOutcome)),
        ["cast", "to"] => Some(schemars::schema_for!(super::cast::CastToOutcome)),
        ["cast", "from"] => Some(schemars::schema_for!(super::cast::CastFromOutcome)),
        ["add" | "rm" | "register", _] => Some(schemars::schema_for!(MutationOutput)),
        _ => None,
    }
}

#[derive(Serialize)]
struct ArgNode {
    name: String,
    long: Option<String>,
    short: Option<char>,
    help: Option<String>,
    required: bool,
    takes_value: bool,
    multiple: bool,
    default: Option<String>,
    value_choices: Option<Vec<String>>,
    env: Option<String>,
}

fn walk(cmd: &clap::Command, parent: &[String]) -> VerbNode {
    let mut path = parent.to_vec();
    if !parent.is_empty() || cmd.get_name() != theta_static::PROGRAM_NAME {
        path.push(cmd.get_name().to_string());
    }

    let args = cmd
        .get_arguments()
        .filter(|a| !a.is_hide_set())
        .map(arg_to_node)
        .collect();

    let subcommands: Vec<VerbNode> = cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .map(|c| walk(c, &path))
        .collect();

    let output_schema = if subcommands.is_empty() {
        output_schema_for(&path)
    } else {
        None
    };

    VerbNode {
        path,
        about: cmd.get_about().map(ToString::to_string),
        args,
        output_schema,
        subcommands,
    }
}

fn arg_to_node(arg: &clap::Arg) -> ArgNode {
    let possible = arg.get_possible_values();
    let value_choices: Option<Vec<String>> = if possible.is_empty() {
        None
    } else {
        Some(possible.iter().map(|v| v.get_name().to_string()).collect())
    };

    let default: Option<String> = {
        let defaults = arg.get_default_values();
        if defaults.is_empty() {
            None
        } else {
            Some(
                defaults
                    .iter()
                    .filter_map(|v| v.to_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        }
    };

    let env: Option<String> = arg
        .get_env()
        .and_then(|e| e.to_str().map(ToString::to_string));

    ArgNode {
        name: arg.get_id().to_string(),
        long: arg.get_long().map(ToString::to_string),
        short: arg.get_short(),
        help: arg.get_help().map(ToString::to_string),
        required: arg.is_required_set(),
        takes_value: arg.get_action().takes_values(),
        multiple: matches!(
            arg.get_action(),
            clap::ArgAction::Append | clap::ArgAction::Count
        ),
        default,
        value_choices,
        env,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leaf verbs that intentionally do not emit a `CommandOutput` envelope.
    /// `schema` itself is the introspection surface; it prints raw JSON Schema.
    const ENVELOPE_EXEMPT: &[&[&str]] = &[&["schema"], &["help"]];

    /// Collect every leaf verb path from the clap tree.
    fn leaf_paths() -> Vec<Vec<String>> {
        fn walk(cmd: &clap::Command, parent: &[String], out: &mut Vec<Vec<String>>) {
            let mut path = parent.to_vec();
            if !parent.is_empty() || cmd.get_name() != theta_static::PROGRAM_NAME {
                path.push(cmd.get_name().to_string());
            }
            let mut has_subs = false;
            for sub in cmd.get_subcommands().filter(|c| !c.is_hide_set()) {
                has_subs = true;
                walk(sub, &path, out);
            }
            if !has_subs && !path.is_empty() {
                out.push(path);
            }
        }
        let mut out = Vec::new();
        walk(&Cli::command(), &[], &mut out);
        out
    }

    /// Every leaf verb must either have an output schema or be in
    /// `ENVELOPE_EXEMPT`. If you add a verb, wire it into `output_schema_for`
    /// (or add it to the exempt list with a justification).
    #[test]
    fn every_leaf_verb_has_an_output_schema_or_is_exempt() {
        let mut missing: Vec<String> = Vec::new();
        for path in leaf_paths() {
            let segments: Vec<&str> = path.iter().map(String::as_str).collect();
            let exempt = ENVELOPE_EXEMPT.contains(&segments.as_slice());
            if !exempt && output_schema_for(&path).is_none() {
                missing.push(path.join(" "));
            }
        }
        assert!(
            missing.is_empty(),
            "verbs missing an output schema in `output_schema_for`: {missing:?}\n\
             Wire each one to its `*Outcome` type, or add the path to ENVELOPE_EXEMPT."
        );
    }
}
