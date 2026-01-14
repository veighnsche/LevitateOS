//! User account configuration.
//!
//! Handles /etc/passwd, /etc/shadow, /etc/group, /etc/gshadow.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// A user account.
#[derive(Clone)]
pub struct User {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub home: String,
    pub shell: String,
    pub password_hash: Option<String>,
}

impl User {
    pub fn new(name: &str, uid: u32, gid: u32, gecos: &str, home: &str, shell: &str) -> Self {
        Self {
            name: name.to_string(),
            uid,
            gid,
            gecos: gecos.to_string(),
            home: home.to_string(),
            shell: shell.to_string(),
            password_hash: None,
        }
    }

    /// Set password (will be hashed with SHA-512).
    #[cfg(test)]
    #[must_use]
    pub fn with_password(mut self, password: &str) -> Self {
        self.password_hash = Some(hash_password(password));
        self
    }

    /// Set pre-computed password hash.
    #[must_use]
    pub fn with_hash(mut self, hash: &str) -> Self {
        self.password_hash = Some(hash.to_string());
        self
    }

    /// Generate /etc/passwd line.
    pub fn passwd_line(&self) -> String {
        format!(
            "{}:x:{}:{}:{}:{}:{}\n",
            self.name, self.uid, self.gid, self.gecos, self.home, self.shell
        )
    }

    /// Generate /etc/shadow line.
    pub fn shadow_line(&self) -> String {
        let hash = self.password_hash.as_deref().unwrap_or("!");
        format!("{}:{}:19740:0:99999:7:::\n", self.name, hash)
    }
}

/// A user group.
#[derive(Clone)]
pub struct Group {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
}

impl Group {
    pub fn new(name: &str, gid: u32) -> Self {
        Self {
            name: name.to_string(),
            gid,
            members: Vec::new(),
        }
    }

    pub fn with_members(mut self, members: &[&str]) -> Self {
        self.members = members.iter().map(std::string::ToString::to_string).collect();
        self
    }

    /// Generate /etc/group line.
    pub fn group_line(&self) -> String {
        format!("{}:x:{}:{}\n", self.name, self.gid, self.members.join(","))
    }

    /// Generate /etc/gshadow line.
    pub fn gshadow_line(&self) -> String {
        format!("{}:!::{}\n", self.name, self.members.join(","))
    }
}

/// User configuration for the system.
pub struct UserConfig {
    pub users: Vec<User>,
    pub groups: Vec<Group>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            users: vec![
                User::new("root", 0, 0, "root", "/root", "/bin/brush")
                    .with_hash("$6$saltsalt$bAY90rAsHhyx.bxmKP9FE5UF4jP1iWgjV0ltM6ZJxfYkiIaCExjBZIbfmqmZEWoR65aM.1nFvG7fF3gYOjHpM."),
                User::new("live", 1000, 1000, "Live User", "/home/live", "/bin/brush")
                    .with_hash("$6$saltsalt$lnz8B.EkP7gx/SsOOLQAcEU/F.7k3CE1I9HTM5hraWcxPafsvSqaJ9s7btu0bk1OOGYbFIG93bLmjZ/qM89J/1"),
                User::new("nobody", 65534, 65534, "Nobody", "/", "/sbin/nologin"),
            ],
            groups: vec![
                Group::new("root", 0),
                Group::new("wheel", 10).with_members(&["root", "live"]),
                Group::new("live", 1000),
                Group::new("nobody", 65534),
            ],
        }
    }
}

impl UserConfig {
    /// Generate /etc/passwd content.
    pub fn passwd_content(&self) -> String {
        self.users.iter().map(User::passwd_line).collect()
    }

    /// Generate /etc/shadow content.
    pub fn shadow_content(&self) -> String {
        self.users.iter().map(User::shadow_line).collect()
    }

    /// Generate /etc/group content.
    pub fn group_content(&self) -> String {
        self.groups.iter().map(Group::group_line).collect()
    }

    /// Generate /etc/gshadow content.
    pub fn gshadow_content(&self) -> String {
        self.groups.iter().map(Group::gshadow_line).collect()
    }

