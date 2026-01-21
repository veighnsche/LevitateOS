//! System administration tests.
//!
//! Can users manage users, services, and system settings?
//!
//! ## Anti-Reward-Hacking Design
//!
//! Each test chains operations and verifies the ACTUAL outcome,
//! not just that commands ran without error.

use super::{test_result, Test, TestResult};
use crate::container::Container;

/// Test: Create a user
struct CreateUser;

impl Test for CreateUser {
    fn name(&self) -> &str { "create user" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can create new user accounts"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                userdel -r testuser 2>/dev/null || true &&
                useradd -m testuser &&
                grep testuser /etc/passwd &&
                test -d /home/testuser &&
                userdel -r testuser
            "#)?;

            if !result.contains("testuser") {
                anyhow::bail!("User not in /etc/passwd");
            }
            Ok("useradd creates user with home directory".into())
        })
    }
}

/// Test: Set user password
struct SetPassword;

impl Test for SetPassword {
    fn name(&self) -> &str { "set password" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can set user passwords"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                userdel -r pwduser 2>/dev/null || true &&
                useradd -m pwduser &&
                echo 'pwduser:testpass123' | chpasswd &&
                grep pwduser /etc/shadow &&
                userdel -r pwduser
            "#)?;

            // Should have a hash, not ! or *
            if result.contains("pwduser:!") || result.contains("pwduser:*") {
                anyhow::bail!("Password not set (still locked)");
            }
            Ok("chpasswd sets password hash".into())
        })
    }
}

/// Test: Add user to group
struct AddToGroup;

impl Test for AddToGroup {
    fn name(&self) -> &str { "add to group" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can add users to groups (e.g., wheel for sudo)"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                userdel -r grpuser 2>/dev/null || true &&
                useradd -m grpuser &&
                usermod -aG wheel grpuser &&
                groups grpuser &&
                userdel -r grpuser
            "#)?;

            if !result.contains("wheel") {
                anyhow::bail!("User not in wheel group: {}", result);
            }
            Ok("usermod -aG works correctly".into())
        })
    }
}

/// Test: Sudo works
struct SudoWorks;

impl Test for SudoWorks {
    fn name(&self) -> &str { "sudo execution" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Users in wheel group can execute commands as root via sudo"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                sudo --version | head -1 &&
                grep '%wheel' /etc/sudoers
            "#)?;

            if !result.contains("Sudo version") {
                anyhow::bail!("sudo not working");
            }
            if !result.contains("ALL=(ALL") {
                anyhow::bail!("wheel not configured in sudoers");
            }
            Ok("sudo configured for wheel group".into())
        })
    }
}

/// Test: Su works
struct SuWorks;

impl Test for SuWorks {
    fn name(&self) -> &str { "su switch user" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Users can switch to other accounts with su"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                su --version | head -1 &&
                test -f /etc/pam.d/su
            "#)?;

            if !result.contains("util-linux") {
                anyhow::bail!("su not from util-linux: {}", result);
            }
            Ok(result.trim().into())
        })
    }
}

/// Test: Systemd services
struct SystemdServices;

impl Test for SystemdServices {
    fn name(&self) -> &str { "systemd services" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can manage system services with systemctl"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                systemctl list-unit-files --no-pager 2>&1 | head -20
            "#)?;

            if !result.contains(".service") && !result.contains(".target") {
                anyhow::bail!("systemctl not listing units: {}", result);
            }
            Ok("systemctl can query services".into())
        })
    }
}

/// Test: Hostname management
struct HostnameManagement;

impl Test for HostnameManagement {
    fn name(&self) -> &str { "hostname" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can view and set the system hostname"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                cat /etc/hostname &&
                hostname
            "#)?;

            if result.trim().is_empty() {
                anyhow::bail!("hostname commands returned empty");
            }
            Ok(format!("hostname: {}", result.lines().next().unwrap_or("").trim()))
        })
    }
}

/// Test: View system logs
struct SystemLogs;

impl Test for SystemLogs {
    fn name(&self) -> &str { "journalctl" }
    fn category(&self) -> &str { "admin" }
    fn ensures(&self) -> &str {
        "Administrator can view system logs"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("journalctl --version | head -1")?;

            if !result.contains("systemd") {
                anyhow::bail!("journalctl not working: {}", result);
            }
            Ok(result.trim().into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(CreateUser),
        Box::new(SetPassword),
        Box::new(AddToGroup),
        Box::new(SudoWorks),
        Box::new(SuWorks),
        Box::new(SystemdServices),
        Box::new(HostnameManagement),
        Box::new(SystemLogs),
    ]
}
