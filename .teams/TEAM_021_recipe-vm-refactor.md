# TEAM_021: Move VM Testing Infrastructure into Recipe Submodule

## Status: Complete

## Problem
The VM testing infrastructure for the recipe crate lived in the wrong place:
- `xtask/src/vm.rs` - in LevitateOS, but specifically for testing recipe
- `.vm/` - in LevitateOS root, gitignored
- `.vm/example-recipes/` - stale duplicate of `recipe/examples/`

This coupling was wrong because:
1. Recipe submodule should be self-contained and independently testable
2. Manual syncing of recipe files was error-prone
3. Anyone cloning just the recipe repo couldn't test it

## Solution
1. Created `recipe/xtask/` with VM management code
2. Moved VM testing into the recipe submodule
3. Deleted stale `.vm/example-recipes/`
4. Simplified LevitateOS xtask (removed recipe-specific VM code)
5. Removed recipe from main workspace (it has its own now)

## New Structure
```
recipe/
  xtask/
    Cargo.toml
    src/
      main.rs    # CLI: cargo xtask vm <command>
      vm.rs      # VM management (QEMU, cloud-init, SSH)
  examples/      # Source of truth for recipes
  .vm/           # Generated VM artifacts (gitignored)
```

## Usage
```bash
cd recipe
cargo xtask vm setup    # Download Arch cloud image
cargo xtask vm prepare  # Build recipe binary
cargo xtask vm start    # Start VM (--gui for display)
cargo xtask vm copy     # Copy recipe + examples to VM
cargo xtask vm ssh      # SSH into VM
cargo xtask vm stop     # Stop VM
```

## Files Created
- `recipe/xtask/Cargo.toml`
- `recipe/xtask/src/main.rs`
- `recipe/xtask/src/vm.rs`

## Files Modified
- `recipe/Cargo.toml` - Added workspace with xtask
- `recipe/.gitignore` - Added /.vm/
- `Cargo.toml` - Removed recipe from main workspace
- `xtask/Cargo.toml` - Removed unused deps
- `xtask/src/main.rs` - Simplified to OS-level tasks only

## Files Deleted
- `Cargo.toml` - Root workspace no longer needed (submodules are standalone)
- `Cargo.lock` - Artifact of old workspace
- `.cargo/config.toml` - Had xtask alias that no longer exists
- `xtask/` - Entire directory (VM testing moved to recipe/xtask)
- `.vm/example-recipes/` - Stale duplicate
