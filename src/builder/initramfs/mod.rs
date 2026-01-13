//! Initramfs builder module
//!
//! TEAM_474: Declarative initramfs builder with pure Rust CPIO writer and TUI dashboard.
//! TEAM_475: Added OpenRC-based initramfs builder.
//!
//! Builds initramfs CPIO archives from declarative TOML manifest.

mod builder;
pub mod cpio;  // TEAM_475: Public for Alpine/OpenRC builder
mod manifest;
mod tui;

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

pub use builder::BuildEvent;

/// Build initramfs for the given architecture
///
/// Loads `initramfs/initramfs.toml` and produces `target/initramfs/{arch}.cpio`
pub fn build_initramfs(arch: &str) -> Result<PathBuf> {
    let base_dir = PathBuf::from("initramfs");
    let manifest_path = base_dir.join("initramfs.toml");

    let manifest = manifest::Manifest::load(
        &manifest_path.to_string_lossy(),
        arch,
        &base_dir,
    )?;

    // Validate with helpful error messages
    if let Err(e) = manifest.validate(&base_dir) {
        // Check if it's a missing busybox binary
        if e.to_string().contains("busybox") {
            eprintln!("  Hint: Run 'cargo xtask build busybox' first");
        }
        return Err(e);
    }

    let totals = manifest.get_totals();
    let builder = builder::InitramfsBuilder::new(manifest, arch, &base_dir);

    if tui::should_use_tui() {
        // TUI mode
        let totals_clone = totals.clone();
        tui::run_build_with_tui(arch, &totals, move |emit| {
            let _ = totals_clone; // capture for lifetime
            builder.build_with_events(move |e| emit(e))
        })
    } else {
        // Simple mode
        println!("  Creating initramfs for {}...", arch);
        builder.build_with_events(|event| {
            tui::print_simple_event(&event);
        })
    }
}

/// Backward compatibility wrapper
///
/// Builds initramfs and copies to legacy location at repo root.
/// All existing call sites use this function.
pub fn create_busybox_initramfs(arch: &str) -> Result<()> {
    println!("  Building BusyBox initramfs for {}...", arch);

    // Require BusyBox to be built first
    super::busybox::require(arch).context("BusyBox binary required")?;

    let output = build_initramfs(arch)?;

    // Copy to legacy location at repo root for backward compatibility
    let legacy_path = format!("initramfs_{arch}.cpio");
    std::fs::copy(&output, &legacy_path)?;

    let size_kb = std::fs::metadata(&legacy_path)?.len() / 1024;
    println!("  BusyBox initramfs created: {} ({} KB)", legacy_path, size_kb);

    Ok(())
}

