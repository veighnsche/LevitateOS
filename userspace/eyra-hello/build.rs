// TEAM_351: Tell the linker not to use the system startup code.
// Eyra provides its own _start implementation via Origin.
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
}
