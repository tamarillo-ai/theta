use std::path::PathBuf;

use etcetera::BaseStrategy;

pub fn home_dir() -> Option<PathBuf> {
    etcetera::home_dir().ok()
}

pub fn config_dir() -> Option<PathBuf> {
    etcetera::base_strategy::choose_base_strategy()
        .ok()
        .map(|dirs| dirs.config_dir().join("theta"))
}

pub fn cache_dir() -> Option<PathBuf> {
    etcetera::base_strategy::choose_base_strategy()
        .ok()
        .map(|dirs| dirs.cache_dir().join("theta"))
}

pub fn data_dir() -> Option<PathBuf> {
    std::env::var("THETA_DATA_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            etcetera::base_strategy::choose_base_strategy()
                .ok()
                .map(|dirs| dirs.data_dir().join("theta"))
        })
}

pub fn store_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("store"))
}

pub fn git_cache_dir() -> Option<PathBuf> {
    cache_dir().map(|d| d.join("git"))
}
