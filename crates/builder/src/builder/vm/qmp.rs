//! QEMU Machine Protocol (QMP) client.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

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
