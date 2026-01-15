//! Fedora ISO extraction and path helpers.
//!
//! Uses a pinned Fedora ISO as the source for all userspace binaries and libraries.
//! The ISO contains a squashfs image with a complete Fedora root filesystem.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Fedora ISO configuration.
const FEDORA_ISO: &str = "Fedora-Sway-Live-x86_64-43-1.6.iso";
const FEDORA_SHA256: &str = "41d08d2e5b99a9f255eddc9aa7c310fea5436de1cc90085e57f5520c937d8bd6";

/// Paths relative to workspace root.
const ISO_PATH: &str = "vendor/images/Fedora-Sway-Live-x86_64-43-1.6.iso";
const FEDORA_ROOT: &str = "vendor/fedora-root";

/// Get the path to the extracted Fedora root filesystem.
pub fn root() -> PathBuf {
    PathBuf::from(FEDORA_ROOT)
}

/// Ensure the Fedora root filesystem is extracted.
/// Returns the path to the extracted root.
pub fn ensure_extracted() -> Result<PathBuf> {
    let root_path = root();

    // Check if already extracted
    if root_path.join("usr/bin/bash").exists() {
        println!("Fedora root already extracted at {}", root_path.display());
        return Ok(root_path);
    }

    println!("=== Extracting Fedora root filesystem ===");

    // Verify ISO exists
    let iso_path = Path::new(ISO_PATH);
    if !iso_path.exists() {
        bail!(
            "Fedora ISO not found at {}\n\
             Download from: https://fedoraproject.org/spins/sway/download\n\
             Expected: {}\n\
             SHA256: {}",
            ISO_PATH, FEDORA_ISO, FEDORA_SHA256
        );
    }

    // Verify ISO checksum
    verify_iso_checksum(iso_path)?;

    // Extract the ISO and squashfs
    extract_fedora_root(iso_path, &root_path)?;

    Ok(root_path)
}

