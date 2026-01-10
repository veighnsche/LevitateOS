# Phase 2: Consolidate Build Configuration

## Objective
Ensure all utilities use the workspace-level build configuration consistently.

## Current State (After Phase 1)
```
crates/userspace/eyra/
├── .cargo/config.toml      ← Workspace config with static-pie flags
├── Cargo.toml              ← Workspace manifest
├── Cargo.lock              ← Single authoritative lock
├── target/                 ← Single shared target directory
├── rust-toolchain.toml     ← nightly-2025-04-28
├── cat/
│   ├── Cargo.toml
│   ├── build.rs            ← Still needed for -nostartfiles
│   └── src/
└── ...
```

## Tasks

### Step 1: Verify build.rs files are consistent
Each utility needs build.rs for `-nostartfiles` flag (Eyra provides _start).

Check all utilities have identical build.rs:
```rust
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    // aarch64 libgcc_eh stub if needed
}
```

### Step 2: Standardize Cargo.toml for each utility
Each utility should have minimal Cargo.toml:
```toml
[package]
name = "cat"
version = "0.1.0"
edition = "2021"

[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }
uu_cat = "0.0.29"  # The actual implementation
```

### Step 3: Add .gitignore for individual utility folders
Ensure per-utility target folders don't get recreated:
```
# Add to each utility folder or workspace root
**/target/
!eyra/target/
```

## Success Criteria
- [ ] All utilities use workspace target/
- [ ] All utilities have consistent build.rs
- [ ] No stale artifacts can be recreated
