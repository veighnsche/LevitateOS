//! PAM (Pluggable Authentication Modules) configuration.
//!
//! Handles /etc/pam.d/* and /etc/security/*.

use anyhow::Result;
use std::path::Path;

/// PAM configuration.
pub struct PamConfig {
    /// Use `pam_unix.so` for real authentication (requires shadow file).
    pub use_unix: bool,
    /// Allow null passwords (empty password field).
    pub allow_nullok: bool,
}

impl Default for PamConfig {
    fn default() -> Self {
        Self {
            use_unix: true,
            allow_nullok: true,
        }
    }
}

impl PamConfig {
    /// Create a permissive config (always allows login - for testing).
    #[cfg(test)]
    pub fn permissive() -> Self {
        Self {
            use_unix: false,
            allow_nullok: true,
        }
    }

    /// Generate /etc/pam.d/login content.
    pub fn login_content(&self) -> String {
        if self.use_unix {
            let nullok = if self.allow_nullok { " nullok" } else { "" };
            format!(
                "auth       required     pam_unix.so{nullok}\n\
                 account    required     pam_unix.so\n\
                 password   required     pam_unix.so sha512 shadow{nullok}\n\
                 session    required     pam_unix.so\n"
            )
        } else {
            // Permissive: always allow
            "auth       required     pam_permit.so\n\
             account    required     pam_permit.so\n\
             password   required     pam_permit.so\n\
             session    required     pam_permit.so\n"
                .to_string()
        }
    }

    /// Generate /etc/pam.d/other content (fallback for unconfigured services).
    pub fn other_content(&self) -> String {
        // Same as login for now
        self.login_content()
    }

    /// Write PAM config files to the given root directory.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        let pam_d = root.join("etc/pam.d");
        let security = root.join("etc/security");

        std::fs::create_dir_all(&pam_d)?;
        std::fs::create_dir_all(&security)?;

        // /etc/pam.d/login
        std::fs::write(pam_d.join("login"), self.login_content())?;

        // /etc/pam.d/other (fallback)
        std::fs::write(pam_d.join("other"), self.other_content())?;

        // /etc/security/opasswd (required by pam_unix.so)
        std::fs::write(security.join("opasswd"), "")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_uses_pam_unix() {
        let config = PamConfig::default();
        let content = config.login_content();
        assert!(content.contains("pam_unix.so"));
        assert!(content.contains("nullok"));
    }

    #[test]
    fn test_permissive_uses_pam_permit() {
        let config = PamConfig::permissive();
        let content = config.login_content();
        assert!(content.contains("pam_permit.so"));
        assert!(!content.contains("pam_unix.so"));
    }

    #[test]
    fn test_strict_no_nullok() {
        let config = PamConfig {
            use_unix: true,
            allow_nullok: false,
        };
        let content = config.login_content();
        assert!(content.contains("pam_unix.so"));
        assert!(!content.contains("nullok"));
    }
}
