//! Semantic equality checkers for cast round-trip testing.
//!
//! Each checker understands what "equal" means for a specific resource type,
//! tolerating known cosmetic differences (YAML quote style, key ordering,
//! inline vs block arrays, JSONC comments, CRLF normalization, trailing newline).

pub mod body;
pub mod claude_code;
pub mod codex_cli;
pub mod copilot;
pub mod cursor;
pub mod frontmatter;
pub mod json;
