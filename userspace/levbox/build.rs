fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-arg=-Tlevbox/link.ld");

    // TEAM_292: x86_64 defaults to PIE (ET_DYN) but kernel loader expects ET_EXEC
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("x86_64") {
        println!("cargo:rustc-link-arg=-no-pie");
    }
}
