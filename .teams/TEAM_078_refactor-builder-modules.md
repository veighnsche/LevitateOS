# TEAM_078: Refactor builder.rs into Smaller Modules

## Goal
Split the 589-line `builder.rs` into focused modules for better organization and testability.

## Changes

### New Files
- `src/rootfs/builder/mod.rs` - RootfsBuilder struct + build orchestration
- `src/rootfs/builder/tarball.rs` - create_tarball, list_tarball, extract_tarball
- `src/rootfs/builder/verify.rs` - verify_tarball and verification lists

### Modified Files
- `src/rootfs/mod.rs` - Update exports to include new public functions

### Deleted Files
- `src/rootfs/builder.rs` - Replaced by builder/ directory

## Verification
```bash
cd leviso && cargo build
```

Public API remains unchanged.
