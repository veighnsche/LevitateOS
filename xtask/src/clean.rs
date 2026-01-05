use anyhow::Result;
use std::process::Command;

/// Kill any running QEMU instances
pub fn kill_qemu() -> Result<()> {
    println!("🔪 Killing QEMU instances...");
    let status = Command::new("pkill").args(["-f", "qemu-system-aarch64"]).status()?;
    if status.success() {
        println!("✅ QEMU processes killed.");
    } else {
        println!("ℹ️  No QEMU processes found.");
    }
    // Also kill websockify if running
    let _ = Command::new("pkill").args(["-f", "websockify"]).status();
    // Remove QMP socket
    if std::path::Path::new("./qmp.sock").exists() {
        let _ = std::fs::remove_file("./qmp.sock");
        println!("✅ Removed qmp.sock");
    }
    Ok(())
}

pub fn clean() -> Result<()> {
    println!("🧹 Cleaning...");
    kill_qemu()?;
    Ok(())
}
