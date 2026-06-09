//! `clap` CLI definitions — argument parsing types only, no business logic.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

fn parse_env_pair(s: &str) -> Result<(String, String), String> {
    let (k, v) = theta_static::validate_env_pair(s).map_err(std::string::ToString::to_string)?;
    Ok((k.to_string(), v.to_string()))
}

fn parse_header_pair(s: &str) -> Result<(String, String), String> {
    let (key, value) = s.split_once('=').ok_or("expected KEY=VALUE format")?;
    if key.is_empty() {
        return Err("header name cannot be empty".to_string());
    }
    if value.is_empty() {
        return Err("header value cannot be empty".to_string());
    }
    Ok((key.to_string(), value.to_string()))
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable colored output (default)
    #[default]
    Human,
    /// Machine-readable JSON
    Json,
}

#[derive(Debug, Parser)]
#[command(
    name = theta_static::PROGRAM_NAME,
    about = "manage agent configurations defined by theta-spec",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub global: GlobalArgs,
}

// when theta grows in terms of global settings (~10+),
// a user-level config file **SHOULD** be considered (`uv.toml` pattern)
#[derive(Debug, clap::Args)]
pub struct GlobalArgs {
    /// Increase logging verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count, hide = true, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long, global = true, hide = true)]
    pub quiet: bool,

    /// Change the working directory before running
    #[arg(long, global = true)]
    pub directory: Option<PathBuf>,

    /// Path to a specific theta.toml file
    #[arg(long, global = true)]
    pub manifest: Option<PathBuf>,

    /// Override the instructions directory (default: "instructions")
    #[arg(long, global = true, env = "THETA_INSTRUCTIONS_DIR")]
    pub instructions_dir: Option<PathBuf>,

    /// Override the rules subdirectory (default: "rules")
    #[arg(long, global = true, env = "THETA_RULES_DIR")]
    pub rules_dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scaffold a new theta.toml in the current directory
    Init(InitArgs),

    /// Validate theta.toml and materialized dependencies
    Check(CheckArgs),

    /// Migrate theta.toml to the latest schema version
    #[command(hide = true)]
    Migrate(MigrateArgs),

    /// Read or set the agent description
    Describe(DescribeArgs),

    /// Add a rule, tool, or skill to the manifest
    Add(AddNamespace),

    /// Remove a rule, tool, skill, or subagent from the manifest
    Rm(RmNamespace),

    /// List rules, tools, skills, or subagents
    List(ListNamespace),

    /// Resolve all sources and write theta.lock
    Lock(LockArgs),

    /// Materialize dependencies into .theta/
    Sync(SyncArgs),

    /// Cast theta.toml to/from a harness-native config
    Cast(CastNamespace),

    /// Register a resource into the system store
    Register(RegisterNamespace),

    /// Print the subagent dependency tree
    Tree(TreeArgs),

    /// Print the theta.toml JSON Schema
    Schema(SchemaArgs),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Initialize from a stored agent in the system store
    #[arg(long, value_name = "NAME")]
    pub from: Option<String>,

    /// Agent name (defaults to directory name)
    #[arg(long)]
    pub name: Option<String>,

    /// Overwrite existing theta.toml (only valid with --from)
    #[arg(long, requires = "from")]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Only validate theta.toml against the JSON Schema
    #[arg(long, conflicts_with = "skip_materialization")]
    pub schema_only: bool,

    /// Allow unresolved remote instruction refs as warnings
    #[arg(long, conflicts_with = "schema_only")]
    pub skip_materialization: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub output_format: OutputFormat,
}

#[derive(Debug, clap::Args)]
pub struct MigrateArgs {
    /// Show what would change without writing (not yet implemented)
    #[arg(long, hide = true)]
    pub dry_run: bool,
}

#[derive(Debug, clap::Args)]
pub struct DescribeArgs {
    /// New description to set (if None, prints current)
    pub description: Option<String>,

    /// Explicit flag form of setting description
    #[arg(long, conflicts_with = "description")]
    pub set: Option<String>,

    /// Also print rules and their summaries
    #[arg(long)]
    pub rules: bool,
}

#[derive(Debug, clap::Args)]
pub struct AddNamespace {
    #[command(subcommand)]
    pub command: AddCommand,
}

