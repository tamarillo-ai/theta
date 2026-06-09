//! `theta add tool` — register an MCP tool server in the manifest.
//!
//! Supports two modes:
//!
//! - **Explicit**: `theta add tool my-tool --command "npx -y @server/foo"`
//! - **Registry**: `theta add tool io.github.user/my-tool[@version]` (auto-detected)

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use std::fmt::Write;
use std::path::Path;
use theta_args::AddToolArgs;
use theta_manifest::{ensure_table, parse_manifest, read_document, write_document};
use theta_schema::Validate;

use crate::commands::{report_diagnostics, require_manifest};

struct ToolEntry {
    name: String,
    command: Option<Vec<String>>,
    url: Option<String>,
    envs: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    args: Vec<String>,
    disabled: bool,
    source_label: Option<String>,
}

pub(super) fn execute(args: AddToolArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let entry = if theta_registry::is_registry_name(&args.name) {
        resolve_from_registry(&args)?
    } else {
        resolve_explicit(&args)?
    };

    write_tool_entry(&entry, manifest_path)?;

    let transport = if entry.command.is_some() {
        "stdio"
    } else {
        "http"
    };
    match &entry.source_label {
        Some(source) => anstream::eprintln!(
            "{} tool \"{}\" from {} ({})",
            "registered".green().bold(),
            entry.name.cyan(),
            source.dimmed(),
            transport,
        ),
        None => anstream::eprintln!(
            "{} tool \"{}\" ({})",
            "registered".green().bold(),
            entry.name.cyan(),
            transport,
        ),
    }

    if !entry.envs.is_empty() {
        let has_placeholder = entry.envs.iter().any(|(_, v)| v.contains("${env:"));
        if has_placeholder {
            anstream::eprintln!(
                "{} some env values are ${{env:NAME}} placeholders - set the actual values in your environment",
                "hint".blue().bold(),
            );
        }
    }

    Ok(())
}

fn resolve_from_registry(args: &AddToolArgs) -> Result<ToolEntry> {
    let (registry_name, version) = theta_registry::parse_registry_ref(&args.name);

    anstream::eprintln!(
        "{} {}{}",
        "resolving".blue().bold(),
        registry_name.cyan(),
        version
            .map(|v| format!("@{v}"))
            .unwrap_or_default()
            .dimmed(),
    );

    let client = theta_registry::RegistryClient::new(args.registry.as_deref())?;
    let resp = if args.no_cache {
        client.get_server_no_cache(registry_name, version)?
    } else {
        client.get_server(registry_name, version)?
    };
    let tool = theta_registry::synthesize_tool(&resp.server)?;

    anstream::eprintln!(
        "{} {} v{} ({})",
        "found".green().bold(),
        resp.server.name.cyan(),
        tool.version.dimmed(),
        tool.command.as_ref().map_or("remote", |c| c[0].as_str()),
    );

    let mut envs = Vec::new();
    let required_env: Vec<_> = tool.env_vars.iter().filter(|v| v.is_required).collect();
    if !required_env.is_empty() {
        for var in &required_env {
            envs.push((var.name.clone(), format!("${{env:{}}}", var.name)));
        }
        anstream::eprintln!(
            "{} {} required env var(s) written as ${{env:NAME}} placeholders:",
            "env".blue().bold(),
            required_env.len(),
        );
        for var in &required_env {
            let desc = var.description.as_deref().unwrap_or("no description");
            let secret_tag = if var.is_secret { " (secret)" } else { "" };
            anstream::eprintln!("  {} - {}{}", var.name.yellow(), desc, secret_tag.dimmed());
        }
    }

    for (key, value) in &args.envs {
        envs.retain(|(k, _)| k != key);
        envs.push((key.clone(), value.clone()));
    }

    // build headers from registry metadata + explicit overrides
    //
    // server.json uses two mechanisms for sensitive headers:
    //   1. isSecret: true - the client should prompt, never hardcode the value
    //   2. {curly_brace} template vars in value, resolved from a variables map
    //
    // theta converts both to ${env:NAME} placeholders so the theta.toml is safe
    // to commit. literal non-secret values (e.g. "X-Region: us-east-1") remain
    // unchanged
    //
    // see: https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md
    let mut headers = Vec::new();
    if !tool.headers.is_empty() {
        for hdr in &tool.headers {
            let raw_value = hdr
                .value
                .as_deref()
                .or(hdr.default.as_deref())
                .unwrap_or("");
            let has_template = raw_value.contains('{') && raw_value.contains('}');

            let value = if raw_value.is_empty() || hdr.is_secret {
                format!("${{env:{}}}", hdr.name.replace('-', "_").to_uppercase())
            } else if has_template {
                template_to_env_ref(raw_value)
            } else {
                raw_value.to_string()
            };
            headers.push((hdr.name.clone(), value));
        }
        let has_placeholder = headers.iter().any(|(_, v)| v.contains("${env:"));
        if has_placeholder {
            anstream::eprintln!(
                "{} header(s) with ${{env:NAME}} placeholders - set the values in your environment:",
                "headers".blue().bold(),
            );
        } else {
            anstream::eprintln!("{} header(s) from registry:", "headers".blue().bold());
        }
        for (name, value) in &headers {
            let meta = tool.headers.iter().find(|h| h.name == *name);
            let desc = meta
                .and_then(|h| h.description.as_deref())
                .unwrap_or("no description");
            anstream::eprintln!("  {} = {} - {}", name.yellow(), value.dimmed(), desc);
        }
    }

    for (key, value) in &args.headers {
        headers.retain(|(k, _)| k != key);
        headers.push((key.clone(), value.clone()));
    }

    Ok(ToolEntry {
        name: tool.name,
        command: tool.command,
        url: tool.url,
        envs,
        headers,
        args: args.extra_args.clone(),
        disabled: args.disabled,
        source_label: Some(registry_name.to_string()),
    })
}

