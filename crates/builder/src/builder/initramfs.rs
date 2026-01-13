//! Initramfs CPIO archive builder.

use crate::builder::{brush, sudo_rs, systemd, uutils};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub const ROOT: &str = "build/initramfs";
const CPIO_PATH: &str = "build/initramfs.cpio";

/// Create the initramfs CPIO archive.
pub fn create() -> Result<()> {
    println!("=== Creating initramfs ===");

    create_directories()?;
    copy_binaries()?;
    create_init_symlink()?;
    create_coreutils_symlinks()?;
    create_etc_files()?;
    create_systemd_units()?;
    create_cpio()?;

    Ok(())
}

/// Get the CPIO archive path.
pub fn cpio_path() -> &'static str {
    CPIO_PATH
}

fn create_directories() -> Result<()> {
    let dirs = [
        ROOT,
        &format!("{}/bin", ROOT),
        &format!("{}/sbin", ROOT),
        &format!("{}/lib64", ROOT),
        &format!("{}/etc", ROOT),
        &format!("{}/dev", ROOT),
        &format!("{}/proc", ROOT),
        &format!("{}/sys", ROOT),
        &format!("{}/tmp", ROOT),
        &format!("{}/run", ROOT),
        &format!("{}/root", ROOT),
        &format!("{}/usr/lib/systemd/system", ROOT),
        &format!("{}/usr/lib/systemd", ROOT),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

fn copy_binaries() -> Result<()> {
    let root = Path::new(ROOT);

    let copies: &[(&str, &str)] = &[
        (uutils::binary_path(), "bin/coreutils"),
        (sudo_rs::sudo_path(), "bin/sudo"),
        (sudo_rs::su_path(), "bin/su"),
        (brush::binary_path(), "bin/brush"),
        (systemd::binary_path(), "sbin/init"),
        (systemd::executor_path(), "usr/lib/systemd/systemd-executor"),
    ];

    for (src, dest) in copies {
        let src_path = Path::new(src);
        let dest_path = root.join(dest);

        if src_path.exists() {
            std::fs::copy(src_path, &dest_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(0o755))?;
            }
            println!("  Copied: {} -> {}", src, dest);
        } else {
            println!(
                "  Warning: {} not found (run builder <component> first)",
                src
            );
        }
    }

    Ok(())
}

fn create_init_symlink() -> Result<()> {
    let root = Path::new(ROOT);
    let init_link = root.join("init");

    if init_link.exists() {
        std::fs::remove_file(&init_link)?;
    }
    std::os::unix::fs::symlink("sbin/init", &init_link)?;
    println!("  Created /init -> sbin/init symlink");

    Ok(())
}

fn create_coreutils_symlinks() -> Result<()> {
    let root = Path::new(ROOT);

    for cmd in uutils::commands() {
        let link_path = root.join("bin").join(cmd);
        if !link_path.exists() {
            std::os::unix::fs::symlink("coreutils", &link_path)?;
        }
    }
    println!("  Created symlinks for coreutils");

    // Also create /bin/sh -> brush
    let sh_link = root.join("bin/sh");
    if !sh_link.exists() {
        std::os::unix::fs::symlink("brush", &sh_link)?;
    }
    println!("  Created /bin/sh -> brush symlink");

    Ok(())
}

fn create_etc_files() -> Result<()> {
    let root = Path::new(ROOT);

    std::fs::write(root.join("etc/passwd"), "root:x:0:0:root:/root:/bin/sh\n")?;
    std::fs::write(root.join("etc/group"), "root:x:0:\n")?;
    std::fs::write(root.join("etc/hostname"), "levitate\n")?;
    std::fs::write(root.join("etc/shells"), "/bin/sh\n/bin/brush\n")?;

    println!("  Created /etc files");
    Ok(())
}

fn create_systemd_units() -> Result<()> {
    let root = Path::new(ROOT);
    let systemd_dir = root.join("usr/lib/systemd/system");

    // basic.target
    std::fs::write(
        systemd_dir.join("basic.target"),
        "[Unit]\n\
         Description=Basic System\n\
         Documentation=man:systemd.special(7)\n",
    )?;

    // multi-user.target
    std::fs::write(
        systemd_dir.join("multi-user.target"),
        "[Unit]\n\
         Description=Multi-User System\n\
         Documentation=man:systemd.special(7)\n\
         Requires=basic.target\n\
         After=basic.target\n\
         AllowIsolate=yes\n",
    )?;

    // rescue.target
    std::fs::write(
        systemd_dir.join("rescue.target"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         Documentation=man:systemd.special(7)\n\
         Requires=rescue.service\n\
         After=rescue.service\n\
         AllowIsolate=yes\n",
    )?;

    // rescue.service
    std::fs::write(
        systemd_dir.join("rescue.service"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         DefaultDependencies=no\n\
         \n\
         [Service]\n\
         Environment=HOME=/root\n\
         Environment=TERM=dumb\n\
         WorkingDirectory=/root\n\
         ExecStart=-/bin/sh --input-backend minimal\n\
         Type=idle\n\
         StandardInput=tty\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         TTYPath=/dev/console\n\
         TTYReset=no\n\
         TTYVHangup=no\n\
         TTYVTDisallocate=no\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n",
    )?;

    // emergency.target
    std::fs::write(
        systemd_dir.join("emergency.target"),
        "[Unit]\n\
         Description=Emergency Shell\n\
         Documentation=man:systemd.special(7)\n\
         Requires=emergency.service\n\
         After=emergency.service\n\
         AllowIsolate=yes\n",
    )?;

    // emergency.service
    std::fs::write(
        systemd_dir.join("emergency.service"),
        "[Unit]\n\
         Description=Emergency Shell\n\
         DefaultDependencies=no\n\
         \n\
         [Service]\n\
         Environment=HOME=/root\n\
         WorkingDirectory=/root\n\
         ExecStart=-/bin/sh\n\
         Type=idle\n\
         StandardInput=tty-force\n\
         StandardOutput=inherit\n\
         StandardError=inherit\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n",
    )?;

    // default.target symlink -> rescue.target
    let default_target = systemd_dir.join("default.target");
    if default_target.exists() {
        std::fs::remove_file(&default_target)?;
    }
    std::os::unix::fs::symlink("rescue.target", &default_target)?;

    println!("  Created systemd unit files");
    Ok(())
}

fn create_cpio() -> Result<()> {
    let output = Command::new("sh")
        .args([
            "-c",
            &format!(
                "cd {} && find . | cpio -o -H newc > ../initramfs.cpio",
                ROOT
            ),
        ])
        .output()
        .context("Failed to create CPIO")?;

    if !output.status.success() {
        bail!(
            "CPIO creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let size = std::fs::metadata(CPIO_PATH)?.len();
    println!(
        "\n  Created: {} ({:.1} MB)",
        CPIO_PATH,
        size as f64 / 1_000_000.0
    );

    Ok(())
}
