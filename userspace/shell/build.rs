fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-arg=-Tshell/link.ld");

    // TEAM_292: x86_64 defaults to PIE (ET_DYN) but kernel loader expects ET_EXEC
    // TEAM_304: aarch64-linux-gnu-gcc requires -nostartfiles to avoid crt1.o/crti.o
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("x86_64") {
        println!("cargo:rustc-link-arg=-no-pie");
    }
    if target.contains("aarch64") {
        println!("cargo:rustc-link-arg=-nostartfiles");
    }
}
