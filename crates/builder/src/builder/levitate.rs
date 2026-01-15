//! LevitateOS-specific components and philosophy.
//!
//! ## What Makes LevitateOS Different
//!
//! We borrow:
//! - Binaries and libraries from Fedora (fedora.rs)
//! - Reference implementation study from Arch (arch.rs)
//! - Fedora's release schedule (not rolling)
//!
//! We deliberately EXCLUDE:
//! - dnf, rpm (Fedora's package manager)
//! - pacman (Arch's package manager)
//! - Any traditional package manager
//!
//! Instead, we provide:
//! - AI-native software management
//! - FunctionGemma (offline SLM for OS tasks)
//! - OpenCode CLI (online AI agent)
//! - Rhai recipe system (AI generates, interpreter executes)
//! - Declarative system config (~/.levitate/system.rhai)
//!
//! ## Architecture
//!
//! ```text
//! User -> AI (online/offline) -> Rhai Recipe -> Build -> Install
//! ```

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// LevitateOS tools built from source (in tools/ directory).
pub const LEVITATE_TOOLS: &[&str] = &[
    // Phase 2: FunctionGemma LLM runner
    "llm-runner",
];

/// External tools to bundle (downloaded to vendor/).
pub const EXTERNAL_TOOLS: &[(&str, &str)] = &[
    // Phase 3: OpenCode CLI
    // ("vendor/opencode/opencode", "bin/opencode"),
];

/// AI models to include in initramfs.
pub const MODELS: &[(&str, &str)] = &[
    // Phase 2: FunctionGemma Q8_0 GGUF
    (
        "vendor/models/FunctionGemma/functiongemma-270m-it-Q8_0.gguf",
        "usr/lib/levitate/models/functiongemma.gguf",
    ),
];

/// Copy LevitateOS-specific tools to initramfs.
///
/// Unlike fedora.rs (extracts Fedora binaries) or arch.rs (extracts Arch binaries),
/// this copies tools we build ourselves.
pub fn copy_tools(root: &Path) -> Result<()> {
    if LEVITATE_TOOLS.is_empty() {
        return Ok(());
    }

    println!("=== Copying LevitateOS tools ===");

    for tool_name in LEVITATE_TOOLS {
        let tool_dir = format!("tools/{tool_name}");

        // Build the tool in release mode
        println!("  Building {tool_name}...");
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&tool_dir)
            .status()
            .with_context(|| format!("Failed to run cargo build for {tool_name}"))?;

        if !status.success() {
            bail!("Failed to build {tool_name}");
        }

        // Copy the binary to initramfs /bin
        let src = format!("{tool_dir}/target/release/{tool_name}");
        let dest = root.join(format!("bin/{tool_name}"));

        if !Path::new(&src).exists() {
            bail!("Built binary not found: {src}");
        }

        std::fs::copy(&src, &dest)
            .with_context(|| format!("Failed to copy {src} to {}", dest.display()))?;

        // Set executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
        }

        println!("  Copied {tool_name} to /bin/{tool_name}");
    }

    Ok(())
}

/// Copy AI models to initramfs.
pub fn copy_models(root: &Path) -> Result<()> {
    if MODELS.is_empty() {
        return Ok(());
    }

    println!("=== Copying AI models ===");

    for (src, dest) in MODELS {
        let src_path = Path::new(src);
        let dest_path = root.join(dest);

        if !src_path.exists() {
            println!("  Warning: Model not found: {src}");
            println!("  Download it with: hf download ggml-org/functiongemma-270m-it-GGUF --include \"functiongemma-270m-it-Q8_0.gguf\" --local-dir vendor/models/FunctionGemma");
            continue;
        }

        // Create parent directory
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Copy the model file
        let size = std::fs::metadata(src_path)?.len();
        println!(
            "  Copying {} ({:.1} MB)...",
            src,
            size as f64 / 1_000_000.0
        );
        std::fs::copy(src_path, &dest_path)
            .with_context(|| format!("Failed to copy {src} to {}", dest_path.display()))?;

        println!("  Model copied to /{dest}");
    }

    Ok(())
}
