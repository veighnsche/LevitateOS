//! Source repository management (fetch, cache, clean).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const VENDOR_DIR: &str = "vendor";

/// Source definitions: (name, git_url, branch/tag)
pub const SOURCES: &[(&str, &str, &str)] = &[
    ("linux", "https://github.com/torvalds/linux.git", "v6.18"),
    ("systemd", "https://github.com/systemd/systemd.git", "v259"),
    ("uutils", "https://github.com/uutils/coreutils.git", "0.5.0"),
    ("sudo-rs", "https://github.com/memorysafety/sudo-rs.git", "v0.2.11"),
    ("brush", "https://github.com/reubeno/brush.git", "main"),
];

/// Check if a source is cached.
pub fn is_cached(name: &str) -> bool {
    PathBuf::from(VENDOR_DIR).join(name).exists()
}

/// Find source definition by name.
pub fn find_source(name: &str) -> Option<(&'static str, &'static str, &'static str)> {
    SOURCES.iter().find(|(n, _, _)| *n == name).copied()
}

/// Get the path to a vendor source, failing if not cached.
pub fn require(name: &str) -> Result<PathBuf> {
    let path = PathBuf::from(VENDOR_DIR).join(name);
    if !path.exists() {
        bail!("{} not found. Run: builder fetch {}", name, name);
    }
    Ok(path)
}

/// Fetch a single source repository.
pub fn fetch(name: &str) -> Result<()> {
    let (_, url, tag) = find_source(name)
        .ok_or_else(|| anyhow::anyhow!("Unknown source: {}", name))?;

    let dest = PathBuf::from(VENDOR_DIR).join(name);

    if dest.exists() {
        println!("{} already cached at {}", name, dest.display());
        return Ok(());
    }

    std::fs::create_dir_all(VENDOR_DIR)?;

    println!("Fetching {} from {} @ {}...", name, url, tag);

    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            tag,
            url,
            dest.to_str().unwrap(),
        ])
        .status()
        .context("Failed to run git clone")?;

    if !status.success() {
        bail!("git clone failed for {}", name);
    }

    let size = dir_size(&dest)?;
    println!(
        "  Cached: {} ({:.1} MB)",
        dest.display(),
        size as f64 / 1_000_000.0
    );

    Ok(())
}

/// Fetch all source repositories.
pub fn fetch_all() -> Result<()> {
    println!("=== Fetching all sources ===\n");
    for (name, _, _) in SOURCES {
        fetch(name)?;
    }
    Ok(())
}

/// Show cache status for all sources.
pub fn status() -> Result<()> {
    println!("Cache Status:\n");

    let mut total_size: u64 = 0;
    let mut cached = 0;

    for (name, url, tag) in SOURCES {
        let path = PathBuf::from(VENDOR_DIR).join(name);
        if path.exists() {
            let size = dir_size(&path)?;
            total_size += size;
            cached += 1;
            println!("  {:12} [cached] {:.1} MB", name, size as f64 / 1_000_000.0);
        } else {
            println!("  {:12} [missing] {} @ {}", name, url, tag);
        }
    }

    println!();
    println!(
        "  Total: {}/{} cached ({:.1} MB)",
        cached,
        SOURCES.len(),
        total_size as f64 / 1_000_000.0
    );

    Ok(())
}

/// Clean cached sources.
pub fn clean(name: Option<&str>) -> Result<()> {
    if let Some(name) = name {
        let path = PathBuf::from(VENDOR_DIR).join(name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
            println!("Cleaned: {}", name);
        } else {
            println!("{} not in cache", name);
        }
    } else if Path::new(VENDOR_DIR).exists() {
        std::fs::remove_dir_all(VENDOR_DIR)?;
        println!("Cleaned all cached sources");
    }
    Ok(())
}

/// List available sources.
pub fn list() {
    println!("Available sources:\n");
    for (name, url, tag) in SOURCES {
        let status = if is_cached(name) { "[cached]" } else { "" };
        println!("  {:12} {} @ {} {}", name, url, tag, status);
    }
}

/// Get directory size in bytes.
fn dir_size(path: &Path) -> Result<u64> {
    let output = Command::new("du")
        .args(["-sb", path.to_str().unwrap()])
        .output()
        .context("Failed to get directory size")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let size_str = stdout.split_whitespace().next().unwrap_or("0");
    Ok(size_str.parse().unwrap_or(0))
}
