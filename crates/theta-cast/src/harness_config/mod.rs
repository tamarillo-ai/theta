//! Harness-specific configuration types — one struct per supported harness.
//!
//! These types are extensions defined by the theta CLI for first-class harnesses,
//! not part of the theta-spec standard. The manifest treats `[harness.*]` as opaque
//! `serde_json::Value`.
//!
//! These types provide typed access only for casting and validation.
//!
//! TODO/NOTE: if these configs are ever consumed by a crate other than theta-cast,
//! extract them into a shared `theta-harness-config` crate or similar.

mod claude_code;
mod codex_cli;
mod copilot;
mod cursor;

pub use claude_code::ClaudeCodeConfig;
pub use codex_cli::CodexCliConfig;
pub use copilot::CopilotConfig;
pub use cursor::CursorConfig;

use theta_schema::Diagnostic;

/// Shared version-constraint interface for all harness configs.
pub(crate) trait HasVersionConstraint {
    fn version(&self) -> Option<&str>;
    fn harness_name(&self) -> &'static str;
    fn detect_version(&self) -> Option<String>;
}

/// Validate version constraint for any harness config that implements `HasVersionConstraint`.
/// Returns a hint if no constraint is set, or a warn/hint when the constraint has a mismatch.
pub(crate) fn validate_version(cfg: &dyn HasVersionConstraint) -> Vec<Diagnostic> {
    let Some(constraint) = cfg.version() else {
        return vec![Diagnostic::hint(
            format!("[harness.{}]", cfg.harness_name()),
            "consider pinning a version constraint (e.g. version = \">=1.0.0\") to catch incompatible installs early".to_string(),
        )];
    };
    let detected = cfg.detect_version();
    crate::harnesses::version::check_version_constraint(
        cfg.harness_name(),
        constraint,
        detected.as_deref(),
    )
    .into_iter()
    .collect()
}
