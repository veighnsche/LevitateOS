//! Initramfs CPIO archive builder.

use crate::builder::auth;
use crate::builder::components::{glibc, registry};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub const ROOT: &str = "build/initramfs";
const CPIO_PATH: &str = "build/initramfs.cpio";

/// Create the initramfs CPIO archive.
pub fn create() -> Result<()> {
    println!("=== Creating initramfs ===");

    // Clean previous build
    if Path::new(ROOT).exists() {
        std::fs::remove_dir_all(ROOT)?;
    }

    create_directories()?;
    glibc::collect()?;
    copy_binaries()?;
    create_init_symlink()?;
    create_symlinks()?;
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
        &format!("{}/home/live", ROOT),
        &format!("{}/var/log", ROOT),
        &format!("{}/usr/lib/systemd/system", ROOT),
        &format!("{}/usr/lib/systemd", ROOT),
        &format!("{}/etc/systemd/system/getty.target.wants", ROOT),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Compatibility symlinks
    let root = Path::new(ROOT);
    let lib_link = root.join("lib");
    if !lib_link.exists() {
        std::os::unix::fs::symlink("lib64", &lib_link)?;
    }

    // PAM module path: Fedora's libpam looks in /usr/lib64/security/ first
    let usr_lib64 = root.join("usr/lib64");
    std::fs::create_dir_all(&usr_lib64)?;
    let security_link = usr_lib64.join("security");
    if !security_link.exists() {
        std::os::unix::fs::symlink("../../lib64/security", &security_link)?;
    }

    Ok(())
}

