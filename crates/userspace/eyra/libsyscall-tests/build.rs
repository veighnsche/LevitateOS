// TEAM_380: Build script for libsyscall-tests
// Based on eyra-hello/build.rs (TEAM_357)
//
// NOTE: -nostartfiles is now configured at workspace level in .cargo/config.toml
// Individual binaries no longer need to specify it in build.rs

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch == "aarch64" {
        // TEAM_380: Create empty libgcc_eh.a stub for aarch64 cross-compilation.
        let lib_path = format!("{}/libgcc_eh.a", out_dir);
        let status = std::process::Command::new("ar")
            .args(["rcs", &lib_path])
            .status();

        if status.is_ok() {
            println!("cargo:rustc-link-search=native={}", out_dir);
        }

        // TEAM_380: Provide getauxval stub for compiler_builtins
        // compiler_builtins tries to detect LSE atomics using getauxval,
        // but with -nostartfiles we need to provide it ourselves.
        create_getauxval_stub(&out_dir);
    }
}

fn create_getauxval_stub(out_dir: &str) {
    let stub_c = PathBuf::from(out_dir).join("getauxval_stub.c");
    let stub_o = PathBuf::from(out_dir).join("getauxval_stub.o");
    let stub_a = PathBuf::from(out_dir).join("libgetauxval_stub.a");

    // Create a simple getauxval stub that returns 0 (feature not available)
    let mut file = File::create(&stub_c).unwrap();
    writeln!(file, "#include <stddef.h>").unwrap();
    writeln!(file, "unsigned long getauxval(unsigned long type) {{").unwrap();
    writeln!(file, "    return 0;  // Indicate feature not available").unwrap();
    writeln!(file, "}}").unwrap();
    drop(file);

    // Compile the stub
    let compile_status = std::process::Command::new("aarch64-linux-gnu-gcc")
        .args([
            "-c",
            stub_c.to_str().unwrap(),
            "-o",
            stub_o.to_str().unwrap(),
            "-O2",
        ])
        .status();

    if compile_status.is_ok() {
        // Create static library
        let ar_status = std::process::Command::new("ar")
            .args(["rcs", stub_a.to_str().unwrap(), stub_o.to_str().unwrap()])
            .status();

        if ar_status.is_ok() {
            println!("cargo:rustc-link-lib=static=getauxval_stub");
        }
    }
}