/// TEAM_475: Create OpenRC-based initramfs
///
/// Combines BusyBox (shell + utilities) with OpenRC (init system).
/// This provides a real init system with service management while keeping
/// BusyBox for the shell and basic utilities.
pub fn create_openrc_initramfs(arch: &str) -> Result<PathBuf> {
    println!("  Building OpenRC initramfs for {}...", arch);

    // Require both BusyBox and OpenRC to be built
    super::busybox::require(arch).context("BusyBox binary required")?;
    let openrc_dir = super::openrc::require(arch).context("OpenRC binaries required")?;
    let busybox_path = super::busybox::output_path(arch);

    let mut archive = cpio::CpioArchive::new();

    // === Create directory structure ===
    let directories = [
        ".", "bin", "sbin", "dev", "etc", "etc/init.d", "etc/conf.d",
        "etc/runlevels", "etc/runlevels/sysinit", "etc/runlevels/boot",
        "etc/runlevels/default", "etc/runlevels/shutdown",
        "lib", "lib/rc", "lib/rc/bin", "lib/rc/sbin", "lib/rc/sh",
        "proc", "sys", "tmp", "root", "mnt", "var", "var/log", "var/run",
        "run", "share", "share/man",
    ];
    for dir in &directories {
        archive.add_directory(dir, 0o755);
    }

    // === Add BusyBox ===
    let busybox_data = std::fs::read(&busybox_path)
        .with_context(|| format!("Failed to read BusyBox from {}", busybox_path.display()))?;
    archive.add_file("bin/busybox", &busybox_data, 0o755);

    // BusyBox symlinks for shell and utilities
    let busybox_applets = [
        // Shell
        ("bin/sh", "/bin/busybox"),
        ("bin/ash", "/bin/busybox"),
        // Core utilities
        ("bin/echo", "/bin/busybox"),
        ("bin/cat", "/bin/busybox"),
        ("bin/ls", "/bin/busybox"),
        ("bin/cp", "/bin/busybox"),
        ("bin/mv", "/bin/busybox"),
        ("bin/rm", "/bin/busybox"),
        ("bin/mkdir", "/bin/busybox"),
        ("bin/rmdir", "/bin/busybox"),
        ("bin/ln", "/bin/busybox"),
        ("bin/pwd", "/bin/busybox"),
        ("bin/true", "/bin/busybox"),
        ("bin/false", "/bin/busybox"),
        ("bin/test", "/bin/busybox"),
        ("bin/[", "/bin/busybox"),
        // Text processing
        ("bin/head", "/bin/busybox"),
        ("bin/tail", "/bin/busybox"),
        ("bin/grep", "/bin/busybox"),
        ("bin/sed", "/bin/busybox"),
        ("bin/awk", "/bin/busybox"),
        ("bin/cut", "/bin/busybox"),
        ("bin/sort", "/bin/busybox"),
        ("bin/wc", "/bin/busybox"),
        ("bin/tr", "/bin/busybox"),
        // File inspection
        ("bin/stat", "/bin/busybox"),
        ("bin/find", "/bin/busybox"),
        ("bin/xargs", "/bin/busybox"),
        ("bin/touch", "/bin/busybox"),
        ("bin/chmod", "/bin/busybox"),
        ("bin/chown", "/bin/busybox"),
        // Process management
        ("bin/ps", "/bin/busybox"),
        ("bin/kill", "/bin/busybox"),
        ("bin/sleep", "/bin/busybox"),
        ("bin/date", "/bin/busybox"),
        ("bin/uname", "/bin/busybox"),
        ("bin/hostname", "/bin/busybox"),
        ("bin/env", "/bin/busybox"),
        ("bin/id", "/bin/busybox"),
        // Mount (BusyBox fallback)
        ("bin/mount", "/bin/busybox"),
        ("bin/umount", "/bin/busybox"),
        // System control (BusyBox fallback)
        ("sbin/reboot", "/bin/busybox"),
        ("sbin/poweroff", "/bin/busybox"),
        ("sbin/halt", "/bin/busybox"),
        // Misc utilities
        ("bin/clear", "/bin/busybox"),
        ("bin/printf", "/bin/busybox"),
        ("bin/seq", "/bin/busybox"),
        ("bin/basename", "/bin/busybox"),
        ("bin/dirname", "/bin/busybox"),
        ("bin/readlink", "/bin/busybox"),
        ("bin/tar", "/bin/busybox"),
        ("bin/gzip", "/bin/busybox"),
        ("bin/gunzip", "/bin/busybox"),
    ];
    for (path, target) in &busybox_applets {
        archive.add_symlink(path, target);
    }

    // === Add OpenRC binaries ===
    // Main OpenRC executables from sbin/
    let openrc_sbin = ["openrc", "openrc-init", "openrc-run", "openrc-shutdown",
                       "rc-service", "rc-update", "start-stop-daemon", "supervise-daemon"];
    for bin in &openrc_sbin {
        let src = openrc_dir.join("sbin").join(bin);
        if src.exists() {
            let data = std::fs::read(&src)
                .with_context(|| format!("Failed to read {}", src.display()))?;
            archive.add_file(&format!("sbin/{}", bin), &data, 0o755);
        }
    }

    // rc-status is in bin/
    let rc_status = openrc_dir.join("bin/rc-status");
    if rc_status.exists() {
        let data = std::fs::read(&rc_status)?;
        archive.add_file("bin/rc-status", &data, 0o755);
    }

    // /init points to BusyBox init (handles inittab, spawns getty)
    // OpenRC runs as services within BusyBox init's inittab
    archive.add_symlink("init", "/bin/busybox");
    archive.add_symlink("sbin/init", "/bin/busybox");

    // === Add OpenRC helper binaries from lib/rc/bin and lib/rc/sbin ===
    for subdir in ["bin", "sbin"] {
        let lib_dir = openrc_dir.join("lib/rc").join(subdir);
        if lib_dir.exists() {
            for entry in std::fs::read_dir(&lib_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap().to_string_lossy();
                    let data = std::fs::read(&path)?;
                    let mode = std::fs::metadata(&path)?.permissions().mode() & 0o7777;
                    archive.add_file(&format!("lib/rc/{}/{}", subdir, name), &data, mode);
                }
            }
        }
    }

    // === Add OpenRC shell scripts from lib/rc/sh ===
    let sh_dir = openrc_dir.join("lib/rc/sh");
    if sh_dir.exists() {
        for entry in std::fs::read_dir(&sh_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                let data = std::fs::read(&path)?;
                archive.add_file(&format!("lib/rc/sh/{}", name), &data, 0o755);
            }
        }
    }

    // === Add OpenRC init scripts from etc/init.d ===
    let initd_dir = openrc_dir.join("etc/init.d");
    if initd_dir.exists() {
        for entry in std::fs::read_dir(&initd_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                let data = std::fs::read(&path)?;
                archive.add_file(&format!("etc/init.d/{}", name), &data, 0o755);
            }
        }
    }

    // === Add OpenRC config files from etc/conf.d ===
    let confd_dir = openrc_dir.join("etc/conf.d");
    if confd_dir.exists() {
        for entry in std::fs::read_dir(&confd_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                let data = std::fs::read(&path)?;
                archive.add_file(&format!("etc/conf.d/{}", name), &data, 0o644);
            }
        }
    }

    // === Add rc.conf ===
    let rc_conf = openrc_dir.join("etc/rc.conf");
    if rc_conf.exists() {
        let data = std::fs::read(&rc_conf)?;
        archive.add_file("etc/rc.conf", &data, 0o644);
    }

    // === Add LevitateOS-specific configuration ===
    // /etc/hostname
    archive.add_file("etc/hostname", b"levitate\n", 0o644);

    // /etc/passwd
    archive.add_file("etc/passwd", b"root:x:0:0:root:/root:/bin/sh\n", 0o644);

    // /etc/group
    archive.add_file("etc/group", b"root:x:0:\n", 0o644);

    // /etc/profile
    archive.add_file("etc/profile", br#"# LevitateOS profile
export PATH=/sbin:/bin:/usr/sbin:/usr/bin
export HOME=/root
export PS1='levitate# '
alias ll='ls -la'
"#, 0o644);

    // /etc/fstab - Required by OpenRC
    archive.add_file("etc/fstab", br#"# LevitateOS fstab
# <device>    <mount>    <type>    <options>    <dump>    <pass>
devtmpfs      /dev       devtmpfs  rw,mode=0755 0         0
proc          /proc      proc      rw           0         0
sysfs         /sys       sysfs     rw           0         0
tmpfs         /tmp       tmpfs     rw,mode=1777 0         0
tmpfs         /run       tmpfs     rw,mode=0755 0         0
"#, 0o644);

    // Add BusyBox applets that OpenRC needs (md5sum, etc)
    archive.add_symlink("bin/md5sum", "/bin/busybox");
    archive.add_symlink("sbin/modprobe", "/bin/busybox");
    archive.add_symlink("sbin/ifconfig", "/bin/busybox");
    archive.add_symlink("sbin/route", "/bin/busybox");
    archive.add_symlink("bin/dmesg", "/bin/busybox");
    archive.add_symlink("sbin/sysctl", "/bin/busybox");
    archive.add_symlink("bin/expr", "/bin/busybox");
    archive.add_symlink("bin/dd", "/bin/busybox");
    archive.add_symlink("bin/mknod", "/bin/busybox");
    archive.add_symlink("sbin/pivot_root", "/bin/busybox");
    archive.add_symlink("sbin/switch_root", "/bin/busybox");
    archive.add_symlink("sbin/blkid", "/bin/busybox");
    archive.add_symlink("sbin/losetup", "/bin/busybox");
    archive.add_symlink("sbin/fsck", "/bin/busybox");
    archive.add_symlink("sbin/swapon", "/bin/busybox");
    archive.add_symlink("sbin/swapoff", "/bin/busybox");
    archive.add_symlink("bin/free", "/bin/busybox");
    archive.add_symlink("sbin/hwclock", "/bin/busybox");
    archive.add_symlink("sbin/getty", "/bin/busybox");
    archive.add_symlink("bin/stty", "/bin/busybox");
    archive.add_symlink("bin/tty", "/bin/busybox");
    archive.add_symlink("bin/login", "/bin/busybox");

    // /etc/inittab for OpenRC
    // TEAM_475: Use ttyS0 for serial console (QEMU -nographic mode)
    // Mount devtmpfs early so /dev/ttyS0 exists for getty
    archive.add_file("etc/inittab", br#"# LevitateOS OpenRC inittab
# Mount devtmpfs first so /dev/ttyS0 exists
::sysinit:/bin/mount -t devtmpfs devtmpfs /dev
::sysinit:/sbin/openrc sysinit
::wait:/sbin/openrc boot
::wait:/sbin/openrc default
# Serial console - spawn shell after runlevels complete
::respawn:/sbin/getty -n -l /bin/ash 115200 ttyS0 vt100
::shutdown:/sbin/openrc shutdown
::ctrlaltdel:/sbin/reboot
"#, 0o644);

    // /root/hello.txt - welcome message
    archive.add_file("root/hello.txt", b"Welcome to LevitateOS with OpenRC!\n", 0o644);

    // === Create runlevel symlinks ===
    // sysinit runlevel
    archive.add_symlink("etc/runlevels/sysinit/devfs", "/etc/init.d/devfs");
    archive.add_symlink("etc/runlevels/sysinit/dmesg", "/etc/init.d/dmesg");
    archive.add_symlink("etc/runlevels/sysinit/procfs", "/etc/init.d/procfs");
    archive.add_symlink("etc/runlevels/sysinit/sysfs", "/etc/init.d/sysfs");

    // boot runlevel
    archive.add_symlink("etc/runlevels/boot/hostname", "/etc/init.d/hostname");
    archive.add_symlink("etc/runlevels/boot/localmount", "/etc/init.d/localmount");

    // default runlevel
    archive.add_symlink("etc/runlevels/default/local", "/etc/init.d/local");

    // shutdown runlevel
    archive.add_symlink("etc/runlevels/shutdown/mount-ro", "/etc/init.d/mount-ro");
    archive.add_symlink("etc/runlevels/shutdown/killprocs", "/etc/init.d/killprocs");
    archive.add_symlink("etc/runlevels/shutdown/savecache", "/etc/init.d/savecache");

    // === Add device nodes ===
    archive.add_char_device("dev/null", 0o666, 1, 3);
    archive.add_char_device("dev/zero", 0o666, 1, 5);
    archive.add_char_device("dev/random", 0o666, 1, 8);
    archive.add_char_device("dev/urandom", 0o666, 1, 9);
    archive.add_char_device("dev/tty", 0o666, 5, 0);
    archive.add_char_device("dev/console", 0o600, 5, 1);

    // === Write archive ===
    let output_dir = PathBuf::from("target/initramfs");
    std::fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join(format!("{}-openrc.cpio", arch));

    let file = std::fs::File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);
    let bytes = archive.write(&mut writer)?;
    drop(writer);

    let size_kb = bytes / 1024;
    println!("  OpenRC initramfs created: {} ({} KB)", output_path.display(), size_kb);

    Ok(output_path)
}
