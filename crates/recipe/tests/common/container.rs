//! Docker container wrapper for integration tests.
//!
//! Uses Fedora as the base image since LevitateOS is Fedora-based.

#![allow(dead_code)]

use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures::StreamExt;
use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};

/// Test environment with a running Docker container.
pub struct TestEnv {
    pub container: ContainerAsync<GenericImage>,
    pub docker: Docker,
    pub container_id: String,
}

impl TestEnv {
    /// Create a new test environment with a Fedora container.
    /// The container comes with curl, git, tar, and common archive tools.
    pub async fn new() -> Self {
        let image = GenericImage::new("fedora", "43")
            .with_cmd(vec![
                "bash",
                "-c",
                "dnf install -y -q curl git tar xz bzip2 zip unzip coreutils ca-certificates && \
                 mkdir -p /tmp/build /usr/local/bin /usr/local/lib /usr/local/share && \
                 while true; do sleep 1; done",
            ]);

        let container = image.start().await.expect("Failed to start container");

        // Wait for dnf to finish
        tokio::time::sleep(Duration::from_secs(5)).await;

        let docker =
            Docker::connect_with_local_defaults().expect("Failed to connect to Docker daemon");
        let container_id = container.id().to_string();

        let env = Self {
            container,
            docker,
            container_id,
        };

        // Wait until tools are installed
        env.wait_for_ready().await;

        env
    }

    /// Wait for the container to be ready (tools installed).
    async fn wait_for_ready(&self) {
        for _ in 0..120 {
            // Check for curl and ca-certificates (Fedora path)
            if self.exec(&["test", "-f", "/etc/pki/tls/certs/ca-bundle.crt"]).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        panic!("Container not ready after 60 seconds");
    }

    /// Execute a command in the container and return stdout.
    pub async fn exec(&self, cmd: &[&str]) -> Result<String, String> {
        let exec = self
            .docker
            .create_exec(
                &self.container_id,
                CreateExecOptions {
                    cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| format!("Failed to create exec: {}", e))?;

        let start_result = self
            .docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| format!("Failed to start exec: {}", e))?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let StartExecResults::Attached { mut output, .. } = start_result {
            while let Some(msg) = output.next().await {
                match msg {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }
        }

        // Check exit code
        let inspect = self
            .docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| format!("Failed to inspect exec: {}", e))?;

        if let Some(exit_code) = inspect.exit_code {
            if exit_code != 0 {
                return Err(format!(
                    "Command {:?} failed with exit code {}: {}",
                    cmd, exit_code, stderr
                ));
            }
        }

        Ok(stdout)
    }

    /// Execute a shell command (passed to bash -c).
    pub async fn shell(&self, cmd: &str) -> Result<String, String> {
        self.exec(&["bash", "-c", cmd]).await
    }

    /// Check if a file exists in the container.
    pub async fn file_exists(&self, path: &str) -> bool {
        self.exec(&["test", "-e", path]).await.is_ok()
    }

    /// Check if a path is a directory.
    pub async fn is_dir(&self, path: &str) -> bool {
        self.exec(&["test", "-d", path]).await.is_ok()
    }

    /// Read a file's contents from the container.
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        self.exec(&["cat", path]).await
    }

    /// Get file permissions as octal (e.g., 755).
    pub async fn file_mode(&self, path: &str) -> Option<u32> {
        let output = self.shell(&format!("stat -c %a {}", path)).await.ok()?;
        u32::from_str_radix(output.trim(), 8).ok()
    }

    /// Get file owner.
    pub async fn file_owner(&self, path: &str) -> Option<String> {
        let output = self.shell(&format!("stat -c %U {}", path)).await.ok()?;
        Some(output.trim().to_string())
    }

    /// Check if a user exists.
    pub async fn user_exists(&self, name: &str) -> bool {
        self.exec(&["id", name]).await.is_ok()
    }

    /// Get user's shell.
    pub async fn user_shell(&self, name: &str) -> Option<String> {
        let passwd = self.read_file("/etc/passwd").await.ok()?;
        passwd
            .lines()
            .find(|l| l.starts_with(&format!("{}:", name)))
            .and_then(|l| l.split(':').last())
            .map(|s| s.to_string())
    }

    /// Get user's UID.
    pub async fn user_uid(&self, name: &str) -> Option<u32> {
        let output = self.exec(&["id", "-u", name]).await.ok()?;
        output.trim().parse().ok()
    }

    /// Create a file in the container.
    pub async fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        // Use base64 to handle special characters
        let encoded = base64_encode(content);
        self.shell(&format!("echo '{}' | base64 -d > {}", encoded, path))
            .await?;
        Ok(())
    }

    /// Create a directory in the container.
    pub async fn mkdir(&self, path: &str) -> Result<(), String> {
        self.exec(&["mkdir", "-p", path]).await?;
        Ok(())
    }
}

fn base64_encode(s: &str) -> String {
    use std::io::Write;
    let mut enc = Vec::new();
    let mut encoder = Base64Encoder::new(&mut enc);
    encoder.write_all(s.as_bytes()).unwrap();
    drop(encoder);
    String::from_utf8(enc).unwrap()
}

// Simple base64 encoder
struct Base64Encoder<W: std::io::Write> {
    writer: W,
    buf: [u8; 3],
    len: usize,
}

impl<W: std::io::Write> Base64Encoder<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            buf: [0; 3],
            len: 0,
        }
    }

    fn flush_buf(&mut self) -> std::io::Result<()> {
        const CHARS: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        if self.len == 0 {
            return Ok(());
        }

        let b0 = self.buf[0];
        let b1 = if self.len > 1 { self.buf[1] } else { 0 };
        let b2 = if self.len > 2 { self.buf[2] } else { 0 };

        let c0 = CHARS[(b0 >> 2) as usize];
        let c1 = CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        let c2 = if self.len > 1 {
            CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize]
        } else {
            b'='
        };
        let c3 = if self.len > 2 {
            CHARS[(b2 & 0x3f) as usize]
        } else {
            b'='
        };

        self.writer.write_all(&[c0, c1, c2, c3])?;
        self.len = 0;
        Ok(())
    }
}

impl<W: std::io::Write> std::io::Write for Base64Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            self.buf[self.len] = byte;
            self.len += 1;
            if self.len == 3 {
                self.flush_buf()?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf()?;
        self.writer.flush()
    }
}

impl<W: std::io::Write> Drop for Base64Encoder<W> {
    fn drop(&mut self) {
        let _ = self.flush_buf();
    }
}
