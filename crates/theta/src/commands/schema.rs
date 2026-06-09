//! `theta schema` — print the `theta.toml` JSON Schema or the CLI verb tree.

#![allow(clippy::print_stdout)]

use anyhow::Result;
use clap::CommandFactory;
use serde::Serialize;
use theta_args::{Cli, SchemaArgs};
use theta_schema::ThetaManifest;

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
    subcommands: Vec<VerbNode>,
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

    let subcommands = cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .map(|c| walk(c, &path))
        .collect();

    VerbNode {
        path,
        about: cmd.get_about().map(ToString::to_string),
        args,
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
