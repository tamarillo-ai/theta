//! Uniform "exhaust" envelope for every `theta` verb's machine-readable output.
//!
//! Every verb wraps its result in a [`CommandOutput<T>`] where `T` is the
//! verb-specific payload. The envelope itself is identical across verbs, so
//! downstream codegen (notably `theta_py`) only has to template one shape.
//!
//! Diagnostics live alongside the payload — they carry the structured form of
//! what would otherwise be printed as colored lines. Anything that today goes
//! through `anstream::eprintln!("warn"|"error"|"hint", ...)` is the same data
//! as a [`crate::Diagnostic`].

use schemars::JsonSchema;
use serde::Serialize;

use crate::Diagnostic;

/// Outcome of a single verb invocation.
///
/// Distinct from `DiagLevel` on purpose: this is the *outcome* of the command,
/// while diagnostics are the *evidence* (which may include warnings or hints
/// even on `Ok`). Warnings never demote status — they remain `Ok`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CommandStatus {
    /// Intended work completed; no errors recorded.
    Ok,
    /// Intended work was a no-op (target already in desired state).
    ///
    /// Example: `lock` invoked when the lockfile is already current.
    NoOp,
    /// Verb did not complete its primary action; see diagnostics.
    Error,
}

impl CommandStatus {
    /// Process exit code corresponding to this status.
    ///
    /// This is the only place the mapping is defined. The CLI boundary
    /// consults it when an envelope is the program's final output.
    pub fn exit_code(self) -> i32 {
        match self {
            CommandStatus::Ok | CommandStatus::NoOp => 0,
            CommandStatus::Error => 1,
        }
    }
}

/// Uniform envelope around a verb's typed payload `T`.
///
/// `verb` is the invoked verb path (e.g. `["cast", "to"]` for `theta cast to`).
/// Subcommands surface as the trailing entries; the root program name is
/// excluded.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CommandOutput<T>
where
    T: Serialize + JsonSchema,
{
    pub verb: Vec<String>,
    pub status: CommandStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub data: T,
}

impl<T> CommandOutput<T>
where
    T: Serialize + JsonSchema,
{
    /// Build an `Ok` envelope with no diagnostics.
    pub fn ok(verb: impl IntoIterator<Item = impl Into<String>>, data: T) -> Self {
        Self {
            verb: verb.into_iter().map(Into::into).collect(),
            status: CommandStatus::Ok,
            diagnostics: Vec::new(),
            data,
        }
    }

    /// Build a `NoOp` envelope with no diagnostics.
    pub fn no_op(verb: impl IntoIterator<Item = impl Into<String>>, data: T) -> Self {
        Self {
            verb: verb.into_iter().map(Into::into).collect(),
            status: CommandStatus::NoOp,
            diagnostics: Vec::new(),
            data,
        }
    }

    /// Build an `Error` envelope. Callers should attach at least one
    /// `Diagnostic::error` describing the failure.
    pub fn error(
        verb: impl IntoIterator<Item = impl Into<String>>,
        data: T,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            verb: verb.into_iter().map(Into::into).collect(),
            status: CommandStatus::Error,
            diagnostics,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;

    #[derive(Serialize, JsonSchema)]
    struct ProbePayload {
        files_written: Vec<String>,
    }

    #[test]
    fn ok_envelope_serializes_with_expected_keys() {
        let out = CommandOutput::ok(
            ["init"],
            ProbePayload {
                files_written: vec!["theta.toml".into()],
            },
        );
        let v = serde_json::to_value(&out).unwrap();
        assert_eq!(v["verb"], serde_json::json!(["init"]));
        assert_eq!(v["status"], serde_json::json!("ok"));
        assert_eq!(v["diagnostics"], serde_json::json!([]));
        assert_eq!(
            v["data"],
            serde_json::json!({ "files_written": ["theta.toml"] })
        );
    }

    #[test]
    fn no_op_status_serializes_lowercase() {
        let out = CommandOutput::no_op(
            ["lock"],
            ProbePayload {
                files_written: vec![],
            },
        );
        let v = serde_json::to_value(&out).unwrap();
        assert_eq!(v["status"], serde_json::json!("noop"));
    }

    #[test]
    fn error_status_carries_diagnostics() {
        let diags = vec![Diagnostic::error("[manifest]", "could not parse")];
        let out = CommandOutput::error(
            ["check"],
            ProbePayload {
                files_written: vec![],
            },
            diags,
        );
        let v = serde_json::to_value(&out).unwrap();
        assert_eq!(v["status"], serde_json::json!("error"));
        assert_eq!(v["diagnostics"][0]["level"], serde_json::json!("error"));
        assert_eq!(v["diagnostics"][0]["path"], serde_json::json!("[manifest]"));
        assert_eq!(
            v["diagnostics"][0]["message"],
            serde_json::json!("could not parse")
        );
    }

    #[test]
    fn exit_code_mapping_is_fixed() {
        assert_eq!(CommandStatus::Ok.exit_code(), 0);
        assert_eq!(CommandStatus::NoOp.exit_code(), 0);
        assert_eq!(CommandStatus::Error.exit_code(), 1);
    }

    #[test]
    fn json_schema_derives_compile() {
        // Just make sure schemars can build a schema for the generic envelope.
        let _ = schema_for!(CommandOutput<ProbePayload>);
    }
}
