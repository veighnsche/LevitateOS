use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

#[derive(Serialize)]
struct QmpCommand {
    execute: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<serde_json::Value>,
}

// TEAM_130: Fields event and greeting are used for serde deserialization
// of QMP protocol responses, even if not explicitly read in code.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct QmpResponse {
    #[serde(rename = "return")]
    return_: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    event: Option<String>,
    #[serde(rename = "QMP")]
    greeting: Option<serde_json::Value>,
}

pub struct QmpClient {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl QmpClient {
    pub fn connect(path: &str) -> Result<Self> {
        let stream = UnixStream::connect(path)
            .with_context(|| format!("Failed to connect to QMP socket at {path}"))?;
        let writer = stream.try_clone()?;
        let mut client = Self {
            reader: BufReader::new(stream),
            writer,
        };

        // 1. Read greeting
        client.read_response()?;

        // 2. Negotiate capabilities
        client.execute("qmp_capabilities", None)?;

        Ok(client)
    }

    pub fn execute(
        &mut self,
        cmd: &str,
        args: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let command = QmpCommand {
            execute: cmd.to_string(),
            arguments: args,
        };
        let mut buf = serde_json::to_vec(&command)?;
        buf.push(b'\n');
        self.writer.write_all(&buf)?;

        loop {
            let resp = self.read_response()?;
            if let Some(ret) = resp.return_ {
                return Ok(ret);
            }
            if let Some(err) = resp.error {
                anyhow::bail!("QMP Error: {}", serde_json::to_string(&err)?);
            }
            // Ignore events
        }
    }

    fn read_response(&mut self) -> Result<QmpResponse> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        if line.is_empty() {
            anyhow::bail!("QMP Connection closed by peer");
        }

        let resp: QmpResponse = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse QMP response: {line}"))?;
        Ok(resp)
    }
}
