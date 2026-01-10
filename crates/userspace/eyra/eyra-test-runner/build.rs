// TEAM_374: Build script for eyra-test-runner
//
// NOTE: -nostartfiles is now configured at workspace level in .cargo/config.toml
// Individual binaries no longer need to specify it in build.rs
fn main() {
    // -nostartfiles moved to workspace config

    // Create empty libgcc_eh.a stub for aarch64 cross-compilation.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch == "aarch64" {
        let lib_path = format!("{}/libgcc_eh.a", out_dir);
        let status = std::process::Command::new("ar")
            .args(["rcs", &lib_path])
            .status();

        if status.is_ok() {
            println!("cargo:rustc-link-search=native={}", out_dir);
        }
    }
}