    /// Generate /etc/login.defs content.
    #[must_use]
    pub fn login_defs_content() -> String {
        "# Login configuration\n\
         MAIL_DIR /var/mail\n\
         ENV_PATH /usr/local/bin:/usr/bin:/bin\n\
         ENV_SUPATH /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\n\
         ENCRYPT_METHOD SHA512\n\
         SHA_CRYPT_MIN_ROUNDS 5000\n\
         SHA_CRYPT_MAX_ROUNDS 5000\n"
            .to_string()
    }

    /// Generate /etc/securetty content.
    #[must_use]
    pub fn securetty_content() -> String {
        "console\ntty1\ntty2\ntty3\ntty4\nttyS0\n".to_string()
    }

    /// Write user config files to the given root directory.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        std::fs::write(root.join("etc/passwd"), self.passwd_content())?;
        std::fs::write(root.join("etc/shadow"), self.shadow_content())?;
        std::fs::write(root.join("etc/group"), self.group_content())?;
        std::fs::write(root.join("etc/gshadow"), self.gshadow_content())?;
        std::fs::write(root.join("etc/login.defs"), Self::login_defs_content())?;
        std::fs::write(root.join("etc/securetty"), Self::securetty_content())?;

        // Set restrictive permissions on shadow file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                root.join("etc/shadow"),
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        // Create home directories with skeleton files
        for user in &self.users {
            // Skip system users with no real home (nobody, etc.)
            if user.home == "/" {
                continue;
            }

            let home_path = root.join(user.home.trim_start_matches('/'));
            std::fs::create_dir_all(&home_path)?;

            // Create standard XDG directories (skip for root)
            if user.uid != 0 {
                for dir in ["Desktop", "Documents", "Downloads", "Music", "Pictures", "Videos"] {
                    std::fs::create_dir_all(home_path.join(dir))?;
                }
            }

            // Write skeleton files
            std::fs::write(
                home_path.join(".profile"),
                "# ~/.profile: executed by login shell\n\
                 if [ -f /etc/profile ]; then\n\
                     . /etc/profile\n\
                 fi\n",
            )?;

            std::fs::write(
                home_path.join(".bashrc"),
                "# ~/.bashrc: executed by interactive shells\n\
                 \n\
                 # Source /etc/profile if it exists\n\
                 if [ -f /etc/profile ]; then\n\
                     . /etc/profile\n\
                 fi\n\
                 \n\
                 # Prompt: user@host:dir$ (or # for root)\n\
                 # Root uses full path, users get ~ abbreviation\n\
                 if [ \"$(id -u)\" -eq 0 ]; then\n\
                     PS1='\\u@\\h:$(pwd)# '\n\
                 else\n\
                     PS1='\\u@\\h:\\w\\$ '\n\
                 fi\n\
                 \n\
                 # Aliases\n\
                 alias ls='ls --color=auto'\n\
                 alias ll='ls -la'\n",
            )?;

            // Set ownership using chown command (works during fakeroot build)
            #[cfg(unix)]
            if let Some(path_str) = home_path.to_str() {
                let _ = Command::new("chown")
                    .args(["-R", &format!("{}:{}", user.uid, user.gid), path_str])
                    .output();
            }
        }

        Ok(())
    }
}

/// Hash a password using SHA-512 with a fixed salt.
#[cfg(test)]
fn hash_password(password: &str) -> String {
    // Use openssl to generate hash
    let output = Command::new("openssl")
        .args(["passwd", "-6", "-salt", "saltsalt", password])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => {
            // Fallback: return locked password
            "!".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_passwd_line() {
        let user = User::new("testuser", 1001, 1001, "Test", "/home/test", "/bin/bash");
        assert_eq!(
            user.passwd_line(),
            "testuser:x:1001:1001:Test:/home/test:/bin/bash\n"
        );
    }

    #[test]
    fn test_user_shadow_line_locked() {
        let user = User::new("testuser", 1001, 1001, "Test", "/home/test", "/bin/bash");
        assert_eq!(user.shadow_line(), "testuser:!:19740:0:99999:7:::\n");
    }

    #[test]
    fn test_group_line() {
        let group = Group::new("developers", 1001).with_members(&["alice", "bob"]);
        assert_eq!(group.group_line(), "developers:x:1001:alice,bob\n");
    }

    #[test]
    fn test_default_has_root_and_live() {
        let config = UserConfig::default();
        let passwd = config.passwd_content();
        assert!(passwd.contains("root:x:0:0:"));
        assert!(passwd.contains("live:x:1000:1000:"));
    }
}