#[derive(Debug, Subcommand)]
pub enum AddCommand {
    /// Scaffold and register a new rule
    Rule(AddRuleArgs),
    /// Scaffold and register the system prompt
    System(AddSystemArgs),
    /// Register an MCP tool (stdio or HTTP)
    Tool(AddToolArgs),
    /// Scaffold or register a skill
    Skill(AddSkillArgs),
    /// Register a subagent (inline or ref)
    Subagent(AddSubagentArgs),
}

#[derive(Debug, clap::Args)]
pub struct AddSystemArgs {
    /// Path to an existing system prompt file (registers without scaffolding)
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Initial content (replaces default template)
    #[arg(long)]
    pub content: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct AddRuleArgs {
    /// Rule name
    pub name: String,

    /// Add from system store (writes `src = { system = "<store-name>" }`, no file scaffolding).
    /// When omitted, uses the leaf segment of the rule name (e.g. `backend/typescript` --> `typescript`).
    #[arg(long, conflicts_with_all = ["path", "content", "summary", "apply", "apply_to", "description", "git", "branch", "tag", "rev", "file"],
          num_args = 0..=1, default_missing_value = "")]
    pub system: Option<String>,

    /// Path to an existing rule file
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Git repository URL
    #[arg(long, conflicts_with_all = ["path", "content", "system"])]
    pub git: Option<String>,

    /// Git branch name
    #[arg(long, requires = "git", conflicts_with_all = ["tag", "rev"])]
    pub branch: Option<String>,

    /// Git tag name
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "rev"])]
    pub tag: Option<String>,

    /// Git commit SHA or rev-parse expression
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "tag"])]
    pub rev: Option<String>,

    /// File path within the git repo
    #[arg(long, requires = "git")]
    pub file: Option<String>,

    /// Trigger immediate lock + sync after adding
    #[arg(long)]
    pub sync: bool,

    /// Initial rule content (overwrites default content)
    #[arg(long)]
    pub content: Option<String>,

    /// Short human-readable summary of the rule
    #[arg(long)]
    pub summary: Option<String>,

    /// Activation mode
    #[arg(long, default_value = "always")]
    pub apply: theta_schema::ApplyMode,

    /// File patterns (for apply = "glob")
    #[arg(long)]
    pub apply_to: Option<Vec<String>>,

    /// Model-facing description (required for apply = "model-decision")
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct AddToolArgs {
    /// Tool name, or registry reference (io.github.user/tool[@version])
    pub name: String,

    /// Stdio command (shell-split into TOML array)
    #[arg(long)]
    pub command: Option<String>,

    /// HTTP endpoint URL
    #[arg(long)]
    pub url: Option<String>,

    /// Environment variable (KEY=VALUE, repeatable)
    #[arg(long = "env", value_name = "KEY=VALUE", value_parser = parse_env_pair)]
    pub envs: Vec<(String, String)>,

    /// HTTP header (KEY=VALUE, repeatable) — only for url-based tools
    #[arg(long = "header", value_name = "KEY=VALUE", value_parser = parse_header_pair)]
    pub headers: Vec<(String, String)>,

    /// Additional arguments (repeatable)
    #[arg(long = "args", value_name = "ARG")]
    pub extra_args: Vec<String>,

    /// Register the tool as disabled
    #[arg(long)]
    pub disabled: bool,

    /// MCP registry URL (default: official MCP Registry)
    #[arg(long)]
    pub registry: Option<String>,

    /// Bypass the registry cache (always fetch fresh metadata)
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Debug, clap::Args)]
pub struct AddSkillArgs {
    /// Skill name, or GitHub reference (owner/repo[/path][@ref])
    pub name_or_ref: String,

    /// Override the inferred skill name
    #[arg(long)]
    pub name: Option<String>,

    /// Path to an existing local skill directory
    #[arg(long, group = "source")]
    pub path: Option<PathBuf>,

    /// Git repository URL
    #[arg(long, group = "source")]
    pub git: Option<String>,

    /// Git branch name
    #[arg(long, requires = "git", conflicts_with_all = ["tag", "rev"])]
    pub branch: Option<String>,

