// TEAM_374: Tell the linker not to use the system startup code.
// Eyra provides its own _start implementation via Origin.
// This fixes the duplicate _start symbol conflict with Scrt1.o.
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");

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
