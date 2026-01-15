# TEAM 487: Builder Structure Refactor

## Status: Complete

## Goal
Refactor the builder crate structure to reflect the new Fedora-based workflow. The old `components/` directory no longer makes sense since we're not building multiple vendor components anymore.

## Current Structure (Before)
```
builder/
├── auth/           # User/group/PAM config
│   ├── mod.rs
│   ├── nss.rs
│   ├── pam.rs
│   └── users.rs
├── components/     # OLD: Multiple buildable components
│   ├── mod.rs      # Buildable trait definition
│   ├── registry.rs # Component registry (just Linux now)
│   ├── glibc.rs    # Library collection
│   └── linux.rs    # Kernel build
├── fedora.rs       # ISO extraction and binary lists
├── initramfs.rs    # Initramfs assembly
├── mod.rs          # Module exports
└── vendor.rs       # Vendor source definitions (stale)
```

## New Structure (After)
```
builder/
├── auth/           # Keep as-is
│   ├── mod.rs
│   ├── nss.rs
│   ├── pam.rs
│   └── users.rs
├── kernel.rs       # Kernel build (moved from components/linux.rs)
├── libraries.rs    # Library collection (renamed from glibc.rs)
├── fedora.rs       # ISO extraction and binary lists
├── initramfs.rs    # Initramfs assembly
└── mod.rs          # Simplified module exports
```

## Changes
1. Remove `components/` directory entirely
2. Move `components/linux.rs` → `kernel.rs` (simplified, no Buildable trait)
3. Move `components/glibc.rs` → `libraries.rs` (clearer name)
4. Delete `components/mod.rs` (Buildable trait no longer needed)
5. Delete `components/registry.rs` (no longer needed)
6. Delete `vendor.rs` (references to deleted vendor sources)
7. Update `mod.rs` to reflect new structure
8. Update `initramfs.rs` to use new module paths

## Rationale
The "component" abstraction made sense when building 11+ vendor projects from source. Now we only have:
- Kernel: Built from source (special case)
- Everything else: Copied from Fedora ISO

This refactor makes the codebase match the actual workflow.
