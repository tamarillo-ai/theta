// harness version detection and constraint checking
//
// each `detect_*` function shells out to the harness binary (or reads a file) to find
// the installed version. returns None on any failure -- per spec, detection failures
// are silently ignored and do not block cast

use std::process::Command;
use std::time::Duration;

use semver::{Version, VersionReq};
use theta_schema::Diagnostic;
use wait_timeout::ChildExt;

/// Timeout for version-detection commands (`--version`, `--list-extensions`, etc.).
const VERSION_CMD_TIMEOUT: Duration = Duration::from_millis(5000);

/// Parse and check a version constraint against a detected version.
/// Returns a `Warn` diagnostic if the version falls outside the declared range,
/// a `Hint` if detection failed (constraint declared but version undetectable),
/// or `None` if either value is absent, unparsable, or matches.
pub(crate) fn check_version_constraint(
    harness_name: &str,
    constraint: &str,
    detected: Option<&str>,
) -> Option<Diagnostic> {
    let Some(installed_str) = detected else {
        return Some(Diagnostic::hint(
            format!("[harness.{harness_name}]"),
            format!(
                "version constraint \"{constraint}\" declared but harness version could not be detected - \\
                 install the harness or remove the constraint"
            ),
        ));
    };
    let req = match VersionReq::parse(constraint) {
        Ok(r) => r,
        Err(e) => {
            return Some(Diagnostic::warn(
                format!("[harness.{harness_name}]"),
                format!("invalid version constraint \"{constraint}\": {e}"),
            ));
        }
    };
    let installed = match Version::parse(installed_str) {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!(harness = harness_name, version = installed_str, error = %e, "installed version is not valid semver");
            return None;
        }
    };

    if req.matches(&installed) {
        None
    } else {
        Some(Diagnostic::warn(
            format!("[harness.{harness_name}]"),
            format!(
                "installed version {installed} does not satisfy declared constraint {constraint} - \
                 update or remove the version constraint"
            ),
        ))
    }
}

/// Run a command with a short timeout; return the first line of stdout, trimmed.
/// Returns `None` if the command fails, times out, or produces no output.
fn run_version_command(program: &str, args: &[&str]) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    match child.wait_timeout(VERSION_CMD_TIMEOUT) {
        Ok(Some(status)) => {
            if !status.success() {
                return None;
            }
            let mut stdout = String::new();
            if let Some(mut out) = child.stdout.take() {
                use std::io::Read;
                out.read_to_string(&mut stdout).ok()?;
            }
            stdout.lines().next().map(|l| l.trim().to_string())
        }
        Ok(None) => {
            tracing::warn!(program, "version command timed out");
            child.kill().ok();
            child.wait().ok();
            None
        }
        Err(_) => None,
    }
}

/// Parse a version string that may contain a prefix like `"claude 1.0.35"`.
/// Extracts the first token that parses as semver.
fn extract_semver(raw: &str) -> Option<String> {
    for token in raw.split_whitespace() {
        if Version::parse(token).is_ok() {
            return Some(token.to_string());
        }
    }
    None
}

/// Detect Claude Code CLI version via `claude --version`.
pub(crate) fn detect_claude_code_version() -> Option<String> {
    let raw = run_version_command("claude", &["--version"])?;
    extract_semver(&raw).or(Some(raw))
}

/// Detect Codex CLI version via `codex --version`.
pub(crate) fn detect_codex_cli_version() -> Option<String> {
    let raw = run_version_command("codex", &["--version"])?;
    extract_semver(&raw).or(Some(raw))
}

/// Detect Cursor IDE version via `cursor --version` (first line is the semver, e.g. `"2.0.69"`).
/// Falls back to reading `~/.cursor/version`, then `product.json` in the app bundle.
pub(crate) fn detect_cursor_version() -> Option<String> {
    // primary: cursor --version --> "2.0.69\n<commit>\n<arch>"
    if let Some(raw) = run_version_command("cursor", &["--version"]) {
        let ver = raw.lines().next().unwrap_or("").trim().to_string();
        if !ver.is_empty() {
            return Some(ver);
        }
    }

    let home = theta_dirs::home_dir()?;

    // secondary: ~/.cursor/version (undocumented post-first-launch cache)
    let ver_file = home.join(".cursor").join("version");
    if let Ok(content) = fs_err::read_to_string(&ver_file) {
        let raw = content.trim().to_string();
        if !raw.is_empty() {
            return Some(raw);
        }
    }

    // tertiary: product.json in the macOS app bundle
    let system_path =
        std::path::Path::new("/Applications/Cursor.app/Contents/Resources/app/product.json");
    let user_path = home
        .join("Applications")
        .join("Cursor.app")
        .join("Contents")
        .join("Resources")
        .join("app")
        .join("product.json");

    for path in &[system_path, user_path.as_path()] {
        if let Ok(content) = fs_err::read_to_string(path) {
            if let Some(ver) = extract_json_string_field(&content, "version") {
                if !ver.is_empty() {
                    return Some(ver);
                }
            }
        }
    }

    None
}