/// Verify the ISO checksum matches expected.
fn verify_iso_checksum(iso_path: &Path) -> Result<()> {
    println!("  Verifying ISO checksum...");

    let output = Command::new("sha256sum")
        .arg(iso_path)
        .output()
        .context("Failed to run sha256sum")?;

    if !output.status.success() {
        bail!("sha256sum failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let checksum_output = String::from_utf8_lossy(&output.stdout);
    let computed_hash = checksum_output
        .split_whitespace()
        .next()
        .context("Failed to parse sha256sum output")?;

    if computed_hash != FEDORA_SHA256 {
        bail!(
            "ISO checksum mismatch!\n\
             Expected: {}\n\
             Got: {}\n\
             The ISO may be corrupted or a different version.",
            FEDORA_SHA256, computed_hash
        );
    }

    println!("  Checksum verified: {}", &FEDORA_SHA256[..16]);
    Ok(())
}

/// Extract the Fedora root filesystem from the ISO.
fn extract_fedora_root(iso_path: &Path, root_path: &Path) -> Result<()> {
    // Create temp directory for ISO mount
    let temp_dir = PathBuf::from("build/fedora-extract-temp");
    let iso_mount = temp_dir.join("iso");
    let _squashfs_path = temp_dir.join("squashfs.img");

    // Clean up any previous extraction attempt
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }
    std::fs::create_dir_all(&iso_mount)?;

    // Step 1: Extract squashfs.img from ISO using 7z (no sudo needed)
    println!("  Extracting squashfs from ISO...");
    let output = Command::new("7z")
        .args(["x", "-y", &format!("-o{}", temp_dir.display())])
        .arg(iso_path)
        .arg("LiveOS/squashfs.img")
        .output()
        .context("Failed to run 7z - is p7zip installed?")?;

    if !output.status.success() {
        // Try bsdtar as fallback
        println!("  7z failed, trying bsdtar...");
        let output = Command::new("bsdtar")
            .args(["-xf", &iso_path.to_string_lossy()])
            .args(["-C", &temp_dir.to_string_lossy()])
            .arg("LiveOS/squashfs.img")
            .output()
            .context("Failed to run bsdtar")?;

        if !output.status.success() {
            bail!(
                "Failed to extract squashfs from ISO.\n\
                 Install one of: p7zip, bsdtar\n\
                 Error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    let squashfs_img = temp_dir.join("LiveOS/squashfs.img");
    if !squashfs_img.exists() {
        bail!("squashfs.img not found in ISO at LiveOS/squashfs.img");
    }

    // Step 2: Extract squashfs using unsquashfs
    println!("  Extracting root filesystem from squashfs...");

    // unsquashfs extracts to squashfs-root by default, we want fedora-root
    if root_path.exists() {
        std::fs::remove_dir_all(root_path)?;
    }

    let output = Command::new("unsquashfs")
        .arg("-no-xattrs")  // Skip SELinux xattrs (requires root otherwise)
        .args(["-d", &root_path.to_string_lossy()])
        .arg(&squashfs_img)
        .output()
        .context("Failed to run unsquashfs - is squashfs-tools installed?")?;

    if !output.status.success() {
        bail!(
            "unsquashfs failed: {}\n\
             Install squashfs-tools package.",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Fix permissions (squashfs preserves original perms which may be restrictive)
    println!("  Fixing permissions...");
    let chmod_output = Command::new("chmod")
        .args(["-R", "u+rwX"])
        .arg(root_path)
        .output()
        .context("Failed to fix permissions")?;

    if !chmod_output.status.success() {
        println!("  Warning: chmod failed, some files may be inaccessible");
    }

    // Verify extraction succeeded
    if !root_path.join("usr/bin/bash").exists() {
        bail!("Extraction failed - /usr/bin/bash not found in extracted root");
    }

    // Clean up temp directory
    std::fs::remove_dir_all(&temp_dir)?;

    // Show extraction stats
    let bin_count = std::fs::read_dir(root_path.join("usr/bin"))
        .map(|d| d.count())
        .unwrap_or(0);
    let lib_count = std::fs::read_dir(root_path.join("usr/lib64"))
        .map(|d| d.count())
        .unwrap_or(0);

    println!(
        "  Extracted Fedora root: ~{} binaries, ~{} libraries",
        bin_count, lib_count
    );

    Ok(())
}

/// List of binaries to copy from Fedora to initramfs.
/// Format: (source path relative to fedora-root, dest path in initramfs)
pub const BINARIES: &[(&str, &str)] = &[
    // Shells
    ("usr/bin/bash", "bin/bash"),
    ("usr/bin/sh", "bin/sh"),  // Usually symlink to bash

    // Coreutils
    ("usr/bin/ls", "bin/ls"),
    ("usr/bin/cat", "bin/cat"),
    ("usr/bin/cp", "bin/cp"),
    ("usr/bin/mv", "bin/mv"),
    ("usr/bin/rm", "bin/rm"),
    ("usr/bin/mkdir", "bin/mkdir"),
    ("usr/bin/rmdir", "bin/rmdir"),
    ("usr/bin/touch", "bin/touch"),
    ("usr/bin/chmod", "bin/chmod"),
    ("usr/bin/chown", "bin/chown"),
    ("usr/bin/ln", "bin/ln"),
    ("usr/bin/pwd", "bin/pwd"),
    ("usr/bin/echo", "bin/echo"),
    ("usr/bin/printf", "bin/printf"),
    ("usr/bin/test", "bin/test"),
    ("usr/bin/[", "bin/["),
    ("usr/bin/true", "bin/true"),
    ("usr/bin/false", "bin/false"),
    ("usr/bin/sleep", "bin/sleep"),
    ("usr/bin/date", "bin/date"),
    ("usr/bin/uname", "bin/uname"),
    ("usr/bin/hostname", "bin/hostname"),
    ("usr/bin/whoami", "bin/whoami"),
    ("usr/bin/id", "bin/id"),
    ("usr/bin/groups", "bin/groups"),
    ("usr/bin/wc", "bin/wc"),
    ("usr/bin/head", "bin/head"),
    ("usr/bin/tail", "bin/tail"),
    ("usr/bin/cut", "bin/cut"),
    ("usr/bin/sort", "bin/sort"),
    ("usr/bin/uniq", "bin/uniq"),
    ("usr/bin/tr", "bin/tr"),
    ("usr/bin/tee", "bin/tee"),
    ("usr/bin/env", "bin/env"),
    ("usr/bin/printenv", "bin/printenv"),
    ("usr/bin/dirname", "bin/dirname"),
    ("usr/bin/basename", "bin/basename"),
    ("usr/bin/readlink", "bin/readlink"),
    ("usr/bin/realpath", "bin/realpath"),
    ("usr/bin/stat", "bin/stat"),
    ("usr/bin/df", "bin/df"),
    ("usr/bin/du", "bin/du"),
    ("usr/bin/seq", "bin/seq"),
    ("usr/bin/expr", "bin/expr"),
    ("usr/bin/nproc", "bin/nproc"),
    ("usr/bin/tty", "bin/tty"),
    ("usr/bin/stty", "bin/stty"),
    ("usr/bin/clear", "bin/clear"),
    ("usr/bin/reset", "bin/reset"),

    // Text processing
    ("usr/bin/grep", "bin/grep"),
    ("usr/bin/egrep", "bin/egrep"),
    ("usr/bin/fgrep", "bin/fgrep"),
    ("usr/bin/sed", "bin/sed"),
    ("usr/bin/awk", "bin/awk"),
    ("usr/bin/gawk", "bin/gawk"),
    ("usr/bin/less", "bin/less"),
    ("usr/bin/more", "bin/more"),

    // Find utils
    ("usr/bin/find", "bin/find"),
    ("usr/bin/xargs", "bin/xargs"),
    ("usr/bin/locate", "bin/locate"),

    // Diff utils
    ("usr/bin/diff", "bin/diff"),
    ("usr/bin/cmp", "bin/cmp"),

    // Editors
    ("usr/bin/nano", "bin/nano"),
    ("usr/bin/vi", "bin/vi"),
    ("usr/libexec/vi", "usr/libexec/vi"),  // Actual vi binary (script wrapper uses this)

    // Process tools (procps-ng)
    ("usr/bin/ps", "bin/ps"),
    ("usr/bin/top", "bin/top"),
    ("usr/bin/htop", "bin/htop"),
    ("usr/bin/free", "bin/free"),
    ("usr/bin/uptime", "bin/uptime"),
    ("usr/bin/pgrep", "bin/pgrep"),
    ("usr/bin/pkill", "bin/pkill"),
    ("usr/bin/pidof", "bin/pidof"),
    ("usr/bin/kill", "bin/kill"),
    ("usr/bin/killall", "bin/killall"),
    ("usr/bin/watch", "bin/watch"),
    ("usr/bin/pmap", "bin/pmap"),
    ("usr/bin/vmstat", "bin/vmstat"),

    // Network tools (iproute2)
    ("usr/sbin/ip", "sbin/ip"),
    ("usr/sbin/ss", "sbin/ss"),
    ("usr/sbin/bridge", "sbin/bridge"),

    // Network tools (iputils)
    ("usr/bin/ping", "bin/ping"),
    ("usr/bin/tracepath", "bin/tracepath"),

    // Compression
    ("usr/bin/tar", "bin/tar"),
    ("usr/bin/gzip", "bin/gzip"),
    ("usr/bin/gunzip", "bin/gunzip"),
    ("usr/bin/zcat", "bin/zcat"),
    ("usr/bin/xz", "bin/xz"),
    ("usr/bin/unxz", "bin/unxz"),
    ("usr/bin/zstd", "bin/zstd"),
    ("usr/bin/unzstd", "bin/unzstd"),

    // Util-linux (mount, login, etc.)
    ("usr/bin/mount", "bin/mount"),
    ("usr/bin/umount", "bin/umount"),
    ("usr/bin/dmesg", "bin/dmesg"),
    ("usr/bin/lsblk", "bin/lsblk"),
    ("usr/bin/findmnt", "bin/findmnt"),
    ("usr/bin/lscpu", "bin/lscpu"),
    ("usr/bin/lsmem", "bin/lsmem"),

    // Login/auth (util-linux)
    ("usr/bin/login", "bin/login"),
    ("usr/sbin/agetty", "sbin/agetty"),
    ("usr/sbin/sulogin", "sbin/sulogin"),
    ("usr/bin/su", "bin/su"),
    ("usr/bin/passwd", "bin/passwd"),
    ("usr/bin/chpasswd", "bin/chpasswd"),

    // Systemd core
    ("usr/lib/systemd/systemd", "sbin/init"),
    ("usr/bin/systemctl", "bin/systemctl"),
    ("usr/bin/journalctl", "bin/journalctl"),
    ("usr/bin/systemd-tmpfiles", "bin/systemd-tmpfiles"),

    // Systemd helpers (in /usr/lib/systemd/)
    ("usr/lib/systemd/systemd-executor", "usr/lib/systemd/systemd-executor"),
    ("usr/lib/systemd/systemd-journald", "usr/lib/systemd/systemd-journald"),
    ("usr/lib/systemd/systemd-fsck", "usr/lib/systemd/systemd-fsck"),
    ("usr/lib/systemd/systemd-sulogin-shell", "usr/lib/systemd/systemd-sulogin-shell"),
    ("usr/lib/systemd/systemd-user-runtime-dir", "usr/lib/systemd/systemd-user-runtime-dir"),
    ("usr/lib/systemd/systemd-shutdown", "usr/lib/systemd/systemd-shutdown"),

    // Network clients
    ("usr/bin/curl", "bin/curl"),
    ("usr/bin/ssh", "bin/ssh"),
    ("usr/bin/scp", "bin/scp"),

    // Misc utilities
    ("usr/bin/which", "bin/which"),
    ("usr/bin/whereis", "bin/whereis"),
    ("usr/bin/file", "bin/file"),
    ("usr/bin/getent", "bin/getent"),
];

/// Setuid binaries that need special permissions (mode 4755).
pub const SETUID_BINARIES: &[(&str, &str)] = &[
    ("usr/bin/sudo", "bin/sudo"),
    ("usr/bin/su", "bin/su"),
    ("usr/bin/passwd", "bin/passwd"),
    ("usr/sbin/unix_chkpwd", "usr/bin/unix_chkpwd"),
];

/// Sudo support files (libraries and plugins).
pub const SUDO_LIBEXEC: &[&str] = &[
    "libsudo_util.so.0.0.0",
    "libsudo_util.so.0",
    "libsudo_util.so",
    "sudoers.so",
    "group_file.so",
    "system_group.so",
];

/// Libraries to copy from Fedora root.
/// These are the essential system libraries needed for the binaries above.
pub const LIBRARIES: &[&str] = &[
    // Core glibc
    "ld-linux-x86-64.so.2",
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "libpthread.so.0",
    "librt.so.1",
    "libresolv.so.2",
    "libnss_files.so.2",
    "libnss_dns.so.2",

    // Crypto and security
    "libcrypt.so.2",
    "libcrypto.so.3",
    "libssl.so.3",

    // Compression
    "libz.so.1",
    "liblzma.so.5",
    "libzstd.so.1",
    "libbz2.so.1",

    // GCC runtime
    "libgcc_s.so.1",
    "libstdc++.so.6",

    // Mount and block device
    "libmount.so.1",
    "libblkid.so.1",
    "libsmartcols.so.1",
    "libfdisk.so.1",

    // SELinux (even if disabled, libs may be linked)
    "libselinux.so.1",
    "libpcre2-8.so.0",

    // PAM authentication
    "libpam.so.0",
    "libpam_misc.so.0",
    "libaudit.so.1",
    "libeconf.so.0",
    "libcap-ng.so.0",

    // PAM module dependencies
    "libtirpc.so.3",
    "libnsl.so.3",

    // Kerberos (pam_unix links to these)
    "libgssapi_krb5.so.2",
    "libkrb5.so.3",
    "libk5crypto.so.3",
    "libcom_err.so.2",
    "libkrb5support.so.0",
    "libkeyutils.so.1",

    // Systemd libraries
    "libsystemd.so.0",
    "libcap.so.2",
    "libseccomp.so.2",
    "libacl.so.1",
    "libattr.so.1",

    // Systemd private libraries (in usr/lib64/systemd/)
    "systemd/libsystemd-core-258-1.fc43.so",
    "systemd/libsystemd-shared-258-1.fc43.so",

    // Procps-ng
    "libproc2.so.0",

    // iproute2 (ss) dependencies
    "libelf.so.1",
    "libmnl.so.0",

    // ncurses (for top, htop, nano, vim)
    "libncurses.so.6",
    "libncursesw.so.6",
    "libtinfo.so.6",
    "libreadline.so.8",

    // Curl dependencies
    "libcurl.so.4",
    "libnghttp2.so.14",
    "libssh.so.4",
    "libpsl.so.5",
    "libidn2.so.0",
    "libunistring.so.5",
    "libbrotlidec.so.1",
    "libbrotlicommon.so.1",

    // iproute2 (ip, ss) - libbpf for BPF support
    "libbpf.so.1",

    // gawk dependencies
    "libmpfr.so.6",
    "libgmp.so.10",
];

/// PAM modules to copy.
pub const PAM_MODULES: &[&str] = &[
    "pam_unix.so",
    "pam_permit.so",
    "pam_deny.so",
    "pam_env.so",
    "pam_limits.so",
    "pam_securetty.so",
    "pam_nologin.so",
    "pam_motd.so",
    "pam_lastlog.so",
    "pam_shells.so",
];
