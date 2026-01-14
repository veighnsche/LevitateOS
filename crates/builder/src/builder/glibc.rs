//! System library collection (glibc + dependencies).

use crate::builder::{initramfs, systemd, util_linux};
use anyhow::Result;
use std::path::Path;

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
            let dest = format!(
                "{}/{}",
                lib_dir,
                path.file_name().unwrap().to_str().unwrap()
            );
            // Follow symlinks and copy the actual file
            let real_path = std::fs::canonicalize(path)?;
            std::fs::copy(&real_path, &dest)?;
            println!("  Copied: {}", lib);
        } else {
            println!("  Warning: {} not found", lib);
        }
    }

    // systemd libraries from build
    for lib in systemd::lib_paths() {
        let path = Path::new(lib);
        if path.exists() {
            let dest = format!(
                "{}/{}",
                lib_dir,
                path.file_name().unwrap().to_str().unwrap()
            );
            std::fs::copy(path, &dest)?;
            println!("  Copied: {}", lib);
        } else {
            println!(
                "  Warning: {} not found (run builder systemd first)",
                lib
            );
        }
    }

    // util-linux libraries from build (may override system libs)
    for lib in util_linux::lib_paths() {
        let path = Path::new(lib);
        if path.exists() {
            let dest = format!(
                "{}/{}",
                lib_dir,
                path.file_name().unwrap().to_str().unwrap()
            );
            std::fs::copy(path, &dest)?;
            println!("  Copied: {}", lib);
        } else {
            // util-linux libs are optional - system libs may suffice
            println!("  Note: {} not found (using system lib)", lib);
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
            let dest = format!(
                "{}/{}",
                security_dir,
                path.file_name().unwrap().to_str().unwrap()
            );
            let real_path = std::fs::canonicalize(path)?;
            std::fs::copy(&real_path, &dest)?;
            println!("  Copied: {}", module);
        } else {
            println!("  Warning: {} not found", module);
        }
    }

    Ok(())
}
