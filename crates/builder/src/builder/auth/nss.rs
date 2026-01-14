//! NSS (Name Service Switch) configuration.
//!
//! Handles /etc/nsswitch.conf - tells glibc where to look up users/groups.

use anyhow::Result;
use std::path::Path;

/// NSS configuration.
pub struct NssConfig {
    /// Sources for passwd lookups (usually "files").
    pub passwd: Vec<String>,
    /// Sources for group lookups (usually "files").
    pub group: Vec<String>,
    /// Sources for shadow lookups (usually "files").
    pub shadow: Vec<String>,
}

impl Default for NssConfig {
    fn default() -> Self {
        Self {
            passwd: vec!["files".to_string()],
            group: vec!["files".to_string()],
            shadow: vec!["files".to_string()],
        }
    }
}

impl NssConfig {
    /// Generate /etc/nsswitch.conf content.
    pub fn content(&self) -> String {
        format!(
            "passwd: {}\n\
             group: {}\n\
             shadow: {}\n",
            self.passwd.join(" "),
            self.group.join(" "),
            self.shadow.join(" ")
        )
    }

    /// Write NSS config to the given root directory.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        std::fs::write(root.join("etc/nsswitch.conf"), self.content())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_uses_files() {
        let config = NssConfig::default();
        let content = config.content();
        assert!(content.contains("passwd: files"));
        assert!(content.contains("group: files"));
        assert!(content.contains("shadow: files"));
    }
}
