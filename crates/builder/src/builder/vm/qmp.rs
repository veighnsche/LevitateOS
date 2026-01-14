//! QEMU Machine Protocol (QMP) client.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// VM run state from query-status.
#[derive(Debug, Clone)]
pub enum VmStatus {
    Running,
    Paused,
    Shutdown,
    InMigrate,
    PostMigrate,
    PreLaunch,
    Finish,
}

impl std::fmt::Display for VmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmStatus::Running => write!(f, "running"),
            VmStatus::Paused => write!(f, "paused"),
            VmStatus::Shutdown => write!(f, "shutdown"),
            VmStatus::InMigrate => write!(f, "in_migrate"),
            VmStatus::PostMigrate => write!(f, "post_migrate"),
            VmStatus::PreLaunch => write!(f, "pre_launch"),
            VmStatus::Finish => write!(f, "finish"),
        }
    }
}

/// QMP client for communicating with QEMU.
pub struct QmpClient {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
}

impl QmpClient {
    /// Connect to QMP socket.
    pub fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)
            .with_context(|| format!("Failed to connect to QMP socket: {}", socket_path))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let reader = BufReader::new(stream.try_clone()?);

        Ok(Self { stream, reader })
    }

    /// Perform QMP handshake (read greeting, send capabilities).
    pub fn handshake(&mut self) -> Result<()> {
        // Read greeting
        let greeting = self.read_response()?;
        if greeting.get("QMP").is_none() {
            bail!("Invalid QMP greeting: {:?}", greeting);
        }

        // Send qmp_capabilities
        self.execute("qmp_capabilities", None)?;
        Ok(())
    }

    /// Execute a QMP command.
    pub fn execute(&mut self, command: &str, arguments: Option<Value>) -> Result<Value> {
        let mut cmd = json!({ "execute": command });
        if let Some(args) = arguments {
            cmd["arguments"] = args;
        }

        let cmd_str = serde_json::to_string(&cmd)? + "\n";
        self.stream
            .write_all(cmd_str.as_bytes())
            .context("Failed to send QMP command")?;
        self.stream.flush()?;

        // Read response (skip events)
        loop {
            let response = self.read_response()?;
            if response.get("return").is_some() || response.get("error").is_some() {
                if let Some(error) = response.get("error") {
                    bail!("QMP error: {:?}", error);
                }
                return Ok(response);
            }
            // Skip events, continue reading
        }
    }

    /// Quit QEMU.
    pub fn quit(&mut self) -> Result<()> {
        self.execute("quit", None)?;
        Ok(())
    }

    /// Execute arbitrary HMP command via QMP.
    pub fn human_monitor_command(&mut self, cmd: &str) -> Result<String> {
        let args = json!({
            "command-line": cmd
        });
        let response = self.execute("human-monitor-command", Some(args))?;

        // Extract the return value (HMP command output)
        if let Some(output) = response.get("return").and_then(|v| v.as_str()) {
            Ok(output.to_string())
        } else {
            bail!("No output from HMP command: {}", cmd);
        }
    }

    /// Get VM run state.
    pub fn query_status(&mut self) -> Result<VmStatus> {
        let response = self.execute("query-status", None)?;

        if let Some(status_str) = response.get("return")
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str())
        {
            let status = match status_str {
                "running" => VmStatus::Running,
                "paused" => VmStatus::Paused,
                "shutdown" => VmStatus::Shutdown,
                "inmigrate" => VmStatus::InMigrate,
                "postmigrate" => VmStatus::PostMigrate,
                "prelaunch" => VmStatus::PreLaunch,
                "finish" => VmStatus::Finish,
                _ => bail!("Unknown VM status: {}", status_str),
            };
            Ok(status)
        } else {
            bail!("Invalid query-status response");
        }
    }

    /// Dump physical memory region to file.
    pub fn pmemsave(&mut self, addr: u64, size: u64, filepath: &str) -> Result<()> {
        let args = json!({
            "val": addr,
            "size": size,
            "filename": filepath
        });
        self.execute("pmemsave", Some(args))?;
        Ok(())
    }

    /// Take a screenshot (requires GUI mode).
    pub fn screendump(&mut self, filepath: &str) -> Result<()> {
        let args = json!({
            "filename": filepath
        });
        self.execute("screendump", Some(args))?;
        Ok(())
    }

    /// Reset the VM (like pressing the reset button).
    pub fn system_reset(&mut self) -> Result<()> {
        self.execute("system_reset", None)?;
        Ok(())
    }

    /// Graceful shutdown.
    pub fn system_powerdown(&mut self) -> Result<()> {
        self.execute("system_powerdown", None)?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .context("Failed to read QMP response")?;
        let response: Value =
            serde_json::from_str(&line).context("Failed to parse QMP response")?;
        Ok(response)
    }
}