    /// Git tag name
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "rev"])]
    pub tag: Option<String>,

    /// Git commit SHA or rev-parse expression
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "tag"])]
    pub rev: Option<String>,

    /// Subdirectory within git repo (requires --git or GitHub shorthand)
    #[arg(long)]
    pub subdirectory: Option<String>,

    /// Skill description (used in scaffold template or metadata)
    #[arg(long)]
    pub description: Option<String>,

    /// Comma-separated tags for discovery and categorization
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Machine-facing purpose statement
    #[arg(long)]
    pub goal: Option<String>,

    /// Add from system store (writes `source = { system = "<name>" }`, no scaffolding)
    #[arg(long, group = "source")]
    pub system: bool,

    /// Plain addition without solving resources
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, clap::Args)]
pub struct AddSubagentArgs {
    /// Subagent name
    pub name: String,

    /// Path to a child theta.toml
    #[arg(long, conflicts_with_all = ["description", "model", "prompt_path", "tools", "skills"])]
    pub agent_ref: Option<PathBuf>,

    /// Subagent description (required for inline mode)
    #[arg(long)]
    pub description: Option<String>,

    /// Model identifier
    #[arg(long)]
    pub model: Option<String>,

    /// Path to a .md file containing the system prompt
    #[arg(long)]
    pub prompt_path: Option<PathBuf>,

    /// Comma-separated tool allow-list
    #[arg(long, value_delimiter = ',')]
    pub tools: Option<Vec<String>>,

    /// Comma-separated skill names
    #[arg(long, value_delimiter = ',')]
    pub skills: Option<Vec<String>>,

    /// Register with description only — no prompt file, no ref.
    /// Mutually exclusive with `--agent-ref` and `--prompt-path`.
    /// Requires `--description`.
    #[arg(long, conflicts_with_all = ["agent_ref", "prompt_path"])]
    pub description_only: bool,
}

#[derive(Debug, clap::Args)]
pub struct CastNamespace {
    #[command(subcommand)]
    pub command: CastCommand,
}

#[derive(Debug, Subcommand)]
pub enum CastCommand {
    /// Cast theta.toml to a harness-native config
    To(CastToArgs),

    /// Import a harness-native config into theta.toml
    From(CastFromArgs),
}

#[derive(Debug, clap::Args)]
pub struct CastToArgs {
    /// Target harness
    pub target: theta_harness::HarnessTarget,

    /// Output directory (defaults to current directory)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Overwrite existing harness config files
    #[arg(long)]
    pub force: bool,

    /// Print known limitations and clarifying notes
    #[arg(long)]
    pub notes: bool,
}

#[derive(Debug, clap::Args)]
pub struct CastFromArgs {
    /// Source harness to import from
    pub source: theta_harness::HarnessTarget,

    /// Input directory (defaults to current directory)
    #[arg(long)]
    pub input: Option<PathBuf>,

    /// Overwrite existing theta.toml
    #[arg(long)]
    pub force: bool,

    /// Directory to write externalized subagent prompt files
    /// (defaults to `<project>/subagents/`)
    #[arg(long)]
    pub subagent_prompts: Option<PathBuf>,

    /// Overwrite existing subagent prompt files with different content
    #[arg(long)]
    pub force_overwrite: bool,

    /// Print known limitations and clarifying notes
    #[arg(long)]
    pub notes: bool,

    /// Also import files from other harness locations that the source harness can discover
    #[arg(long)]
    pub cross_read: bool,
}

#[derive(Debug, clap::Args)]
pub struct ListNamespace {
    #[command(subcommand)]
    pub command: ListCommand,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    pub output_format: OutputFormat,
}

#[derive(Debug, Subcommand)]
pub enum ListCommand {
    /// List registered rules
    Rules,
    /// List registered tools (MCP servers)
    Tools,
    /// List registered skills
    Skills,
    /// List registered subagents
    Subagents,
    /// List contents of the system store
    Store,
}

