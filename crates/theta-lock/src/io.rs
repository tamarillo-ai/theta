use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::LockError;
use crate::types::LockFile;

/// Read and parse a `theta.lock` file from disk.
pub fn read_lock(path: &Path) -> Result<LockFile, LockError> {
    let content = fs_err::read_to_string(path).map_err(|e| LockError::Read {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| LockError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Atomically write a `theta.lock` file via temp file + rename.
pub fn write_lock(path: &Path, lock: &LockFile) -> Result<(), LockError> {
    let content = toml::to_string_pretty(lock).map_err(|e| LockError::Serialize { source: e })?;

    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let mut tmp = NamedTempFile::new_in(dir).map_err(|e| LockError::Write {
        path: path.to_path_buf(),
        source: e,
    })?;
    tmp.write_all(content.as_bytes())
        .map_err(|e| LockError::Write {
            path: path.to_path_buf(),
            source: e,
        })?;
    tmp.persist(path).map_err(|e| LockError::Write {
        path: path.to_path_buf(),
        source: e.error,
    })?;
    Ok(())
}
