//! Initramfs CPIO archive builder.

use crate::builder::auth;
use crate::builder::fedora;
use crate::builder::libraries;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub const ROOT: &str = "build/initramfs";
const CPIO_PATH: &str = "build/initramfs.cpio.gz";

/// Create the initramfs CPIO archive.
pub fn create() -> Result<()> {
    println!("=== Creating initramfs ===");

    // Ensure Fedora root is extracted first
    fedora::ensure_extracted()?;

    // Clean previous build
    if Path::new(ROOT).exists() {
        std::fs::remove_dir_all(ROOT)?;
    }

    create_directories()?;
    libraries::collect()?;
    copy_binaries()?;
    create_init_symlink()?;
    create_etc_files()?;
    create_systemd_units()?;
    create_cpio()?;

    Ok(())
}

fn create_directories() -> Result<()> {
    let dirs = [
        ROOT,
        &format!("{ROOT}/bin"),
        &format!("{ROOT}/sbin"),
        &format!("{ROOT}/lib64"),
        &format!("{ROOT}/etc"),
        &format!("{ROOT}/dev"),
        &format!("{ROOT}/proc"),
        &format!("{ROOT}/sys"),
        &format!("{ROOT}/sys/fs/cgroup"),
        &format!("{ROOT}/tmp"),
        &format!("{ROOT}/run"),
        &format!("{ROOT}/run/systemd"),
        &format!("{ROOT}/root"),
        &format!("{ROOT}/home/live"),
        &format!("{ROOT}/var/log"),
        &format!("{ROOT}/var/tmp"),
        &format!("{ROOT}/usr/lib/systemd/system"),
        &format!("{ROOT}/usr/lib/systemd"),
        &format!("{ROOT}/etc/systemd/system/getty.target.wants"),
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

    // /var/run -> /run symlink (required by many programs)
    let var_run = root.join("var/run");
    if !var_run.exists() {
        std::os::unix::fs::symlink("../run", &var_run)?;
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
    println!("=== Copying binaries from Fedora ===");

    let root = Path::new(ROOT);
    let fedora_root = fedora::root();
    let mut copied = 0;
    let mut missing = 0;

    // Copy regular binaries
    for (src, dest) in fedora::BINARIES {
        let src_path = fedora_root.join(src);
        if copy_fedora_binary(&src_path, dest, root, 0o755)? {
            copied += 1;
        } else {
            println!("  Warning: {} not found", src);
            missing += 1;
        }
    }

    // Copy setuid binaries
    for (src, dest) in fedora::SETUID_BINARIES {
        let src_path = fedora_root.join(src);
        if copy_fedora_binary(&src_path, dest, root, 0o4755)? {
            copied += 1;
        } else {
            println!("  Warning: {} not found (setuid)", src);
            missing += 1;
        }
    }

    println!("  Copied {copied} binaries ({missing} not found)");

    Ok(())
}

/// Copy a binary from Fedora root, handling symlinks.
/// Returns true if file was copied, false if source doesn't exist.
fn copy_fedora_binary(src_path: &Path, dest: &str, root: &Path, mode: u32) -> Result<bool> {
    let dest_path = root.join(dest);

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !src_path.exists() {
        return Ok(false);
    }

    // Handle symlinks - copy the target file
    if src_path.is_symlink() {
        let real_path = std::fs::canonicalize(src_path)?;
        std::fs::copy(&real_path, &dest_path)?;
    } else {
        std::fs::copy(src_path, &dest_path)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(mode))?;
    }

    Ok(true)
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

fn create_etc_files() -> Result<()> {
    let root = Path::new(ROOT);

    // Authentication configuration (passwd, shadow, group, PAM, NSS)
    let auth_config = auth::AuthConfig::default();
    auth_config.write_to(root)?;

    // Copy levitate-test binary (standalone Rust binary for in-VM testing)
    copy_test_binary(root)?;

    // Non-auth system files
    std::fs::write(root.join("etc/hostname"), "levitate\n")?;
    std::fs::write(root.join("etc/shells"), "/bin/sh\n/bin/bash\n")?;

    // /etc/os-release - required by systemd
    std::fs::write(
        root.join("etc/os-release"),
        "NAME=\"LevitateOS\"\n\
         ID=levitate\n\
         VERSION_ID=1\n\
         PRETTY_NAME=\"LevitateOS 1\"\n",
    )?;

    // /etc/machine-id - systemd will generate if empty, but file must exist
    std::fs::write(root.join("etc/machine-id"), "")?;

    // /var/lib/systemd directory for state
    std::fs::create_dir_all(root.join("var/lib/systemd"))?;

    // /etc/profile - basic shell environment
    std::fs::write(
        root.join("etc/profile"),
        "export PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin\n\
         export TERM=${TERM:-vt100}\n\
         export HOME=${HOME:-/root}\n\
         export PS1='\\u@\\h:\\w\\$ '\n",
    )?;

    // Terminfo entries for terminal handling (from Fedora)
    let fedora_root = fedora::root();
    let terminfo_entries = [
        "usr/share/terminfo/l/linux",
        "usr/share/terminfo/v/vt100",
        "usr/share/terminfo/v/vt220",
        "usr/share/terminfo/x/xterm",
        "usr/share/terminfo/x/xterm-256color",
    ];
    for entry in terminfo_entries {
        let src = fedora_root.join(entry);
        let dest = root.join(entry);
        if src.exists() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)?;
        }
    }

    // Power control scripts (systemctl needs --force without D-Bus)
    create_power_scripts(root)?;

    println!("  Created /etc files (with authentication)");
    Ok(())
}

