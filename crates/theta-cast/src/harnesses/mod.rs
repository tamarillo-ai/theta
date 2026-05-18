mod claude_code;
mod codex_cli;
mod copilot;
mod cursor;
pub(crate) mod version;

pub use claude_code::ClaudeCode;
pub use codex_cli::CodexCli;
pub use copilot::Copilot;
pub use cursor::CursorHarness;

pub(crate) use claude_code::notes::{
    cast_notes as claude_code_cast_notes, import_notes as claude_code_import_notes,
};
pub(crate) use codex_cli::notes::{
    cast_notes as codex_cli_cast_notes, import_notes as codex_cli_import_notes,
};
pub(crate) use copilot::notes::{
    cast_notes as copilot_cast_notes, import_notes as copilot_import_notes,
};
pub(crate) use cursor::notes::{
    cast_notes as cursor_cast_notes, import_notes as cursor_import_notes,
};
