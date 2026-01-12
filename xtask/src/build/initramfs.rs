//! Initramfs creation module
//!
//! TEAM_466: Extracted from commands.rs during refactor.
//! Consolidates all initramfs creation logic with deduplicated CPIO helper.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Shell test scripts embedded at compile time
const TEST_SH: &str = include_str!("../../initrd_resources/test.sh");
const TEST_CORE_SH: &str = include_str!("../../initrd_resources/test-core.sh");

/// Create a CPIO archive from a directory.
/// Shared helper to eliminate duplicate code across initramfs builders.
fn create_cpio_archive(root: &Path, output_filename: &str) -> Result<()> {
    let cpio_file = std::fs::File::create(output_filename)?;

    let find = Command::new("find")
        .current_dir(root)
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run find")?;

    let mut cpio = Command::new("cpio")
        .current_dir(root)
        .args(["-o", "-H", "newc"])
        .stdin(find.stdout.unwrap())
        .stdout(cpio_file)
        .spawn()
        .context("Failed to run cpio")?;

    let status = cpio.wait()?;
    if !status.success() {
        bail!("cpio failed");
    }

    Ok(())
}

/// Set file as executable (Unix only)
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

// TEAM_435: Uses c-gull sysroot binaries instead of Eyra
// TEAM_444: Migrated to musl - Rust apps use musl target, C apps use musl-gcc
pub fn create_initramfs(arch: &str) -> Result<()> {
    println!("Creating initramfs for {}...", arch);
    let root = PathBuf::from("initrd_root");

    // TEAM_292: Always clean initrd_root to ensure correct arch binaries
    // Without this, stale binaries from other architectures persist
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    // 1. Create content
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;

    // 2. Copy userspace binaries (init, shell - bare-metal)
    let binaries = crate::get_binaries(arch)?;
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };
    print!("📦 Creating initramfs ({} binaries)... ", binaries.len());
    let mut count = 0;
    for bin in &binaries {
        let src = PathBuf::from(format!("crates/userspace/target/{}/release/{}", target, bin));
        if src.exists() {
            std::fs::copy(&src, root.join(bin))?;
            count += 1;
        }
    }

    // TEAM_438: Use apps registry for external apps - fail fast on required, skip optional
    for app in super::apps::APPS {
        if app.required {
            // Required apps must exist - fail fast with helpful message
            let src = app.require(arch)?;
            std::fs::copy(&src, root.join(app.binary))?;
            count += 1;

            // Create symlinks for multi-call binaries
            for symlink_name in app.symlinks {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    let link_path = root.join(symlink_name);
                    let _ = std::fs::remove_file(&link_path);
                    symlink(app.binary, &link_path)?;
                }
                #[cfg(not(unix))]
                {
                    std::fs::copy(&src, root.join(symlink_name))?;
                }
            }

            if app.symlinks.is_empty() {
                println!("  📦 Added {}", app.name);
            } else {
                println!("  📦 Added {} + {} symlinks", app.name, app.symlinks.len());
            }
        } else {
            // Optional apps - include if built, otherwise inform user
            if app.exists(arch) {
                let src = app.output_path(arch);
                std::fs::copy(&src, root.join(app.binary))?;
                count += 1;
                println!("  📦 Added {} (optional)", app.name);
            } else {
                println!("  ℹ️  {} not found (optional). Run 'cargo xtask build {}' to include it.", app.name, app.name);
            }
        }
    }

    // TEAM_444: Include C apps (dash, etc.) if built
    for app in super::c_apps::C_APPS {
        if app.exists(arch) {
            let src = app.output_path(arch);
            let binary_name = std::path::Path::new(app.binary)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| app.name.to_string());
            std::fs::copy(&src, root.join(&binary_name))?;
            count += 1;
            println!("  📦 Added {} (C)", app.name);
        }
    }

    println!("[DONE] ({} added)", count);

    // 3. Create CPIO archive
    // TEAM_327: Use arch-specific filename to prevent cross-arch contamination
    let cpio_filename = format!("initramfs_{}.cpio", arch);
    create_cpio_archive(&root, &cpio_filename)?;

    Ok(())
}

/// TEAM_451: Create BusyBox-based initramfs
/// Single binary provides init, shell, and 300+ utilities
pub fn create_busybox_initramfs(arch: &str) -> Result<()> {
    println!("📦 Creating BusyBox initramfs for {}...", arch);

    // Require BusyBox to be built
    let busybox_path = super::busybox::require(arch)?;

    let root = PathBuf::from("initrd_root");

    // Clean and create directory structure
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir_all(&root)?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::create_dir_all(root.join("sbin"))?;
    std::fs::create_dir_all(root.join("etc"))?;
    std::fs::create_dir_all(root.join("proc"))?;
    std::fs::create_dir_all(root.join("sys"))?;
    std::fs::create_dir_all(root.join("tmp"))?;
    std::fs::create_dir_all(root.join("dev"))?;
    std::fs::create_dir_all(root.join("root"))?;

    // Copy BusyBox binary
    std::fs::copy(&busybox_path, root.join("bin/busybox"))?;
    make_executable(&root.join("bin/busybox"))?;

    // Create symlinks for all applets
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        for (applet, dir) in super::busybox::applets() {
            let link_path = root.join(dir).join(applet);
            let target = if *dir == "sbin" {
                "../bin/busybox"
            } else {
                "busybox"
            };
            let _ = std::fs::remove_file(&link_path);
            symlink(target, &link_path)?;
        }
    }

    // Create /init as a copy of busybox (kernel entry point)
    // TEAM_451: Can't use symlink - kernel ELF loader doesn't follow symlinks
    let _ = std::fs::remove_file(root.join("init"));
    std::fs::copy(&busybox_path, root.join("init"))?;
    make_executable(&root.join("init"))?;

    // Create /etc/inittab (TEAM_460: changed to wait so exit works)
    let inittab = r#"# LevitateOS BusyBox init configuration