fn create_power_scripts(root: &Path) -> Result<()> {
    let sbin = root.join("sbin");
    std::fs::create_dir_all(&sbin)?;

    // These scripts use systemctl --force to bypass D-Bus requirement
    let scripts = [
        ("reboot", "#!/bin/sh\nexec /bin/systemctl --force reboot \"$@\"\n"),
        ("poweroff", "#!/bin/sh\nexec /bin/systemctl --force poweroff \"$@\"\n"),
        ("halt", "#!/bin/sh\nexec /bin/systemctl --force halt \"$@\"\n"),
        ("shutdown", "#!/bin/sh\n# shutdown [-h|-r] [-t secs] time [message]\ncase \"$1\" in\n  -r) exec /bin/systemctl --force reboot ;;\n  -h|-P) exec /bin/systemctl --force poweroff ;;\n  *) exec /bin/systemctl --force poweroff ;;\nesac\n"),
    ];

    for (name, content) in scripts {
        let path = sbin.join(name);
        std::fs::write(&path, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    Ok(())
}

fn copy_test_binary(root: &Path) -> Result<()> {
    let bin_src = Path::new("tools/levitate-test/target/release/levitate-test");
    let bin_dest = root.join("bin/levitate-test");

    if bin_src.exists() {
        std::fs::copy(bin_src, &bin_dest)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_dest, std::fs::Permissions::from_mode(0o755))?;
        }
        println!("  Copied levitate-test binary");
    } else {
        println!("  Warning: levitate-test not found (run: cd tools/levitate-test && cargo build --release)");
    }

    Ok(())
}

#[allow(clippy::too_many_lines)] // Unit files are static config, grouping is intentional
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

    // levitate-test.target - for automated testing (use with systemd.unit=levitate-test.target)
    std::fs::write(
        systemd_dir.join("levitate-test.target"),
        "[Unit]\n\
         Description=LevitateOS Automated Test\n\
         Requires=basic.target\n\
         After=basic.target\n\
         Wants=levitate-test.service\n\
         AllowIsolate=yes\n",
    )?;

    // levitate-test.service - runs test suite and shuts down
    std::fs::write(
        systemd_dir.join("levitate-test.service"),
        "[Unit]\n\
         Description=Run LevitateOS Test Suite\n\
         After=basic.target\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         Environment=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         TTYPath=/dev/console\n\
         ExecStart=/bin/levitate-test\n\
         ExecStopPost=/sbin/poweroff\n",
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
        std::os::unix::fs::symlink("/usr/lib/systemd/system/getty@.service", &getty_tty1)?;
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
    // Compress with gzip for faster loading and smaller size
    let output = Command::new("fakeroot")
        .args([
            "sh",
            "-c",
            &format!("cd {ROOT} && find . | cpio -o -H newc | gzip > ../initramfs.cpio.gz"),
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
    #[allow(clippy::cast_precision_loss)] // Display doesn't need u64 precision
    let size_mb = size as f64 / 1_000_000.0;
    println!("\n  Created: {CPIO_PATH} ({size_mb:.1} MB)");

    Ok(())
}
