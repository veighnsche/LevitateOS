use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn check_preflight(arch: &str) -> Result<()> {
    println!("🔍 Running preflight checks for {arch}...");

    let common_tools = ["cargo", "rustup", "find", "cpio", "dd", "curl", "git"];
    let x86_64_tools = ["xorriso", "sfdisk", "mformat", "mcopy", "mdir"];
    let aarch64_tools = [
        "aarch64-linux-gnu-objcopy",
        "sfdisk",
        "mformat",
        "mcopy",
        "mdir",
    ];

    let mut missing = Vec::new();

    for tool in &common_tools {
        if !check_tool(tool) {
            missing.push(*tool);
        }
    }

    if arch == "x86_64" {
        for tool in &x86_64_tools {
            if !check_tool(tool) {
                missing.push(*tool);
            }
        }
    } else if arch == "aarch64" {
        for tool in &aarch64_tools {
            if !check_tool(tool) {
                missing.push(*tool);
            }
        }
    }

    if !missing.is_empty() {
        println!("\n❌ Preflight FAILED. Missing tools:");
        for tool in &missing {
            println!("   - {tool}");
        }

        println!("\n💡 Tip: Install missing dependencies using:");
        if arch == "x86_64" {
            println!("   sudo apt-get install mtools xorriso curl cpio fdisk git");
        } else {
            println!("   sudo apt-get install mtools curl cpio fdisk git gcc-aarch64-linux-gnu");
        }

        bail!("Missing dependencies for {arch}");
    }

    // Check Rust targets
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => unreachable!(),
    };

    if !check_rust_target(target)? {
        println!("\n❌ Preflight FAILED. Missing Rust target: {target}");
        println!("💡 Tip: Install it using: rustup target add {target}");
        bail!("Missing Rust target: {target}");
    }

    if !check_rust_component("rust-src")? {
        println!("\n❌ Preflight FAILED. Missing Rust component: rust-src");
        println!("💡 Tip: Install it using: rustup component add rust-src");
        bail!("Missing Rust component: rust-src");
    }

    println!("✅ Preflight checks PASSED for {arch}\n");
    Ok(())
}

fn check_tool(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn check_rust_target(target: &str) -> Result<bool> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("Failed to run rustup")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|l| l.trim() == target))
}

fn check_rust_component(component: &str) -> Result<bool> {
    let output = Command::new("rustup")
        .args(["component", "list", "--installed"])
        .output()
        .context("Failed to run rustup")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|l| l.trim().starts_with(component)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tool_cargo_exists() {
        // cargo should always exist in dev environment
        assert!(check_tool("cargo"));
    }

    #[test]
    fn test_check_tool_nonexistent() {
        assert!(!check_tool("this-tool-definitely-does-not-exist-12345"));
    }

    #[test]
    fn test_check_rust_target() {
        // x86_64-unknown-linux-gnu is the host target, should always be available
        let result = check_rust_target("x86_64-unknown-linux-gnu");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rust_component_rust_src() {
        // rust-src should be installed for kernel dev
        let result = check_rust_component("rust-src");
        assert!(result.is_ok());
    }
}
