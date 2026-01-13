//! Common test utilities for QMP interaction

use anyhow::{bail, Result};
use std::time::{Duration, Instant};

use crate::support::qmp::QmpClient;

/// Wait for a QMP socket file to appear
pub fn wait_for_qmp_socket(socket: &str, timeout_secs: u64) -> Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if std::path::Path::new(socket).exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    bail!("QMP socket not created within {timeout_secs}s")
}

/// Send keys via QMP human-monitor-command
pub fn qmp_send_keys(client: &mut QmpClient, text: &str) -> Result<()> {
    for ch in text.chars() {
        let (key, needs_shift) = char_to_qcode(ch);
        let cmd = if needs_shift {
            format!("sendkey shift-{key}")
        } else {
            format!("sendkey {key}")
        };
        let args = serde_json::json!({ "command-line": cmd });
        client.execute("human-monitor-command", Some(args))?;
        std::thread::sleep(Duration::from_millis(30));
    }
    Ok(())
}

/// Send a single key via QMP
pub fn qmp_send_key(client: &mut QmpClient, qcode: &str) -> Result<()> {
    let cmd = format!("sendkey {qcode}");
    let args = serde_json::json!({ "command-line": cmd });
    client.execute("human-monitor-command", Some(args))?;
    Ok(())
}

/// Convert character to QEMU qcode
fn char_to_qcode(ch: char) -> (&'static str, bool) {
    match ch {
        'a'..='z' => {
            let idx = (ch as u8 - b'a') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], false)
        }
        'A'..='Z' => {
            let idx = (ch as u8 - b'A') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], true)
        }
        '0'..='9' => {
            let idx = (ch as u8 - b'0') as usize;
            let keys = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
            (keys[idx], false)
        }
        ' ' => ("spc", false),
        '\n' => ("ret", false),
        '-' => ("minus", false),
        '_' => ("minus", true),
        '/' => ("slash", false),
        '.' => ("dot", false),
        '=' => ("equal", false),
        '+' => ("equal", true),
        '[' => ("bracket_left", false),
        ']' => ("bracket_right", false),
        ';' => ("semicolon", false),
        ':' => ("semicolon", true),
        '\'' => ("apostrophe", false),
        '"' => ("apostrophe", true),
        ',' => ("comma", false),
        '<' => ("comma", true),
        '>' => ("dot", true),
        '?' => ("slash", true),
        '!' => ("1", true),
        '@' => ("2", true),
        '#' => ("3", true),
        '$' => ("4", true),
        '%' => ("5", true),
        '^' => ("6", true),
        '&' => ("7", true),
        '*' => ("8", true),
        '(' => ("9", true),
        ')' => ("0", true),
        _ => ("spc", false),
    }
}
