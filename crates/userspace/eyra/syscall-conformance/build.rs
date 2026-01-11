// TEAM_424: Required for Eyra-based binaries
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
}
