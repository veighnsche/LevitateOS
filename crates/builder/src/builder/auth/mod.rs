//! Authentication configuration module.
//!
//! Handles PAM, user accounts, and NSS configuration.
//! Can be tested in isolation without running a full VM.

mod nss;
mod pam;
mod users;

use anyhow::Result;
use std::path::Path;

pub use nss::NssConfig;
pub use pam::PamConfig;
pub use users::{User, UserConfig};

/// Complete authentication configuration.
pub struct AuthConfig {
    pub users: UserConfig,
    pub pam: PamConfig,
    pub nss: NssConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            users: UserConfig::default(),
            pam: PamConfig::default(),
            nss: NssConfig::default(),
        }
    }
}

impl AuthConfig {
    /// Write all authentication config files to the given root directory.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        self.users.write_to(root)?;
        self.pam.write_to(root)?;
        self.nss.write_to(root)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_writes_files() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create required directories
        std::fs::create_dir_all(root.join("etc/pam.d")).unwrap();
        std::fs::create_dir_all(root.join("etc/security")).unwrap();

        let config = AuthConfig::default();
        config.write_to(root).unwrap();

        // Verify files exist
        assert!(root.join("etc/passwd").exists());
        assert!(root.join("etc/shadow").exists());
        assert!(root.join("etc/group").exists());
        assert!(root.join("etc/nsswitch.conf").exists());
        assert!(root.join("etc/pam.d/login").exists());
    }

    #[test]
    fn test_password_hash_format() {
        let user = User::new("test", 1000, 1000, "Test User", "/home/test", "/bin/sh")
            .with_password("testpass");

        let shadow_line = user.shadow_line();
        assert!(shadow_line.starts_with("test:$6$"));
        assert!(shadow_line.contains(":19740:0:99999:7:::"));
    }

    #[test]
    fn test_user_lookup_in_passwd() {
        let config = UserConfig::default();
        let passwd = config.passwd_content();

        assert!(passwd.contains("root:x:0:0:"));
        assert!(passwd.contains("live:x:1000:1000:"));
    }
}
