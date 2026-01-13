#![allow(dead_code)]
//! Configuration file support for xtask
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GoldenRating {
    /// Gold files must match exactly. Test fails on mismatch.
    /// User must explicitly use --update to refresh.
    #[default]
    Gold,

    /// Silver files auto-update on every run.
    /// Test always passes but shows the diff.
    /// Use during active development when behavior changes frequently.
    Silver,
}

#[derive(Debug, Deserialize)]
pub struct XtaskConfig {
    #[serde(default)]
    pub golden_files: HashMap<String, GoldenRating>,

    #[serde(default)]
    pub test: TestConfig,

    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub qemu: QemuConfig,
}

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    #[serde(default = "default_behavior_timeout")]
    pub behavior_timeout: u64,

    #[serde(default = "default_vm_timeout")]
    pub vm_timeout: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            behavior_timeout: default_behavior_timeout(),
            vm_timeout: default_vm_timeout(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize)]
pub struct QemuConfig {
    #[serde(default = "default_profile")]
    pub default_profile_aarch64: String,

    #[serde(default = "default_profile")]
    pub default_profile_x86_64: String,
}

impl Default for QemuConfig {
    fn default() -> Self {
        QemuConfig {
            default_profile_aarch64: default_profile(),
            default_profile_x86_64: default_profile(),
        }
    }
}

fn default_behavior_timeout() -> u64 {
    15
}
fn default_vm_timeout() -> u64 {
    30
}
fn default_profile() -> String {
    "default".to_string()
}

impl XtaskConfig {
    /// Load config from xtask.toml (or use defaults if file doesn't exist)
    pub fn load() -> Result<Self> {
        let config_path = Path::new(CONFIG_FILE);

        if !config_path.exists() {
            // No config file - use defaults
            return Ok(XtaskConfig {
                golden_files: HashMap::new(),
                test: TestConfig::default(),
                build: BuildConfig::default(),
                qemu: QemuConfig::default(),
            });
        }

        let content =
            fs::read_to_string(config_path).context(format!("Failed to read {CONFIG_FILE}"))?;

        let config: XtaskConfig =
            toml::from_str(&content).context(format!("Failed to parse {CONFIG_FILE}"))?;

        Ok(config)
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
        assert_eq!(config.test.behavior_timeout, 15);
        assert_eq!(config.test.vm_timeout, 30);
        assert_eq!(config.golden_rating("nonexistent.txt"), GoldenRating::Gold);
    }
}
