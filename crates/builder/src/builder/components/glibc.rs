//! System library collection (glibc + dependencies).

use super::{systemd::Systemd, util_linux::UtilLinux, Buildable};
use crate::builder::initramfs;
use anyhow::{Context, Result};
use std::path::Path;

/// Extract filename as string from path, with proper error handling.
fn file_name_str(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("Invalid path: {}", path.display()))
}

/// System libraries to collect from host.
const SYSTEM_LIBS: &[&str] = &[
    "/lib64/ld-linux-x86-64.so.2",
    "/lib64/libc.so.6",
    "/lib64/libm.so.6",
    "/lib64/libdl.so.2",
    "/lib64/libpthread.so.0",
    "/lib64/librt.so.1",
    "/lib64/libcrypt.so.2",
    "/lib64/libcrypto.so.3",
    "/lib64/libz.so.1",
    "/lib64/libgcc_s.so.1",
    "/lib64/libmount.so.1",     // systemd uses dlopen for this
    "/lib64/libblkid.so.1",     // dependency of libmount
    "/lib64/libselinux.so.1",   // dependency of libmount
    "/lib64/libpcre2-8.so.0",   // dependency of libselinux
    "/lib64/libsmartcols.so.1", // for lsblk output
    "/lib64/libfdisk.so.1",     // for fdisk/sfdisk
    "/lib64/libpam.so.0",       // for login
    "/lib64/libpam_misc.so.0",  // for login
    "/lib64/libaudit.so.1",     // dependency of libpam
    "/lib64/libeconf.so.0",     // dependency of libpam
    "/lib64/libcap-ng.so.0",    // dependency of libpam
    "/lib64/libtirpc.so.3",       // dependency of pam_unix
    "/lib64/libnsl.so.3",         // dependency of pam_unix
    "/lib64/libnss_files.so.2",   // NSS module for /etc/passwd, shadow, group
    "/lib64/libresolv.so.2",      // resolver library
    // Kerberos libs (pam_unix.so links against them even if not used)
    "/lib64/libgssapi_krb5.so.2",
    "/lib64/libkrb5.so.3",
    "/lib64/libk5crypto.so.3",
    "/lib64/libcom_err.so.2",
    "/lib64/libkrb5support.so.0",
    "/lib64/libkeyutils.so.1",
];

/// Collect system libraries into initramfs.
pub fn collect() -> Result<()> {
    println!("=== Collecting system libraries ===");

    let lib_dir = format!("{}/lib64", initramfs::ROOT);
    std::fs::create_dir_all(&lib_dir)?;

    // System libraries from host
    for lib in SYSTEM_LIBS {
        let path = Path::new(lib);
        if path.exists() {
            let dest = format!("{}/{}", lib_dir, file_name_str(path)?);
            // Follow symlinks and copy the actual file
            let real_path = std::fs::canonicalize(path)?;
            std::fs::copy(&real_path, &dest)?;
            println!("  Copied: {lib}");
        } else {
            println!("  Warning: {lib} not found");
        }
    }

    // systemd libraries from build
    for lib in Systemd.lib_paths() {
        let path = Path::new(lib);
        if path.exists() {
            let dest = format!("{}/{}", lib_dir, file_name_str(path)?);
            std::fs::copy(path, &dest)?;
            println!("  Copied: {lib}");
        } else {
            println!("  Warning: {lib} not found (run builder build systemd first)");
        }
    }

    // util-linux libraries from build (may override system libs)
    for lib in UtilLinux.lib_paths() {
        let path = Path::new(lib);
        if path.exists() {
            let dest = format!("{}/{}", lib_dir, file_name_str(path)?);
            std::fs::copy(path, &dest)?;
            println!("  Copied: {lib}");
        } else {
            // util-linux libs are optional - system libs may suffice
            println!("  Note: {lib} not found (using system lib)");
        }
    }

    // PAM modules (in /lib64/security/)
    let security_dir = format!("{}/lib64/security", initramfs::ROOT);
    std::fs::create_dir_all(&security_dir)?;
    let pam_modules = [
        "/lib64/security/pam_unix.so",
        "/lib64/security/pam_permit.so", // For testing/fallback
    ];
    for module in pam_modules {
        let path = Path::new(module);
        if path.exists() {
            let dest = format!("{}/{}", security_dir, file_name_str(path)?);
            let real_path = std::fs::canonicalize(path)?;
            std::fs::copy(&real_path, &dest)?;
            println!("  Copied: {module}");
        } else {
            println!("  Warning: {module} not found");
        }
    }

    // unix_chkpwd helper (used by pam_unix.so for non-root password checks)
    // IMPORTANT: pam_unix.so has /usr/bin/unix_chkpwd hardcoded at compile time
    let chkpwd_paths = ["/usr/bin/unix_chkpwd", "/usr/sbin/unix_chkpwd", "/sbin/unix_chkpwd"];
    let usr_bin_dir = format!("{}/usr/bin", initramfs::ROOT);
    std::fs::create_dir_all(&usr_bin_dir)?;
    for chkpwd in chkpwd_paths {
        if Path::new(chkpwd).exists() {
            let dest = format!("{usr_bin_dir}/unix_chkpwd");
            std::fs::copy(chkpwd, &dest)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o4755))?;
            }
            println!("  Copied: {chkpwd} (setuid)");
            break;
        }
    }

    Ok(())
}
