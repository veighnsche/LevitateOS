// TEAM_430: Build script for c-gull test
// Required for Eyra: -nostartfiles flag

fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
}