fn resolve_explicit(args: &AddToolArgs) -> Result<ToolEntry> {
    if args.command.is_none() && args.url.is_none() {
        bail!(
            "either --command or --url is required for local tool registration.\n\
             did you mean a registry reference? registry names contain '/' (e.g. io.github.user/tool-name)"
        );
    }
    if args.command.is_some() && args.url.is_some() {
        bail!("cannot specify both --command and --url");
    }
    if !theta_schema::is_valid_tool_name(&args.name) {
        bail!(
            "\"{}\" is not a valid tool name (lowercase alphanumeric + hyphens, no leading/trailing hyphens)",
            args.name
        );
    }

    let command = match &args.command {
        Some(cmd) => {
            let parts = shlex::split(cmd)
                .ok_or_else(|| anyhow::anyhow!("invalid shell quoting in --command value"))?;
            Some(parts)
        }
        None => None,
    };

    let envs: Vec<(String, String)> = args.envs.clone();

    let headers: Vec<(String, String)> = args.headers.clone();

    Ok(ToolEntry {
        name: args.name.clone(),
        command,
        url: args.url.clone(),
        envs,
        headers,
        args: args.extra_args.clone(),
        disabled: args.disabled,
        source_label: None,
    })
}

fn write_tool_entry(entry: &ToolEntry, manifest_path: &Path) -> Result<()> {
    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if theta_manifest::has_tool(&doc, &entry.name) {
        bail!(
            "tool \"{}\" is already registered in {}",
            entry.name,
            manifest_path.display()
        );
    }

    let tools_table = ensure_table(&mut doc, &["tools"]);
    let mut tool_table = toml_edit::Table::new();

    if let Some(ref cmd) = entry.command {
        let mut arr = toml_edit::Array::new();
        for part in cmd {
            arr.push(part.as_str());
        }
        tool_table["command"] = toml_edit::value(arr);
    }

    if let Some(ref url) = entry.url {
        tool_table["url"] = toml_edit::value(url.as_str());
    }

    if !entry.envs.is_empty() {
        let mut env_table = toml_edit::InlineTable::new();
        for (key, value) in &entry.envs {
            env_table.insert(key, toml_edit::Value::from(value.as_str()));
        }
        tool_table["env"] = toml_edit::value(env_table);
    }

    if !entry.headers.is_empty() {
        let mut hdr_table = toml_edit::InlineTable::new();
        for (key, value) in &entry.headers {
            hdr_table.insert(key, toml_edit::Value::from(value.as_str()));
        }
        tool_table["headers"] = toml_edit::value(hdr_table);
    }

    if !entry.args.is_empty() {
        let mut arr = toml_edit::Array::new();
        for a in &entry.args {
            arr.push(a.as_str());
        }
        tool_table["args"] = toml_edit::value(arr);
    }

    if entry.disabled {
        tool_table["enabled"] = toml_edit::value(false);
    }

    tools_table[&entry.name] = toml_edit::Item::Table(tool_table);

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;
    let mut diags = Vec::new();
    manifest.validate(&mut diags);
    let tool_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path.contains("[tools"))
        .collect();
    let (errors, _) = report_diagnostics(&tool_diags);
    if errors > 0 {
        bail!("tool rejected - manifest not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    Ok(())
}

// convert server.json {template_var} patterns to ${env:TEMPLATE_VAR} references
// see: https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md
fn template_to_env_ref(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let var: String = chars.by_ref().take_while(|&ch| ch != '}').collect();
            if var.is_empty() {
                result.push('{');
                result.push('}');
            } else {
                let env_name = var.replace('-', "_").to_uppercase();

                let _ = write!(result, "${{env:{env_name}}}");
            }
        } else {
            result.push(c);
        }
    }
    result
}
