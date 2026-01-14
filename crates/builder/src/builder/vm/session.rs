//! VM session state management.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const SESSION_FILE: &str = "build/.vm-session.json";

/// VM session state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub pid: u32,
    pub qmp_socket: String,
    pub serial_socket: String,
    pub output_file: String,
    pub started_at: String,
}

impl Session {
    /// Check if the session's process is still running.
    pub fn is_alive(&self) -> bool {
        Path::new(&format!("/proc/{}", self.pid)).exists()
    }
}

/// Check if a session file exists.
pub fn exists() -> bool {
    Path::new(SESSION_FILE).exists()
}

/// Load session from file.
pub fn load() -> Result<Session> {
    let contents = std::fs::read_to_string(SESSION_FILE)
        .context("No VM session found. Run 'vm start' first.")?;
    let session: Session =
        serde_json::from_str(&contents).context("Failed to parse session file")?;
    Ok(session)
}

/// Save session to file.
pub fn save(session: &Session) -> Result<()> {
    let contents = serde_json::to_string_pretty(session)?;
    std::fs::write(SESSION_FILE, contents).context("Failed to write session file")?;
    Ok(())
}

/// Clear session file.
pub fn clear() -> Result<()> {
    if exists() {
        std::fs::remove_file(SESSION_FILE).context("Failed to remove session file")?;
    }
    Ok(())
}
