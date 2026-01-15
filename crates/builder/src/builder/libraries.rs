//! Library collection from Fedora root.
//!
//! Copies shared libraries from the extracted Fedora root filesystem.
//! This ensures reproducible builds with consistent library versions.

use crate::builder::{fedora, initramfs};
use anyhow::{Context, Result};
use std::path::Path;

/// Collect system libraries from Fedora root into initramfs.
pub fn collect() -> Result<()> {
    println!("=== Collecting libraries from Fedora ===");

    let fedora_root = fedora::root();
    let lib_dir = format!("{}/lib64", initramfs::ROOT);
    let usr_lib_dir = format!("{}/usr/lib64", initramfs::ROOT);
    std::fs::create_dir_all(&lib_dir)?;
    std::fs::create_dir_all(&usr_lib_dir)?;

    // Copy libraries from fedora.LIBRARIES list
    let mut copied = 0;
    let mut missing = 0;

    for lib_name in fedora::LIBRARIES {
        // Try usr/lib64 first, then lib64 (some libs are in root lib64)
        let src_paths = [
            fedora_root.join("usr/lib64").join(lib_name),
            fedora_root.join("lib64").join(lib_name),
        ];

        let mut found = false;
        for src_path in &src_paths {
            if src_path.exists() {
                // Libraries with subdirs (like systemd/) go to /usr/lib64 to match RUNPATH
                let dest = if lib_name.contains('/') {
                    format!("{usr_lib_dir}/{lib_name}")
                } else {
                    format!("{lib_dir}/{lib_name}")
                };
                copy_lib(src_path, &dest)?;
                copied += 1;
                found = true;
                break;
            }
        }

        if !found {
            println!("  Warning: {lib_name} not found in Fedora root");
            missing += 1;
        }
    }

    println!("  Copied {copied} libraries ({missing} not found)");

    // PAM modules
    copy_pam_modules(&fedora_root)?;

    Ok(())
}

/// Copy a library, following symlinks to get the real file.
fn copy_lib(src: &Path, dest: &str) -> Result<()> {
    // Ensure parent directory exists (for libs in subdirs like systemd/)
    let dest_path = Path::new(dest);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Follow symlinks to get the actual file
    let real_path = std::fs::canonicalize(src)
        .with_context(|| format!("Failed to resolve symlink: {}", src.display()))?;

    std::fs::copy(&real_path, dest)
        .with_context(|| format!("Failed to copy {} -> {dest}", real_path.display()))?;

    // If src was a symlink, also create the symlink in dest
    if src.is_symlink() {
        let link_target = std::fs::read_link(src)?;
        let link_name = src.file_name().unwrap().to_str().unwrap();
        let dest_dir = Path::new(dest).parent().unwrap();
        let dest_link = dest_dir.join(link_name);

        // The dest we copied to is the real file name, create symlink to it
        if dest_link.to_string_lossy() != dest {
            let _ = std::fs::remove_file(&dest_link);
            std::os::unix::fs::symlink(link_target, &dest_link)?;
        }
    }

    Ok(())
}

/// Copy PAM modules from Fedora root.
fn copy_pam_modules(fedora_root: &Path) -> Result<()> {
    let security_src = fedora_root.join("usr/lib64/security");
    let security_dest = format!("{}/lib64/security", initramfs::ROOT);
    std::fs::create_dir_all(&security_dest)?;

    let mut copied = 0;

    for module_name in fedora::PAM_MODULES {
        let src = security_src.join(module_name);
        if src.exists() {
            let dest = format!("{security_dest}/{module_name}");
            let real_path = std::fs::canonicalize(&src)?;
            std::fs::copy(&real_path, &dest)?;
            copied += 1;
        } else {
            println!("  Warning: PAM module {module_name} not found");
        }
    }

    println!("  Copied {copied} PAM modules");

    Ok(())
}
