# Phase 2: Structural Extraction (Eyra Setup)

**TEAM_362** | Refactor Userspace to Eyra/std  
**Created:** 2026-01-09

---

## 1. Target Design

### 1.1 New Directory Structure

```
crates/userspace/
├── eyra/                   # Eyra-based apps (std, static-pie)
│   ├── rust-toolchain.toml # Shared nightly toolchain
│   ├── eyra-hello/         # Test binary (reference)
│   ├── cat/
│   ├── cp/
│   ├── init/
│   ├── ls/
│   ├── mkdir/
│   ├── mv/
│   ├── pwd/
│   ├── rm/
│   ├── rmdir/
│   ├── ln/
│   ├── shell/
│   └── touch/
├── libsyscall/             # KEEP: Custom syscall wrappers
└── Cargo.toml              # Updated (remove old members)
```

### 1.2 Shared Eyra Configuration

All apps share the same `rust-toolchain.toml`:
```toml
[toolchain]
channel = "nightly-2025-04-28"
components = ["rustc", "cargo", "rust-std", "rust-src"]
targets = ["x86_64-unknown-linux-gnu"]
```

### 1.3 Standard Cargo.toml Pattern

```toml
[package]
name = "app-name"
version = "0.1.0"
edition = "2021"

# Exclude from parent workspace - uses different toolchain
[workspace]

# Eyra provides std via pure-Rust Linux syscalls
[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }

[profile.release]
panic = "abort"
opt-level = "s"
lto = true
strip = true

[unstable]
build-std-features = ["panic_immediate_abort"]
```

**Required in main.rs:**
```rust
extern crate eyra;  // Required for -Zbuild-std compatibility
```

---

## 2. Extraction Strategy

### 2.1 Order of Operations

1. **Create apps/ directory structure**
2. **Create shared rust-toolchain.toml**
3. **Create template Cargo.toml**
4. **Migrate one app as proof of concept (cat)**
5. **Verify it works in LevitateOS**
6. **Proceed with remaining apps**

### 2.2 Coexistence Strategy

During migration:
- Old apps in `init/`, `shell/`, `levbox/` (broken, won't build)
- New apps in `apps/` (building with Eyra)
- Once all migrated, delete old directories

---

## 3. Build System Changes

### 3.1 New Build Command

```bash
# Build single app
cd crates/userspace/eyra/cat
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# Build all apps
for app in init shell cat cp ls mkdir mv pwd rm rmdir ln touch; do
  (cd crates/userspace/eyra/$app && cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort)
done
```

### 3.2 xtask Integration

Add to xtask:
```rust
// Build all Eyra userspace apps
pub fn build_userspace_apps() -> Result<()> {
    let apps = ["init", "shell", "cat", "cp", "ls", "mkdir", "mv", "pwd", "rm", "rmdir", "ln", "touch"];
    for app in apps {
        build_eyra_app(app)?;
    }
    Ok(())
}
```

### 3.3 Initramfs Integration

Update `make_initramfs.sh` to copy Eyra binaries:
```bash
# Copy Eyra apps to initramfs
for app in init shell cat cp ls mkdir mv pwd rm rmdir ln touch; do
  cp crates/userspace/eyra/$app/target/x86_64-unknown-linux-gnu/release/$app $STAGING/
done
```

---

## 4. Phase 2 Steps

### Step 1: Create Directory Structure
- Create `crates/userspace/eyra/` subdirectories for each app
- Create subdirectory for each app

### Step 2: Create Shared Configuration
- Create `rust-toolchain.toml` template
- Create `Cargo.toml` template

### Step 3: Migrate cat as PoC
- Create `crates/userspace/eyra/cat/` with Eyra
- Rewrite cat.rs with std
- Build and test

### Step 4: Verify Integration
- Add cat to initramfs
- Boot LevitateOS
- Run `cat /etc/motd` (or similar)