#[derive(Debug, clap::Args)]
pub struct LockArgs {
    /// Re-lock even if theta.lock is up to date
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct SyncArgs {
    /// Re-lock and re-sync even if everything is up to date
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct TreeArgs {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub output_format: OutputFormat,
}

#[derive(Debug, clap::Args)]
pub struct SchemaArgs {}

#[derive(Debug, clap::Args)]
pub struct RmNamespace {
    #[command(subcommand)]
    pub command: RmCommand,
}

#[derive(Debug, Subcommand)]
pub enum RmCommand {
    /// Remove a rule from the manifest
    Rule(RmRuleArgs),
    /// Remove the system prompt from the manifest
    System(RmSystemArgs),
    /// Remove a tool from the manifest
    Tool(RmToolArgs),
    /// Remove a skill from the manifest
    Skill(RmSkillArgs),
    /// Remove a subagent from the manifest
    Subagent(RmSubagentArgs),
    /// Unregister a resource from the system store
    Store(RmStoreArgs),
}

#[derive(Debug, clap::Args)]
pub struct RmRuleArgs {
    /// Rule name to remove
    pub name: String,
    /// Also delete the source file
    #[arg(long)]
    pub delete: bool,
    /// Skip lock + sync after removing
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, clap::Args)]
pub struct RmSystemArgs {
    /// Also delete the source file
    #[arg(long)]
    pub delete: bool,
    /// Skip lock + sync after removing
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, clap::Args)]
pub struct RmToolArgs {
    /// Tool name to remove
    pub name: String,
}

#[derive(Debug, clap::Args)]
pub struct RmSkillArgs {
    /// Skill name to remove
    pub name: String,
    /// Also delete the source directory
    #[arg(long)]
    pub delete: bool,
    /// Skip lock + sync after removing
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, clap::Args)]
pub struct RmSubagentArgs {
    /// Subagent name to remove
    pub name: String,
    /// Also delete the source file (ref theta.toml or `prompt_path` .md)
    #[arg(long)]
    pub delete: bool,
    /// Skip lock + sync after removing
    #[arg(long)]
    pub no_sync: bool,
}

#[derive(Debug, clap::Args)]
pub struct RmStoreArgs {
    /// Resource type (skill, rule, or agent)
    #[arg(value_name = "TYPE")]
    pub kind: theta_static::StoreResourceKind,
    /// Resource name to unregister
    pub name: String,
}

#[derive(Debug, clap::Args)]
pub struct RegisterNamespace {
    #[command(subcommand)]
    pub command: RegisterCommand,
}

#[derive(Debug, Subcommand)]
pub enum RegisterCommand {
    /// Register a skill into the system store
    Skill(RegisterSkillArgs),
    /// Register a rule into the system store
    Rule(RegisterRuleArgs),
    /// Register this agent into the system store
    Agent(RegisterAgentArgs),
}

#[derive(Debug, clap::Args)]
pub struct RegisterSkillArgs {
    /// Skill name (from theta.toml), GitHub ref (owner/repo[/path][@ref]),
    /// or bare name when used with --path or --git
    pub name_or_ref: String,

    /// Override the skill name stored in the system store
    #[arg(long)]
    pub name: Option<String>,

    /// Register from a local skill directory (no theta.toml needed)
    #[arg(long, group = "source")]
    pub path: Option<PathBuf>,

    /// Register from a git repository URL (no theta.toml needed)
    #[arg(long, group = "source")]
    pub git: Option<String>,

    /// Git branch name
    #[arg(long, requires = "git", conflicts_with_all = ["tag", "rev"])]
    pub branch: Option<String>,

    /// Git tag name
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "rev"])]
    pub tag: Option<String>,

    /// Git commit SHA or rev-parse expression
    #[arg(long, requires = "git", conflicts_with_all = ["branch", "tag"])]
    pub rev: Option<String>,

    /// Subdirectory within git repo containing the skill
    #[arg(long, requires = "git")]
    pub subdirectory: Option<String>,

    /// Skill description (used in scaffold template or metadata)
    #[arg(long)]
    pub description: Option<String>,

    /// Overwrite existing store entry
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct RegisterRuleArgs {
    /// Rule name (must match an [instructions.rules.<name>] entry in theta.toml)
    pub name: String,
    /// Overwrite existing store entry
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct RegisterAgentArgs {
    /// Agent name override (defaults to [agent].name in theta.toml)
    #[arg(long)]
    pub name: Option<String>,
    /// Overwrite existing store entry
    #[arg(long)]
    pub force: bool,
    /// Skip running theta lock before registering
    #[arg(long)]
    pub no_lock: bool,
}

/// Generate markdown CLI reference from clap definitions.
#[cfg(feature = "docgen")]
pub fn generate_cli_reference() -> String {
    use clap::CommandFactory;
    let cmd = Cli::command();
    clap_markdown::help_markdown_command(&cmd)
}