fn copy_binaries() -> Result<()> {
    let root = Path::new(ROOT);

    // Copy all binaries from registry
    for component in registry::COMPONENTS {
        for (src, dest) in component.binaries {
            let src_path = Path::new(src);
            let dest_path = root.join(dest);

            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

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
                    "  Warning: {} not found (run builder build {} first)",
                    src, component.name
                );
            }
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

fn create_symlinks() -> Result<()> {
    let root = Path::new(ROOT);

    // Create all symlinks from registry
    for component in registry::COMPONENTS {
        for (link_name, target) in component.symlinks {
            let link_path = root.join("bin").join(link_name);
            if !link_path.exists() {
                std::os::unix::fs::symlink(target, &link_path)?;
            }
        }
        if !component.symlinks.is_empty() {
            println!("  Created {} symlinks for {}", component.symlinks.len(), component.name);
        }
    }

    // Copy runtime directories from registry
    for component in registry::COMPONENTS {
        for (src, dest) in component.runtime_dirs {
            let src_path = Path::new(src);
            let dest_path = root.join(dest);
            if src_path.exists() {
                copy_dir_recursive(src_path, &dest_path)?;
                println!("  Copied {} runtime: {}", component.name, dest);
            }
        }
    }

    Ok(())
}

fn create_etc_files() -> Result<()> {
    let root = Path::new(ROOT);

    // Authentication configuration (passwd, shadow, group, PAM, NSS)
    let auth_config = auth::AuthConfig::default();
    auth_config.write_to(root)?;

    // Non-auth system files
    std::fs::write(root.join("etc/hostname"), "levitate\n")?;
    std::fs::write(root.join("etc/shells"), "/bin/sh\n/bin/brush\n")?;

    // /etc/profile - basic shell environment
    std::fs::write(
        root.join("etc/profile"),
        "export PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin\n\
         export TERM=${TERM:-vt100}\n\
         export HOME=${HOME:-/root}\n\
         export PS1='\\u@\\h:\\w\\$ '\n",
    )?;

    // Terminfo entries for terminal handling
    let terminfo_dirs = ["usr/share/terminfo/l", "usr/share/terminfo/v"];
    for dir in terminfo_dirs {
        std::fs::create_dir_all(root.join(dir))?;
    }
    // Copy minimal terminfo entries from host
    let terminfo_files = [
        ("/usr/share/terminfo/l/linux", "usr/share/terminfo/l/linux"),
        ("/usr/share/terminfo/v/vt100", "usr/share/terminfo/v/vt100"),
    ];
    for (src, dest) in terminfo_files {
        if Path::new(src).exists() {
            std::fs::copy(src, root.join(dest))?;
        }
    }

    println!("  Created /etc files (with authentication)");
    Ok(())
}

fn create_systemd_units() -> Result<()> {
    let root = Path::new(ROOT);
    let systemd_dir = root.join("usr/lib/systemd/system");
    let getty_wants_dir = root.join("etc/systemd/system/getty.target.wants");

    // sysinit.target - system initialization
    std::fs::write(
        systemd_dir.join("sysinit.target"),
        "[Unit]\n\
         Description=System Initialization\n\
         Documentation=man:systemd.special(7)\n\
         DefaultDependencies=no\n",
    )?;

    // basic.target - basic system ready
    std::fs::write(
        systemd_dir.join("basic.target"),
        "[Unit]\n\
         Description=Basic System\n\
         Documentation=man:systemd.special(7)\n\
         Requires=sysinit.target\n\
         After=sysinit.target\n",
    )?;

    // getty.target - login prompts (after basic, before multi-user)
    std::fs::write(
        systemd_dir.join("getty.target"),
        "[Unit]\n\
         Description=Login Prompts\n\
         Documentation=man:systemd.special(7)\n\
         Requires=basic.target\n\
         After=basic.target\n",
    )?;

    // multi-user.target - full multi-user system
    std::fs::write(
        systemd_dir.join("multi-user.target"),
        "[Unit]\n\
         Description=Multi-User System\n\
         Documentation=man:systemd.special(7)\n\
         Requires=basic.target\n\
         After=basic.target\n\
         Wants=getty.target\n\
         AllowIsolate=yes\n",
    )?;

    // systemd-user-sessions.service - allow user logins
    std::fs::write(
        systemd_dir.join("systemd-user-sessions.service"),
        "[Unit]\n\
         Description=Permit User Sessions\n\
         Documentation=man:systemd-user-sessions.service(8)\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         RemainAfterExit=yes\n\
         ExecStart=/bin/true\n",
    )?;

    // shutdown.target
    std::fs::write(
        systemd_dir.join("shutdown.target"),
        "[Unit]\n\
         Description=System Shutdown\n\
         Documentation=man:systemd.special(7)\n\
         DefaultDependencies=no\n",
    )?;

    // getty@.service (template for virtual terminals)
    // Standard agetty -> login -> PAM -> shell flow
    // Reference: vendor/systemd/units/getty@.service.in
    std::fs::write(
        systemd_dir.join("getty@.service"),
        "[Unit]\n\
         Description=Getty on %I\n\
         Documentation=man:agetty(8) man:systemd-getty-generator(8)\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         ExecStart=-/sbin/agetty -o '-p -- \\\\u' --noclear %I linux\n\
         Type=idle\n\
         Restart=always\n\
         RestartSec=0\n\
         UtmpIdentifier=%I\n\
         TTYPath=/dev/%I\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
         TTYVTDisallocate=yes\n\
         StandardInput=tty\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n\
         \n\
         [Install]\n\
         WantedBy=getty.target\n",
    )?;

    // serial-getty@.service (for serial consoles)
    // Standard agetty -> login -> PAM -> shell flow
    // Reference: vendor/systemd/units/serial-getty@.service.in
    std::fs::write(
        systemd_dir.join("serial-getty@.service"),
        "[Unit]\n\
         Description=Serial Getty on %I\n\
         Documentation=man:agetty(8) man:systemd-getty-generator(8)\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         ExecStart=-/sbin/agetty -o '-p -- \\\\u' --keep-baud 115200,57600,38400,9600 %I vt100\n\
         Type=idle\n\
         Restart=always\n\
         RestartSec=0\n\
         UtmpIdentifier=%I\n\
         TTYPath=/dev/%I\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
         StandardInput=tty\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n\
         \n\
         [Install]\n\
         WantedBy=getty.target\n",
    )?;

    // Create getty.target.wants symlinks
    let getty_tty1 = getty_wants_dir.join("getty@tty1.service");
    if !getty_tty1.exists() {
        std::os::unix::fs::symlink(
            "/usr/lib/systemd/system/getty@.service",
            &getty_tty1,
        )?;
    }

    let serial_getty = getty_wants_dir.join("serial-getty@ttyS0.service");
    if !serial_getty.exists() {
        std::os::unix::fs::symlink(
            "/usr/lib/systemd/system/serial-getty@.service",
            &serial_getty,
        )?;
    }

    // rescue.target
    std::fs::write(
        systemd_dir.join("rescue.target"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         Documentation=man:systemd.special(7)\n\
         Requires=sysinit.target rescue.service\n\
         After=sysinit.target rescue.service\n\
         AllowIsolate=yes\n",
    )?;

    // rescue.service
    std::fs::write(
        systemd_dir.join("rescue.service"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         DefaultDependencies=no\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         Environment=HOME=/root\n\
         Environment=TERM=linux\n\
         WorkingDirectory=/root\n\
         ExecStart=-/bin/sh\n\
         Type=idle\n\
         StandardInput=tty\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         TTYPath=/dev/console\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
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

    // default.target symlink -> multi-user.target
    let default_target = systemd_dir.join("default.target");
    if default_target.exists() {
        std::fs::remove_file(&default_target)?;
    }
    std::os::unix::fs::symlink("multi-user.target", &default_target)?;

    println!("  Created systemd unit files (with getty)");
    Ok(())
}

fn create_cpio() -> Result<()> {
    // Use fakeroot to make all files appear owned by root:root
    // This is required for PAM to accept the shadow file
    let output = Command::new("fakeroot")
        .args([
            "sh",
            "-c",
            &format!(
                "cd {} && find . | cpio -o -H newc > ../initramfs.cpio",
                ROOT
            ),
        ])
        .output()
        .context("Failed to create CPIO (is fakeroot installed?)")?;

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

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
