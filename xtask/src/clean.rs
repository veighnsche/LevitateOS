use anyhow::Result;
use std::process::Command;

pub fn clean() -> Result<()> {
    println!("🧹 Cleaning...");
    // Kill QEMU/Websockify
    let _ = Command::new("pkill").args(["-f", "qemu-system-aarch64"]).status();
    let _ = Command::new("pkill").args(["-f", "websockify"]).status();
    // Remove sockets
    if std::path::Path::new("./qmp.sock").exists() {
        let _ = std::fs::remove_file("./qmp.sock");
    }
    Ok(())
}
