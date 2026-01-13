//! System library collection (glibc + dependencies).

use crate::builder::{initramfs, systemd};
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
    "/lib64/libmount.so.1",   // systemd uses dlopen for this
    "/lib64/libblkid.so.1",   // dependency of libmount
    "/lib64/libselinux.so.1", // dependency of libmount
    "/lib64/libpcre2-8.so.0", // dependency of libselinux
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

    Ok(())
}
