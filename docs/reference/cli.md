# CLI reference

<!-- generated - do not edit by hand -->
<!-- regenerate: cargo run -p theta-args --features docgen --example gen_cli_reference > docs/reference/cli.md -->

# Command-Line Help for `theta`

This document contains the help content for the `theta` command-line program.

**Command Overview:**

* [`theta`↴](#theta)
* [`theta init`↴](#theta-init)
* [`theta check`↴](#theta-check)
* [`theta describe`↴](#theta-describe)
* [`theta add`↴](#theta-add)
* [`theta add rule`↴](#theta-add-rule)
* [`theta add system`↴](#theta-add-system)
* [`theta add tool`↴](#theta-add-tool)
* [`theta add skill`↴](#theta-add-skill)
* [`theta add subagent`↴](#theta-add-subagent)
* [`theta rm`↴](#theta-rm)
* [`theta rm rule`↴](#theta-rm-rule)
* [`theta rm system`↴](#theta-rm-system)
* [`theta rm tool`↴](#theta-rm-tool)
* [`theta rm skill`↴](#theta-rm-skill)
* [`theta rm subagent`↴](#theta-rm-subagent)
* [`theta rm store`↴](#theta-rm-store)
* [`theta list`↴](#theta-list)
* [`theta list rules`↴](#theta-list-rules)
* [`theta list tools`↴](#theta-list-tools)
* [`theta list skills`↴](#theta-list-skills)
* [`theta list subagents`↴](#theta-list-subagents)
* [`theta list store`↴](#theta-list-store)
* [`theta lock`↴](#theta-lock)
* [`theta sync`↴](#theta-sync)
* [`theta cast`↴](#theta-cast)
* [`theta cast to`↴](#theta-cast-to)
* [`theta cast from`↴](#theta-cast-from)
* [`theta register`↴](#theta-register)
* [`theta register skill`↴](#theta-register-skill)
* [`theta register rule`↴](#theta-register-rule)
* [`theta register agent`↴](#theta-register-agent)
* [`theta tree`↴](#theta-tree)
* [`theta schema`↴](#theta-schema)

## `theta`

manage agent configurations defined by theta-spec

**Usage:** `theta [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `init` — Scaffold a new theta.toml in the current directory
* `check` — Validate theta.toml and materialized dependencies
* `describe` — Read or set the agent description
* `add` — Add a rule, tool, or skill to the manifest
* `rm` — Remove a rule, tool, skill, or subagent from the manifest
* `list` — List rules, tools, skills, or subagents
* `lock` — Resolve all sources and write theta.lock
* `sync` — Materialize dependencies into .theta/
* `cast` — Cast theta.toml to/from a harness-native config
* `register` — Register a resource into the system store
* `tree` — Print the subagent dependency tree
* `schema` — Print the theta.toml JSON Schema

###### **Options:**

* `--directory <DIRECTORY>` — Change the working directory before running
* `--manifest <MANIFEST>` — Path to a specific theta.toml file
* `--instructions-dir <INSTRUCTIONS_DIR>` — Override the instructions directory (default: "instructions")
* `--rules-dir <RULES_DIR>` — Override the rules subdirectory (default: "rules")



## `theta init`

Scaffold a new theta.toml in the current directory

**Usage:** `theta init [OPTIONS]`

###### **Options:**

* `--from <NAME>` — Initialize from a stored agent in the system store
* `--name <NAME>` — Agent name (defaults to directory name)
* `--force` — Overwrite existing theta.toml (only valid with --from)



## `theta check`

Validate theta.toml and materialized dependencies

**Usage:** `theta check [OPTIONS]`

###### **Options:**

* `--schema-only` — Only validate theta.toml against the JSON Schema
* `--skip-materialization` — Allow unresolved remote instruction refs as warnings
* `--output-format <OUTPUT_FORMAT>` — Output format

  Default value: `human`

  Possible values:
  - `human`:
    Human-readable colored output (default)
  - `json`:
    Machine-readable JSON




## `theta describe`

Read or set the agent description

**Usage:** `theta describe [OPTIONS] [DESCRIPTION]`

###### **Arguments:**

* `<DESCRIPTION>` — New description to set (if None, prints current)

###### **Options:**

* `--set <SET>` — Explicit flag form of setting description
* `--rules` — Also print rules and their summaries



## `theta add`

Add a rule, tool, or skill to the manifest

**Usage:** `theta add <COMMAND>`

###### **Subcommands:**

* `rule` — Scaffold and register a new rule
* `system` — Scaffold and register the system prompt
* `tool` — Register an MCP tool (stdio or HTTP)
* `skill` — Scaffold or register a skill
* `subagent` — Register a subagent (inline or ref)



## `theta add rule`

Scaffold and register a new rule

**Usage:** `theta add rule [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Rule name

###### **Options:**

* `--system <SYSTEM>` — Add from system store (writes `src = { system = "<store-name>" }`, no file scaffolding). When omitted, uses the leaf segment of the rule name (e.g. `backend/typescript` --> `typescript`)
* `--path <PATH>` — Path to an existing rule file
* `--git <GIT>` — Git repository URL
* `--branch <BRANCH>` — Git branch name
* `--tag <TAG>` — Git tag name
* `--rev <REV>` — Git commit SHA or rev-parse expression
* `--file <FILE>` — File path within the git repo
* `--sync` — Trigger immediate lock + sync after adding
* `--content <CONTENT>` — Initial rule content (overwrites default content)
* `--summary <SUMMARY>` — Short human-readable summary of the rule
* `--apply <APPLY>` — Activation mode

  Default value: `always`
* `--apply-to <APPLY_TO>` — File patterns (for apply = "glob")
* `--description <DESCRIPTION>` — Model-facing description (required for apply = "model-decision")



## `theta add system`

Scaffold and register the system prompt

**Usage:** `theta add system [OPTIONS]`

###### **Options:**

* `--path <PATH>` — Path to an existing system prompt file (registers without scaffolding)
* `--content <CONTENT>` — Initial content (replaces default template)



## `theta add tool`

Register an MCP tool (stdio or HTTP)

**Usage:** `theta add tool [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Tool name, or registry reference (io.github.user/tool[@version])

###### **Options:**

* `--command <COMMAND>` — Stdio command (shell-split into TOML array)
* `--url <URL>` — HTTP endpoint URL
* `--env <KEY=VALUE>` — Environment variable (KEY=VALUE, repeatable)
* `--header <KEY=VALUE>` — HTTP header (KEY=VALUE, repeatable) — only for url-based tools
* `--args <ARG>` — Additional arguments (repeatable)
* `--disabled` — Register the tool as disabled
* `--registry <REGISTRY>` — MCP registry URL (default: official MCP Registry)
* `--no-cache` — Bypass the registry cache (always fetch fresh metadata)



## `theta add skill`

Scaffold or register a skill

**Usage:** `theta add skill [OPTIONS] <NAME_OR_REF>`

###### **Arguments:**

* `<NAME_OR_REF>` — Skill name, or GitHub reference (owner/repo[/path][@ref])

###### **Options:**

* `--name <NAME>` — Override the inferred skill name
* `--path <PATH>` — Path to an existing local skill directory
* `--git <GIT>` — Git repository URL
* `--branch <BRANCH>` — Git branch name
* `--tag <TAG>` — Git tag name
* `--rev <REV>` — Git commit SHA or rev-parse expression
* `--subdirectory <SUBDIRECTORY>` — Subdirectory within git repo (requires --git or GitHub shorthand)
* `--description <DESCRIPTION>` — Skill description (used in scaffold template or metadata)
* `--tags <TAGS>` — Comma-separated tags for discovery and categorization
* `--goal <GOAL>` — Machine-facing purpose statement
* `--system` — Add from system store (writes `source = { system = "<name>" }`, no scaffolding)
* `--no-sync` — Plain addition without solving resources



## `theta add subagent`

Register a subagent (inline or ref)

**Usage:** `theta add subagent [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Subagent name

###### **Options:**

* `--agent-ref <AGENT_REF>` — Path to a child theta.toml
* `--description <DESCRIPTION>` — Subagent description (required for inline mode)
* `--model <MODEL>` — Model identifier
* `--prompt-path <PROMPT_PATH>` — Path to a .md file containing the system prompt
* `--tools <TOOLS>` — Comma-separated tool allow-list
* `--skills <SKILLS>` — Comma-separated skill names
* `--description-only` — Register with description only — no prompt file, no ref. Mutually exclusive with `--agent-ref` and `--prompt-path`. Requires `--description`



## `theta rm`

Remove a rule, tool, skill, or subagent from the manifest

**Usage:** `theta rm <COMMAND>`

###### **Subcommands:**

* `rule` — Remove a rule from the manifest
* `system` — Remove the system prompt from the manifest
* `tool` — Remove a tool from the manifest
* `skill` — Remove a skill from the manifest
* `subagent` — Remove a subagent from the manifest
* `store` — Unregister a resource from the system store



## `theta rm rule`

Remove a rule from the manifest

**Usage:** `theta rm rule [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Rule name to remove

###### **Options:**

* `--delete` — Also delete the source file
* `--no-sync` — Skip lock + sync after removing



## `theta rm system`

Remove the system prompt from the manifest

**Usage:** `theta rm system [OPTIONS]`

###### **Options:**

* `--delete` — Also delete the source file
* `--no-sync` — Skip lock + sync after removing



## `theta rm tool`

Remove a tool from the manifest

**Usage:** `theta rm tool <NAME>`

###### **Arguments:**

* `<NAME>` — Tool name to remove



## `theta rm skill`

Remove a skill from the manifest

**Usage:** `theta rm skill [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Skill name to remove

###### **Options:**

* `--delete` — Also delete the source directory
* `--no-sync` — Skip lock + sync after removing



## `theta rm subagent`

Remove a subagent from the manifest

**Usage:** `theta rm subagent [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Subagent name to remove

###### **Options:**

* `--delete` — Also delete the source file (ref theta.toml or `prompt_path` .md)
* `--no-sync` — Skip lock + sync after removing



## `theta rm store`

Unregister a resource from the system store

**Usage:** `theta rm store <TYPE> <NAME>`

###### **Arguments:**

* `<TYPE>` — Resource type (skill, rule, or agent)
* `<NAME>` — Resource name to unregister



## `theta list`

List rules, tools, skills, or subagents

**Usage:** `theta list [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `rules` — List registered rules
* `tools` — List registered tools (MCP servers)
* `skills` — List registered skills
* `subagents` — List registered subagents
* `store` — List contents of the system store

###### **Options:**

* `--output-format <OUTPUT_FORMAT>` — Output format

  Default value: `human`

  Possible values:
  - `human`:
    Human-readable colored output (default)
  - `json`:
    Machine-readable JSON




## `theta list rules`

List registered rules

**Usage:** `theta list rules`



## `theta list tools`

List registered tools (MCP servers)

**Usage:** `theta list tools`



## `theta list skills`

List registered skills

**Usage:** `theta list skills`



## `theta list subagents`

List registered subagents

**Usage:** `theta list subagents`



## `theta list store`

List contents of the system store

**Usage:** `theta list store`



## `theta lock`

Resolve all sources and write theta.lock

**Usage:** `theta lock [OPTIONS]`

###### **Options:**

* `--force` — Re-lock even if theta.lock is up to date



## `theta sync`

Materialize dependencies into .theta/

**Usage:** `theta sync [OPTIONS]`

###### **Options:**

* `--force` — Re-lock and re-sync even if everything is up to date



## `theta cast`

Cast theta.toml to/from a harness-native config

**Usage:** `theta cast <COMMAND>`

###### **Subcommands:**

* `to` — Cast theta.toml to a harness-native config
* `from` — Import a harness-native config into theta.toml



## `theta cast to`

Cast theta.toml to a harness-native config

**Usage:** `theta cast to [OPTIONS] <TARGET>`

###### **Arguments:**

* `<TARGET>` — Target harness

###### **Options:**

* `--output <OUTPUT>` — Output directory (defaults to current directory)
* `--force` — Overwrite existing harness config files
* `--notes` — Print known limitations and clarifying notes



## `theta cast from`

Import a harness-native config into theta.toml

**Usage:** `theta cast from [OPTIONS] <SOURCE>`

###### **Arguments:**

* `<SOURCE>` — Source harness to import from

###### **Options:**

* `--input <INPUT>` — Input directory (defaults to current directory)
* `--force` — Overwrite existing theta.toml
* `--subagent-prompts <SUBAGENT_PROMPTS>` — Directory to write externalized subagent prompt files (defaults to `<project>/subagents/`)
* `--force-overwrite` — Overwrite existing subagent prompt files with different content
* `--notes` — Print known limitations and clarifying notes
* `--cross-read` — Also import files from other harness locations that the source harness can discover



## `theta register`

Register a resource into the system store

**Usage:** `theta register <COMMAND>`

###### **Subcommands:**

* `skill` — Register a skill into the system store
* `rule` — Register a rule into the system store
* `agent` — Register this agent into the system store



## `theta register skill`

Register a skill into the system store

**Usage:** `theta register skill [OPTIONS] <NAME_OR_REF>`

###### **Arguments:**

* `<NAME_OR_REF>` — Skill name (from theta.toml), GitHub ref (owner/repo[/path][@ref]), or bare name when used with --path or --git

###### **Options:**

* `--name <NAME>` — Override the skill name stored in the system store
* `--path <PATH>` — Register from a local skill directory (no theta.toml needed)
* `--git <GIT>` — Register from a git repository URL (no theta.toml needed)
* `--branch <BRANCH>` — Git branch name
* `--tag <TAG>` — Git tag name
* `--rev <REV>` — Git commit SHA or rev-parse expression
* `--subdirectory <SUBDIRECTORY>` — Subdirectory within git repo containing the skill
* `--description <DESCRIPTION>` — Skill description (used in scaffold template or metadata)
* `--force` — Overwrite existing store entry



## `theta register rule`

Register a rule into the system store

**Usage:** `theta register rule [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Rule name (must match an [instructions.rules.<name>] entry in theta.toml)

###### **Options:**

* `--force` — Overwrite existing store entry



## `theta register agent`

Register this agent into the system store

**Usage:** `theta register agent [OPTIONS]`

###### **Options:**

* `--name <NAME>` — Agent name override (defaults to [agent].name in theta.toml)
* `--force` — Overwrite existing store entry
* `--no-lock` — Skip running theta lock before registering



## `theta tree`

Print the subagent dependency tree

**Usage:** `theta tree [OPTIONS]`

###### **Options:**

* `--output-format <OUTPUT_FORMAT>` — Output format

  Default value: `human`

  Possible values:
  - `human`:
    Human-readable colored output (default)
  - `json`:
    Machine-readable JSON




## `theta schema`

Print the theta.toml JSON Schema

**Usage:** `theta schema [OPTIONS]`

###### **Options:**

* `--list-verbs` — Emit a JSON tree of every verb in the CLI (name, path, args). When set, the manifest JSON Schema is not printed



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
