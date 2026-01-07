use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_arch == "x86_64" {
        // TEAM_258: Compile assembly boot code for x86_64
        cc::Build::new()
            .file("src/arch/x86_64/boot.S")
            .flag("-fno-PIC")
            .flag("-mno-red-zone")
            .compile("boot");

        // Rerun if assembly changes
        println!("cargo:rerun-if-changed=src/arch/x86_64/boot.S");
        // Rerun if linker script changes
        println!("cargo:rerun-if-changed=src/arch/x86_64/linker.ld");
    } else if target_arch == "aarch64" {
        // AArch64 linker script handling (if needed)
        println!("cargo:rerun-if-changed=src/arch/aarch64/linker.ld");
    }
}
