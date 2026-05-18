//! Resolved settings for the theta CLI.
//!
//! Resolution order (highest wins): CLI flag --> env var --> built-in default.
//!
//! Inspired by uv's settings cascade (uv-settings/src/combine.rs).

use std::path::PathBuf;

pub struct EnvVars;

impl EnvVars {
    pub const THETA_INSTRUCTIONS_DIR: &'static str = "THETA_INSTRUCTIONS_DIR";
    pub const THETA_RULES_DIR: &'static str = "THETA_RULES_DIR";
}

#[derive(Debug, Clone)]
pub struct ThetaSettings {
    pub instructions_dir: PathBuf,
    pub rules_dir: PathBuf,
}

#[derive(Debug, Default)]
pub struct CliOverrides {
    pub instructions_dir: Option<PathBuf>,
    pub rules_dir: Option<PathBuf>,
}

impl ThetaSettings {
    pub fn resolve(cli: &CliOverrides) -> Self {
        Self {
            instructions_dir: resolve_path(
                cli.instructions_dir.as_ref(),
                EnvVars::THETA_INSTRUCTIONS_DIR,
                theta_static::INSTRUCTIONS_DIR,
            ),
            rules_dir: resolve_path(
                cli.rules_dir.as_ref(),
                EnvVars::THETA_RULES_DIR,
                theta_static::RULES_DIR,
            ),
        }
    }

    pub fn rules_path(&self) -> PathBuf {
        self.instructions_dir.join(&self.rules_dir)
    }
}

fn resolve_path(cli: Option<&PathBuf>, env_var: &str, default: &str) -> PathBuf {
    if let Some(p) = cli {
        return p.clone();
    }
    if let Ok(val) = std::env::var(env_var) {
        if !val.is_empty() {
            return PathBuf::from(val);
        }
    }
    PathBuf::from(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_static_constants() {
        let settings = ThetaSettings::resolve(&CliOverrides::default());
        assert_eq!(
            settings.instructions_dir,
            PathBuf::from(theta_static::INSTRUCTIONS_DIR)
        );
        assert_eq!(settings.rules_dir, PathBuf::from(theta_static::RULES_DIR));
    }

    #[test]
    fn cli_overrides_win() {
        let cli = CliOverrides {
            instructions_dir: Some(PathBuf::from("custom/inst")),
            rules_dir: Some(PathBuf::from("custom/rules")),
        };
        let settings = ThetaSettings::resolve(&cli);
        assert_eq!(settings.instructions_dir, PathBuf::from("custom/inst"));
        assert_eq!(settings.rules_dir, PathBuf::from("custom/rules"));
    }

    #[test]
    fn rules_path_joins_dirs() {
        let settings = ThetaSettings::resolve(&CliOverrides::default());
        assert_eq!(settings.rules_path(), PathBuf::from("instructions/rules"));
    }
}
