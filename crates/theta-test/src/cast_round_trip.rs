//! Fixture and path helpers for cast e2e tests.
//!
//! Extends `TestContext` with methods to load fixtures, write inline files,
//! and snapshot workspace state.

use std::path::{Path, PathBuf};

use crate::TestContext;

impl TestContext {
    /// Copy a fixture directory into the temp workspace.
    ///
    /// `fixture_rel` is relative to `test/cast-fixtures/`
    /// (e.g. `"copilot/rules/glob-patterns"`).
    #[must_use]
    pub fn with_fixture(self, fixture_rel: &str) -> Self {
        let fixture_dir = self.fixtures_dir().join("cast-fixtures").join(fixture_rel);
        assert!(
            fixture_dir.is_dir(),
            "fixture directory does not exist: {}",
            fixture_dir.display()
        );
        copy_dir_recursive(&fixture_dir, self.temp_dir.path());
        self
    }

    /// Write inline files into the temp workspace.
    #[must_use]
    pub fn with_files(self, files: &[(&str, &str)]) -> Self {
        for (rel, content) in files {
            let path = self.temp_dir.path().join(rel);
            if let Some(parent) = path.parent() {
                fs_err::create_dir_all(parent).unwrap();
            }
            fs_err::write(&path, content).unwrap();
        }
        self
    }

    /// Absolute path to a file in the workspace.
    pub fn path(&self, rel: &str) -> PathBuf {
        self.temp_dir.path().join(rel)
    }

    /// Read a file from the workspace, panics if missing.
    pub fn read_file(&self, rel: &str) -> String {
        let p = self.path(rel);
        fs_err::read_to_string(&p).unwrap_or_else(|e| panic!("failed to read {}: {e}", p.display()))
    }

    /// Check if a file exists in the workspace.
    pub fn exists(&self, rel: &str) -> bool {
        self.path(rel).exists()
    }

    /// Snapshot the current workspace into a sibling `original/` directory.
    ///
    /// Call before running commands so assertions can compare input vs output.
    /// Returns the path to the snapshot.
    pub fn snapshot_original(&self) -> PathBuf {
        let original = self.temp_dir.path().parent().unwrap().join("original");
        copy_dir_recursive(self.temp_dir.path(), &original);
        original
    }
}

pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs_err::create_dir_all(dst).unwrap();
    for entry in fs_err::read_dir(src).unwrap().flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs_err::copy(&src_path, &dst_path).unwrap();
        }
    }
}
