//! systemd-nspawn container management for rootfs testing.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

/// Manages a systemd-nspawn container for testing.
pub struct Container {
    /// Path to the rootfs directory
    rootfs: PathBuf,
    /// Whether we created the rootfs (and should clean it up)
    owned: bool,
}

impl Container {
    /// Create a container from an existing rootfs directory.
    pub fn from_dir(rootfs: impl AsRef<Path>) -> Result<Self> {
        let rootfs = rootfs.as_ref().to_path_buf();
        if !rootfs.exists() {
            bail!("Rootfs directory does not exist: {}", rootfs.display());
        }
        Ok(Self { rootfs, owned: false })
    }

    /// Create a container by extracting a tarball.
    pub fn from_tarball(tarball: impl AsRef<Path>, target_dir: impl AsRef<Path>) -> Result<Self> {
        let tarball = tarball.as_ref();
        let target = target_dir.as_ref();

        if !tarball.exists() {
            bail!("Tarball not found: {}", tarball.display());
        }

        // Clean up existing directory
        if target.exists() {
            std::fs::remove_dir_all(target)
                .context("Failed to remove existing rootfs directory")?;
        }
        std::fs::create_dir_all(target)?;

        // Extract tarball
        let status = Command::new("sudo")
            .args(["tar", "-xJf"])
            .arg(tarball)
            .arg("-C")
            .arg(target)
            .status()
            .context("Failed to extract tarball")?;

        if !status.success() {
            bail!("Failed to extract tarball");
        }

        Ok(Self {
            rootfs: target.to_path_buf(),
            owned: true,
        })
    }

    /// Execute a command in the container and return the output.
    ///
    /// Note: Each exec() is a separate nspawn invocation. To verify state
    /// persists across commands, chain them with && in a single exec() call.
    /// This mirrors real user behavior and prevents "reward hacking" where
    /// individual commands pass but the end-to-end flow doesn't work.
    pub fn exec(&self, command: &str) -> Result<Output> {
        let output = Command::new("sudo")
            .args([
                "systemd-nspawn",
                "-D",
                self.rootfs.to_str().unwrap(),
                "--pipe",
                "--volatile=no",  // Don't use tmpfs overlays - persist changes
                "-q",             // Quiet mode - reduce nspawn noise
                "/bin/bash",
                "-c",
                command,
            ])
            .output()
            .context("Failed to execute command in container")?;

        Ok(output)
    }

    /// Execute a command and return stdout as string (fails if command fails).
    pub fn exec_ok(&self, command: &str) -> Result<String> {
        let output = self.exec(command)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Command failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute a command and return (success, stdout, stderr).
    pub fn exec_capture(&self, command: &str) -> Result<(bool, String, String)> {
        let output = self.exec(command)?;
        Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Check if a binary exists in the container.
    pub fn has_binary(&self, binary: &str) -> Result<bool> {
        let output = self.exec(&format!("which {} 2>/dev/null", binary))?;
        Ok(output.status.success())
    }

    /// Get the version of a binary.
    pub fn binary_version(&self, binary: &str, version_flag: &str) -> Result<String> {
        let output = self.exec_ok(&format!("{} {} 2>&1 | head -1", binary, version_flag))?;
        Ok(output.trim().to_string())
    }

    /// Boot the container with systemd (for service tests).
    /// Returns after reaching multi-user.target or timeout.
    pub fn boot_systemd(&self, timeout: Duration) -> Result<()> {
        let timeout_secs = timeout.as_secs();
        let status = Command::new("sudo")
            .args([
                "timeout",
                &timeout_secs.to_string(),
                "systemd-nspawn",
                "-b",
                "-D",
                self.rootfs.to_str().unwrap(),
            ])
            .status()
            .context("Failed to boot container")?;

        // Timeout exit code 124 is expected (we don't want it to run forever)
        if !status.success() && status.code() != Some(124) {
            bail!("Container boot failed with status: {:?}", status.code());
        }

        Ok(())
    }

    /// Get the rootfs path.
    pub fn rootfs(&self) -> &Path {
        &self.rootfs
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        if self.owned {
            // Clean up the rootfs we created
            let _ = Command::new("sudo")
                .args(["rm", "-rf"])
                .arg(&self.rootfs)
                .status();
        }
    }
}
