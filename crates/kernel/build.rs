use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

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

        // TEAM_274: Set linker script via build.rs instead of root .cargo/config.toml
        // This ensures it only applies to the kernel crate, not userspace
        // Use absolute path based on CARGO_MANIFEST_DIR
        println!(
            "cargo:rustc-link-arg=-T{}/src/arch/x86_64/linker.ld",
            manifest_dir
        );
        println!("cargo:rustc-link-arg=-no-pie");
        println!("cargo:rustc-link-arg=-zmax-page-size=0x1000");
    } else if target_arch == "aarch64" {
        // AArch64 linker script handling (linker script might be implicitly handled or default used)
        println!("cargo:rerun-if-changed=src/arch/aarch64/linker.ld");

        // TEAM_304: -nostartfiles required to avoid generic libc startup files (crt1.o)
        println!("cargo:rustc-link-arg=-nostartfiles");
    }
}
