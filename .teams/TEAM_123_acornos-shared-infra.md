# TEAM_123: AcornOS Shared Infrastructure Parity

## Goal
Audit and fix gaps where AcornOS is NOT using shared infrastructure from `distro-builder` and `distro-spec` that leviso uses.

## Status: COMPLETE

## Changes Made

### 1. initramfs.rs - Use `create_initramfs_dirs()` and `atomic_move()`

**Before:**
```rust
// Manual directory creation
for dir in INITRAMFS_DIRS {
    fs::create_dir_all(root.join(dir))?;
}

// Direct fs::rename
fs::rename(&temp_cpio, &output_cpio)?;
```

**After:**
```rust
use distro_builder::artifact::filesystem::{create_initramfs_dirs, atomic_move};

// Use shared infrastructure
create_initramfs_dirs(&initramfs_root, INITRAMFS_DIRS)?;

// Atomic move with cross-filesystem fallback
atomic_move(&temp_cpio, &output_cpio)?;
```

### 2. iso.rs - Use `atomic_move()` for artifacts

**Before:**
```rust
fs::rename(&temp_iso, &paths.iso_output)?;
let _ = fs::rename(&temp_checksum, &final_checksum);
```

**After:**
```rust
use distro_builder::artifact::filesystem::atomic_move;

atomic_move(&temp_iso, &paths.iso_output)?;
let _ = atomic_move(&temp_checksum, &final_checksum);
```

### 3. Verified: ServiceManager trait implemented

`distro_spec::acorn::services::ServiceSpec` already implements `ServiceManager` trait (confirmed in code).

### 4. Decision: Keep custom preflight checking

AcornOS's `host_tools.rs` has better UX with install suggestions per-tool. The shared `check_required_tools()` is more generic. Keeping the custom implementation.

## Not Changed (Future Work)

- `create_fhs_structure()` - AcornOS uses declarative component system instead
- `BootEntry`, `LoaderConfig` - Needed when AcornOS supports disk installation
- `PartitionLayout` - Needed when AcornOS supports disk installation
- `CHROOT_BIND_MOUNTS` - Needed when AcornOS supports recstrap-style installation

## Verification

```bash
cd AcornOS && cargo build
cargo run -- build
```
