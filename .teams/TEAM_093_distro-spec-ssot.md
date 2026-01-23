# TEAM_093: distro-spec SSOT Optimization

**Status**: PHASE 1-4 COMPLETE
**Started**: 2026-01-23

## Objective

Make distro-spec the actual Single Source of Truth (SSOT) with compile-time enforcement. Eliminate duplication between levitate and acorn modules by extracting shared types.

---

## Completed Work

### Phase 1: Extract Shared Boot Types (COMPLETE)

Created `distro-spec/src/shared/boot.rs`:
- `BootEntry` struct with builder methods
- `LoaderConfig` struct with builder methods
- Shared constants: `ESP_MOUNT_POINT`, `LOADER_CONF_PATH`, `ENTRIES_DIR`, `DEFAULT_TIMEOUT`
- `bootctl_install_command()` function

Updated `levitate/boot.rs`:
- Re-exports shared types
- Keeps `BOOT_MODULES` (distro-specific kernel modules)
- Added convenience functions: `default_boot_entry()`, `default_loader_config()`, etc.

Updated `acorn/boot.rs`:
- Re-exports shared types
- Added `BOOT_MODULES` (Alpine-specific, with .ko.gz extensions)
- Added convenience functions matching levitate API

**Lines eliminated:** ~95 (duplicated BootEntry/LoaderConfig implementations)

### Phase 2: Create ServiceManager Trait (COMPLETE)

Created `distro-spec/src/shared/services.rs`:
```rust
pub trait ServiceManager {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required(&self) -> bool;
    fn enable_command(&self) -> String;
    fn disable_command(&self) -> String;
    fn start_command(&self) -> String;
    fn stop_command(&self) -> String;
}
```

Updated `levitate/services.rs`:
- Implements `ServiceManager` for `ServiceSpec`
- Commands: `systemctl enable/disable/start/stop`

Updated `acorn/services.rs`:
- Implements `ServiceManager` for `ServiceSpec`
- Commands: `rc-update add/del`, `rc-service start/stop`

### Phase 3: Complete AcornOS Foundation (COMPLETE)

Added to `acorn/paths.rs`:
- `ISO_LABEL = "ACORNOS"`
- `SQUASHFS_COMPRESSION = "gzip"`
- `SQUASHFS_BLOCK_SIZE = "1M"`
- `SQUASHFS_NAME = "filesystem.squashfs"`
- `SQUASHFS_CDROM_PATH`

Added to `acorn/boot.rs`:
- `BOOT_MODULES` array (Alpine kernel module paths with .ko.gz)

### Phase 4: Documentation (COMPLETE)

Updated `distro-spec/CLAUDE.md`:
- Architecture diagram
- Why structure decisions
- How to add new distro variant
- Usage examples
- Consumer list

### Phase 5: Consumer Updates (COMPLETE)

Updated `install-tests/src/steps/phase5_boot.rs`:
- Use `default_boot_entry()` instead of `BootEntry::with_root()`
- Use `default_loader_config()` instead of `LoaderConfig { ..Default::default() }`
- Import `ServiceManager` trait for `enable_command()` method

---

## STILL BLOCKED (Awaiting TEAM_092)

The following changes require modifying leviso, which is being refactored by TEAM_092:

- Make leviso import `ISO_LABEL` from distro-spec
- Make leviso import `SQUASHFS_*` from distro-spec
- Remove `BOOT_MODULES` duplication in `leviso/src/artifact/initramfs.rs`

---

## Verification

```bash
# All pass:
cargo build -p distro-spec     # ✓
cargo build -p install-tests   # ✓
cargo clippy -p distro-spec    # ✓ (no warnings)
cargo doc -p distro-spec       # ✓
```

---

## Files Modified

### New Files
| File | Lines |
|------|-------|
| `src/shared/boot.rs` | ~195 |
| `src/shared/services.rs` | ~40 |

### Modified Files
| File | Changes |
|------|---------|
| `src/shared/mod.rs` | Added boot, services modules |
| `src/levitate/boot.rs` | Removed structs, re-exports, kept BOOT_MODULES |
| `src/levitate/mod.rs` | Added new exports |
| `src/levitate/services.rs` | Implemented ServiceManager |
| `src/acorn/boot.rs` | Removed structs, re-exports, added BOOT_MODULES |
| `src/acorn/mod.rs` | Added new exports (ISO_LABEL, SQUASHFS_*, etc.) |
| `src/acorn/paths.rs` | Added ISO_LABEL, SQUASHFS_*, SQUASHFS_CDROM_PATH |
| `src/acorn/services.rs` | Implemented ServiceManager |
| `src/lib.rs` | Added shared boot and services re-exports |
| `CLAUDE.md` | Complete rewrite with architecture docs |
| `testing/install-tests/src/steps/phase5_boot.rs` | Updated to new API |

---

## Result

```
distro-spec/src/
├── lib.rs
├── shared/
│   ├── mod.rs
│   ├── boot.rs        # BootEntry, LoaderConfig (SHARED)
│   ├── chroot.rs
│   ├── partitions.rs
│   ├── services.rs    # ServiceManager trait (NEW)
│   └── users.rs
├── levitate/
│   ├── mod.rs
│   ├── boot.rs        # Re-exports + BOOT_MODULES + constructors
│   ├── paths.rs
│   └── services.rs    # ServiceSpec + ServiceManager impl
└── acorn/
    ├── mod.rs
    ├── boot.rs        # Re-exports + BOOT_MODULES + constructors
    ├── paths.rs       # ISO_LABEL, SQUASHFS_* (COMPLETE)
    └── services.rs    # ServiceSpec + ServiceManager impl
```
