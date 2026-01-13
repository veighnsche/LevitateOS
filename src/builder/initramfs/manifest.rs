//! Initramfs manifest parser
//!
//! TEAM_474: Parses `initramfs/initramfs.toml` declarative configuration.

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root manifest structure
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub meta: Meta,
    pub layout: Layout,
    #[serde(default)]
    pub binaries: HashMap<String, Binary>,
    #[serde(default)]
    pub symlinks: HashMap<String, String>,
    #[serde(default)]
    pub files: HashMap<String, FileEntry>,
    #[serde(default)]
    pub scripts: HashMap<String, FileEntry>,
    #[serde(default)]
    pub devices: HashMap<String, DeviceEntry>,
}

/// Manifest metadata
#[derive(Debug, Deserialize)]
pub struct Meta {
    pub version: String,
    #[serde(default)]
    pub description: String,
}

/// Directory layout configuration
#[derive(Debug, Deserialize)]
pub struct Layout {
    pub directories: Vec<String>,
}

/// Binary definition
#[derive(Debug, Deserialize)]
pub struct Binary {
    pub source: String,
    pub dest: String,
    #[serde(default = "default_mode")]
    pub mode: String,
}

/// File entry - either from file or inline content
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FileEntry {
    FromFile {
        source: String,
        #[serde(default = "default_mode")]
        mode: String,
    },
    Inline {
        content: String,
        #[serde(default = "default_mode")]
        mode: String,
    },
}

/// Device node entry
#[derive(Debug, Deserialize)]
pub struct DeviceEntry {
    #[serde(rename = "type")]
    pub dev_type: String,
    pub major: u32,
    pub minor: u32,
    #[serde(default = "default_device_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "0644".to_string()
}

fn default_device_mode() -> String {
    "0666".to_string()
}

impl Manifest {
    /// Load manifest from file, substituting variables
    pub fn load(path: &str, arch: &str, base_dir: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path))?;

        // Perform variable substitution
        let content = content
            .replace("${arch}", arch)
            .replace("${root}", &base_dir.to_string_lossy())
            .replace("${toolchain}", "toolchain");

        let manifest: Manifest =
            toml::from_str(&content).with_context(|| "Failed to parse manifest TOML")?;

        Ok(manifest)
    }

    /// Validate all referenced files exist
    pub fn validate(&self, base_dir: &Path) -> Result<()> {
        // Validate binary sources
        for (name, binary) in &self.binaries {
            // Skip commented-out entries (source is empty)
            if binary.source.is_empty() {
                continue;
            }
            let source_path = PathBuf::from(&binary.source);
            if !source_path.exists() {
                return Err(anyhow!(
                    "Binary '{}' source not found: {}",
                    name,
                    source_path.display()
                ));
            }
        }

        // Validate file sources
        for (dest, entry) in &self.files {
            if let FileEntry::FromFile { source, .. } = entry {
                let source_path = base_dir.join("files").join(source);
                if !source_path.exists() {
                    return Err(anyhow!(
                        "File '{}' source not found: {}",
                        dest,
                        source_path.display()
                    ));
                }
            }
        }

        // Validate script sources
        for (dest, entry) in &self.scripts {
            if let FileEntry::FromFile { source, .. } = entry {
                let source_path = base_dir.join(source);
                if !source_path.exists() {
                    return Err(anyhow!(
                        "Script '{}' source not found: {}",
                        dest,
                        source_path.display()
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get total counts for progress reporting
    pub fn get_totals(&self) -> ManifestTotals {
        ManifestTotals {
            directories: self.layout.directories.len(),
            binaries: self.binaries.values().filter(|b| !b.source.is_empty()).count(),
            symlinks: self.symlinks.len(),
            files: self.files.len() + self.scripts.len(),
            devices: self.devices.len(),
        }
    }
}

/// Summary of manifest contents for progress reporting
#[derive(Debug, Clone)]
pub struct ManifestTotals {
    pub directories: usize,
    pub binaries: usize,
    pub symlinks: usize,
    pub files: usize,
    pub devices: usize,
}

impl ManifestTotals {
    pub fn total(&self) -> usize {
        self.directories + self.binaries + self.symlinks + self.files + self.devices
    }
}

/// Parse octal mode string to u32
pub fn parse_mode(mode: &str) -> u32 {
    u32::from_str_radix(mode.trim_start_matches('0'), 8).unwrap_or(0o644)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("0755"), 0o755);
        assert_eq!(parse_mode("0644"), 0o644);
        assert_eq!(parse_mode("755"), 0o755);
        assert_eq!(parse_mode("0666"), 0o666);
    }
}
