//! theta-install — materialization and validation of the `.theta/` project environment.
//!
//! Owns the full lifecycle of `.theta/`:
//! - **Validation**: `check_consistency()` verifies lock + `.theta/` state
//! - **Materialization**: `materialize()` writes locked resources into `.theta/`
//! - **Cleanup**: `cleanup_orphans()` removes stale artifacts

pub mod agent_graph;
pub mod materialize;
pub mod satisfies;

pub use agent_graph::{ResolvedSubagent, SubagentGraph, walk_subagent_graph};
pub use materialize::{MaterializeError, SyncReport, cleanup_orphans, materialize};
pub use satisfies::check_consistency;