/// Detect GitHub Copilot extension version.
/// Primary: `code --list-extensions --show-versions` --> parses `github.copilot-chat@X.Y.Z`
///   or `github.copilot@X.Y.Z`. Prefers `github.copilot` (clean semver) when both present.
/// Fallback: scans `~/.vscode/extensions/` for `github.copilot-*` directories.
pub(crate) fn detect_copilot_version() -> Option<String> {
    // primary: VS Code CLI (spawns `code --list-extensions --show-versions`)
    if let Some(raw) = run_version_command("code", &["--list-extensions", "--show-versions"]) {
        let mut chat_version: Option<String> = None;
        let mut base_version: Option<String> = None;
        for line in raw.lines() {
            let line = line.trim();
            if let Some(ver) = line.strip_prefix("github.copilot-chat@") {
                chat_version = Some(ver.to_string());
            } else if let Some(ver) = line.strip_prefix("github.copilot@") {
                base_version = Some(ver.to_string());
            }
        }
        if let Some(v) = base_version.or(chat_version) {
            return Some(v);
        }
    }

    // fallback: filesystem scan of ~/.vscode/extensions/
    let home = theta_dirs::home_dir()?;
    let ext_dir = home.join(".vscode").join("extensions");
    let entries = fs_err::read_dir(&ext_dir).ok()?;
    let mut best: Option<String> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        if s.starts_with("github.copilot-") && !s.starts_with("github.copilot-language") {
            let version = read_package_json_version(&entry.path()).or_else(|| {
                let part = s.rsplit('-').next()?.to_string();
                Version::parse(&part).ok().map(|_| part)
            });
            if let Some(ver) = version {
                let needs_update = match &best {
                    None => true,
                    Some(cur) => Version::parse(&ver).ok() > Version::parse(cur.as_str()).ok(),
                };
                if needs_update {
                    best = Some(ver);
                }
            }
        }
    }
    best
}

fn read_package_json_version(ext_dir: &std::path::Path) -> Option<String> {
    let pkg = ext_dir.join("package.json");
    let content = fs_err::read_to_string(&pkg).ok()?;
    extract_json_string_field(&content, "version")
}

fn extract_json_string_field(content: &str, field: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    parsed.get(field)?.as_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use theta_schema::DiagLevel;

    // check_version_constraint

    #[test]
    fn matching_version_returns_none() {
        let result = check_version_constraint("copilot", ">=1.0.0", Some("1.2.3"));
        assert!(result.is_none());
    }

    #[test]
    fn exact_match_returns_none() {
        let result = check_version_constraint("cursor", "=2.0.69", Some("2.0.69"));
        assert!(result.is_none());
    }

    #[test]
    fn mismatch_returns_warn() {
        let d = check_version_constraint("copilot", ">=2.0.0", Some("1.5.0")).unwrap();
        assert_eq!(d.level, DiagLevel::Warn);
        assert!(d.message.contains("does not satisfy"));
    }

    #[test]
    fn no_detected_version_returns_hint() {
        let d = check_version_constraint("claude_code", ">=1.0.0", None).unwrap();
        assert_eq!(d.level, DiagLevel::Hint);
        assert!(d.message.contains("could not be detected"));
    }

    #[test]
    fn invalid_constraint_returns_warn() {
        let d = check_version_constraint("cursor", "not-semver!!!", Some("1.0.0")).unwrap();
        assert_eq!(d.level, DiagLevel::Warn);
        assert!(d.message.contains("invalid version constraint"));
    }

    #[test]
    fn unparsable_installed_version_returns_none() {
        let result = check_version_constraint("cursor", ">=1.0.0", Some("not-a-version"));
        assert!(result.is_none());
    }

    #[test]
    fn caret_constraint_matches() {
        assert!(check_version_constraint("copilot", "^1.2.0", Some("1.9.5")).is_none());
    }

    #[test]
    fn caret_constraint_rejects_major_bump() {
        let d = check_version_constraint("copilot", "^1.2.0", Some("2.0.0")).unwrap();
        assert_eq!(d.level, DiagLevel::Warn);
    }

    #[test]
    fn tilde_constraint_matches() {
        assert!(check_version_constraint("cursor", "~1.2.0", Some("1.2.9")).is_none());
    }

    #[test]
    fn tilde_constraint_rejects_minor_bump() {
        let d = check_version_constraint("cursor", "~1.2.0", Some("1.3.0")).unwrap();
        assert_eq!(d.level, DiagLevel::Warn);
    }

    // extract_semver

    #[test]
    fn extract_semver_from_prefixed_string() {
        assert_eq!(extract_semver("claude 1.0.35"), Some("1.0.35".into()));
    }

    #[test]
    fn extract_semver_plain_version() {
        assert_eq!(extract_semver("2.0.69"), Some("2.0.69".into()));
    }

    #[test]
    fn extract_semver_no_version() {
        assert_eq!(extract_semver("no version here"), None);
    }

    #[test]
    fn extract_semver_multiple_tokens_picks_first() {
        assert_eq!(
            extract_semver("tool 1.2.3 extra 4.5.6"),
            Some("1.2.3".into())
        );
    }

    #[test]
    fn extract_semver_empty_string() {
        assert_eq!(extract_semver(""), None);
    }

    // extract_json_string_field

    #[test]
    fn extract_field_from_valid_json() {
        let json = r#"{"version": "1.2.3", "name": "test"}"#;
        assert_eq!(
            extract_json_string_field(json, "version"),
            Some("1.2.3".into())
        );
    }

    #[test]
    fn extract_missing_field_returns_none() {
        let json = r#"{"name": "test"}"#;
        assert_eq!(extract_json_string_field(json, "version"), None);
    }

    #[test]
    fn extract_non_string_field_returns_none() {
        let json = r#"{"version": 123}"#;
        assert_eq!(extract_json_string_field(json, "version"), None);
    }

    #[test]
    fn extract_from_invalid_json_returns_none() {
        assert_eq!(extract_json_string_field("not json {{{", "version"), None);
    }

    #[test]
    fn extract_field_with_escaped_quotes() {
        let json = r#"{"version": "1.0.0-beta\"rc1\""}"#;
        // serde_json handles escapes correctly
        assert!(extract_json_string_field(json, "version").is_some());
    }
}
