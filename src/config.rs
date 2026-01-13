//! Configuration file support
//!
//! Reads xtask.toml from project root to configure test behavior,
//! especially golden file ratings (gold vs silver).

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "xtask.toml";

/// Golden file rating determines how test failures are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GoldenRating {
    /// Gold files must match exactly. Test fails on mismatch.
    #[default]
    Gold,

    /// Silver files auto-update on every run. Test always passes but shows diff.
    Silver,
}

#[derive(Debug, Deserialize, Default)]
pub struct XtaskConfig {
    #[serde(default)]
    pub golden_files: HashMap<String, GoldenRating>,
}

impl XtaskConfig {
    /// Load config from xtask.toml (or use defaults if file doesn't exist)
    pub fn load() -> Result<Self> {
        let config_path = Path::new(CONFIG_FILE);

        if !config_path.exists() {
            return Ok(XtaskConfig::default());
        }

        let content =
            fs::read_to_string(config_path).context(format!("Failed to read {CONFIG_FILE}"))?;

        toml::from_str(&content).context(format!("Failed to parse {CONFIG_FILE}"))
    }

    /// Get the rating for a golden file (default: Gold)
    pub fn golden_rating(&self, file_path: &str) -> GoldenRating {
        self.golden_files
            .get(file_path)
            .copied()
            .unwrap_or(GoldenRating::Gold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = XtaskConfig::load().unwrap();
        assert_eq!(config.golden_rating("nonexistent.txt"), GoldenRating::Gold);
    }
}
