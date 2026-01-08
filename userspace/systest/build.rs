fn main() {
    // TEAM_304: aarch64-linux-gnu-gcc requires -nostartfiles to avoid crt1.o/crti.o
    let target = std::env::var("TARGET").unwrap_or_default();

    // x86_64 defaults to PIE but we need ET_EXEC for kernel loader
    if target.contains("x86_64") {
        println!("cargo:rustc-link-arg=-no-pie");
    }

    // AArch64 strict freestanding requirement
    if target.contains("aarch64") {
        println!("cargo:rustc-link-arg=-nostartfiles");
    }

    // Try to use common linker script?
    // The previous error showed `-Tlinker.ld` which refers to workspace root linker.ld
    // We explicitly add it here to be safe/consistent if the global one isn't reliable?
    // Actually, let's just stick to fixing the flags first.
}