# TEAM_451: Generated by xtask
# TEAM_460: Changed respawn to wait so 'exit' terminates shell

# System initialization
::sysinit:/bin/echo "LevitateOS (BusyBox) starting..."
::sysinit:/bin/mount -t proc proc /proc
::sysinit:/bin/mount -t sysfs sysfs /sys

# Test scripts at root level (kernel has issues with subdirectory files)
# Run: sh /test-core.sh [phase] to test coreutils
# Run: sh /test.sh to test basic ash functionality

# Start interactive shell
::wait:-/bin/ash

# Handle Ctrl+Alt+Del
::ctrlaltdel:/sbin/reboot

# Shutdown hooks
::shutdown:/bin/echo "System shutting down..."
"#;
    std::fs::write(root.join("etc/inittab"), inittab)?;

    // Create /etc/passwd
    let passwd = "root:x:0:0:root:/root:/bin/ash\n";
    std::fs::write(root.join("etc/passwd"), passwd)?;

    // Create /etc/group
    let group = "root:x:0:\n";
    std::fs::write(root.join("etc/group"), group)?;

    // Create /etc/profile
    let profile = r#"export PATH=/bin:/sbin
export HOME=/root
export PS1='LevitateOS# '
alias ll='ls -la'
"#;
    std::fs::write(root.join("etc/profile"), profile)?;

    // Create sample files
    std::fs::write(root.join("etc/motd"), "Welcome to LevitateOS!\n")?;
    std::fs::write(root.join("root/hello.txt"), "Hello from BusyBox initramfs!\n")?;

    // TEAM_459: Test script to verify ash shell works
    // TEAM_466: Now loaded from external file
    // TEAM_466: Moved to root level - kernel has issues with initramfs subdirectory files
    std::fs::write(root.join("test.sh"), TEST_SH)?;
    make_executable(&root.join("test.sh"))?;

    // TEAM_460: Comprehensive coreutils test suite - deliberate dependency order
    // TEAM_465: Added phase selection support
    // TEAM_466: Now loaded from external file, moved to root level
    std::fs::write(root.join("test-core.sh"), TEST_CORE_SH)?;
    make_executable(&root.join("test-core.sh"))?;

    // Show what we created
    let applet_count = super::busybox::applets().len();
    println!("  📦 BusyBox binary + {} applet symlinks", applet_count);
    println!("  📄 /etc/inittab, passwd, group, profile");

    // Create CPIO archive
    let cpio_filename = format!("initramfs_{}.cpio", arch);
    create_cpio_archive(&root, &cpio_filename)?;

    // Show final size
    let metadata = std::fs::metadata(&cpio_filename)?;
    let size_kb = metadata.len() / 1024;
    println!("✅ BusyBox initramfs created: {} ({} KB)", cpio_filename, size_kb);

    Ok(())
}

/// TEAM_435: Create test-specific initramfs with coreutils.
/// TEAM_438: Uses apps registry for external apps.
/// Includes init, shell, and required apps for testing.
pub fn create_test_initramfs(arch: &str) -> Result<()> {
    println!("Creating test initramfs for {}...", arch);
    let root = PathBuf::from("initrd_test_root");

    // Clean and create directory
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    let bare_target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Copy init and shell for boot
    let init_src = PathBuf::from(format!("crates/userspace/target/{}/release/init", bare_target));
    let shell_src = PathBuf::from(format!("crates/userspace/target/{}/release/shell", bare_target));

    if init_src.exists() {
        std::fs::copy(&init_src, root.join("init"))?;
    }
    if shell_src.exists() {
        std::fs::copy(&shell_src, root.join("shell"))?;
    }

    // Create hello.txt for cat test
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;

    // TEAM_438: Use apps registry - only include required apps for test initramfs
    let mut app_count = 0;
    for app in super::apps::required_apps() {
        let src = app.require(arch)?;
        std::fs::copy(&src, root.join(app.binary))?;
        app_count += 1;

        // Create symlinks for multi-call binaries
        for symlink_name in app.symlinks {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let link_path = root.join(symlink_name);
                let _ = std::fs::remove_file(&link_path);
                symlink(app.binary, &link_path)?;
            }
            #[cfg(not(unix))]
            {
                std::fs::copy(&src, root.join(symlink_name))?;
            }
        }
    }

    println!("📦 Test initramfs: {} apps + init/shell", app_count);

    // Create CPIO archive
    create_cpio_archive(&root, "initramfs_test.cpio")?;

    println!("✅ Created initramfs_test.cpio");
    Ok(())
}
