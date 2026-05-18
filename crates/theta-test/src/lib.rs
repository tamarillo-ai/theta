// shamelessly stolen from `uv`
#![allow(dead_code, unreachable_pub)]

pub mod cast_round_trip;
pub mod checkers;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::fixture::{ChildPath, PathChild, PathCreateDir};

/// Compile-time path to the `theta` binary built by cargo.
///
/// Only available from the `theta` crate's integration tests.
#[macro_export]
macro_rules! get_bin {
    () => {
        std::path::PathBuf::from(env!("CARGO_BIN_EXE_theta"))
    };
}

/// Create a [`TestContext`] wired to the compiled `theta` binary.
#[macro_export]
macro_rules! test_context {
    () => {
        $crate::TestContext::new(std::path::PathBuf::from(env!("CARGO_BIN_EXE_theta")))
    };
}

/// Insta snapshot filters applied to every test by default.
///
/// Normalises paths, timing, and other environment-dependent output so
/// snapshots stay stable across machines.
pub const INSTA_FILTERS: &[(&str, &str)] = &[
    // Absolute paths --> placeholder
    (r"(/[^\s]+/)\.theta", "[THETA_DIR]/.theta"),
    (r"/tmp/[^\s]+", "[TEMP]"),
    // Timing
    (r"(\s|\()(\d+m )?(\d+\.)?\d+(ms|s)", "$1[TIME]"),
    // Windows path separators
    (r"\\([\w\d]|\.)", "/$1"),
    // theta version
    (r"theta \d+\.\d+\.\d+", "theta [VERSION]"),
];

/// Shared test context for theta e2e tests.
///
/// Manages a temporary directory tree and provides command builders
/// that point at the compiled `theta` binary.
pub struct TestContext {
    pub temp_dir: ChildPath,
    pub data_dir: ChildPath,
    pub workspace_root: PathBuf,
    theta_bin: PathBuf,
    filters: Vec<(String, String)>,
    extra_env: Vec<(OsString, OsString)>,
    _root: TempDir,
}

impl TestContext {
    pub fn new(theta_bin: PathBuf) -> Self {
        let root = TempDir::new().expect("failed to create temp dir");
        let temp_dir = root.child("workspace");
        temp_dir
            .create_dir_all()
            .expect("failed to create workspace dir");
        let data_dir = root.child("data");
        data_dir
            .create_dir_all()
            .expect("failed to create data dir");

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("failed to find workspace root")
            .to_path_buf();

        let mut filters: Vec<(String, String)> = INSTA_FILTERS
            .iter()
            .map(|(p, r)| (p.to_string(), r.to_string()))
            .collect();

        // replace the temp dir path in output so snapshots are stable.
        filters.push((
            regex::escape(root.path().to_str().unwrap()),
            "[TEMP_DIR]".to_string(),
        ));

        Self {
            temp_dir,
            data_dir,
            workspace_root,
            theta_bin,
            filters,
            extra_env: Vec::new(),
            _root: root,
        }
    }

    /// Return the insta filters for this context.
    pub fn filters(&self) -> Vec<(String, String)> {
        self.filters.clone()
    }

    /// Path to the repo-root `test/` fixtures directory.
    pub fn fixtures_dir(&self) -> PathBuf {
        self.workspace_root.join("test")
    }

    /// Build an `assert_cmd::Command` for `theta` with the working
    /// directory set to the temp workspace.
    pub fn command(&self) -> Command {
        let mut cmd = Command::new(&self.theta_bin);
        cmd.current_dir(self.temp_dir.path());
        cmd.env_remove("THETA_CONFIG_DIR");
        cmd.env("THETA_DATA_DIR", self.data_dir.path());
        for (k, v) in &self.extra_env {
            cmd.env(k, v);
        }
        cmd
    }

    /// Shortcut: `theta cast to <target>`
    pub fn cast_to(&self, target: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["cast", "to", target]);
        cmd
    }

    /// Shortcut: `theta cast from <source>`
    pub fn cast_from(&self, source: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["cast", "from", source]);
        cmd
    }

    /// Shortcut: `theta init`
    pub fn init(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("init");
        cmd
    }

    /// Shortcut: `theta add <subcommand> <args...>`
    pub fn add(&self, sub: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["add", sub]);
        cmd
    }

    /// Shortcut: `theta rm <subcommand> <args...>`
    pub fn rm(&self, sub: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["rm", sub]);
        cmd
    }

    /// Shortcut: `theta list <subcommand>`
    pub fn list(&self, sub: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["list", sub]);
        cmd
    }

    /// Shortcut: `theta check`
    pub fn check(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("check");
        cmd
    }

    /// Shortcut: `theta sync`
    pub fn sync(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("sync");
        cmd
    }

    /// Shortcut: `theta lock`
    pub fn lock(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("lock");
        cmd
    }

    /// Shortcut: `theta describe`
    pub fn describe(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("describe");
        cmd
    }

    /// Shortcut: `theta register <subcommand>`
    pub fn register(&self, sub: &str) -> Command {
        let mut cmd = self.command();
        cmd.args(["register", sub]);
        cmd
    }

    /// Shortcut: `theta tree`
    pub fn tree(&self) -> Command {
        let mut cmd = self.command();
        cmd.arg("tree");
        cmd
    }
}

/// Snapshot-test a theta CLI command.
///
/// Captures stdout, stderr, and exit code, then compares against
/// an inline snapshot via `insta::assert_snapshot!`.
///
/// ```ignore
/// theta_snapshot!(ctx.filters(), ctx.init().arg("--name").arg("my-agent"), @"
///     success: true
///     exit_code: 0
///     ----- stdout -----
///
///     ----- stderr -----
///     initialized theta.toml
/// ");
/// ```
#[macro_export]
macro_rules! theta_snapshot {
    ($filters:expr, $command:expr, @$snapshot:literal) => {{
        let output = $command.output().expect("failed to execute theta");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        let snapshot = format!(
            "success: {success}\nexit_code: {exit_code}\n----- stdout -----\n{stdout}----- stderr -----\n{stderr}"
        );

        // apply filters to normalize paths, timing, etc.
        let mut filtered = snapshot;
        for (pattern, replacement) in $filters.iter() {
            if let Ok(re) = regex::Regex::new(pattern) {
                filtered = re.replace_all(&filtered, replacement.as_str()).to_string();
            }
        }

        insta::assert_snapshot!(filtered, @$snapshot);
    }};
}
