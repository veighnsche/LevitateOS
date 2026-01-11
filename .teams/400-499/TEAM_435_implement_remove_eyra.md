# TEAM_435: Implement Remove Eyra Refactor

## Objective

Execute the TEAM_433 plan to remove Eyra and replace with c-gull sysroot approach.

## Status: COMPLETE

## Plan Reference

`docs/planning/refactor-remove-eyra/` (Phases 3-5)

Reviewed by TEAM_434.

## Progress Log

### Session 1 (2026-01-11)
- Created implementation team file
- Executed Phase 3 migration steps
- Executed Phase 4 cleanup
- Verified builds pass

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution order | Follow plan exactly | Plan reviewed and approved |
| Build from source | Yes (for now) | No dynamic linker yet |
| Future: binary downloads | Document for later | Once ld.so implemented |
| Skip syscall-conformance move | Skip | Deleted with eyra directory |

## Phase 3 Progress

- [x] Step 1: Move libsyscall to `crates/userspace/libsyscall`
- [x] Step 2: Add xtask sysroot.rs and external.rs modules
- [x] Step 3: Update create_initramfs() to use toolchain paths
- [x] Step 4: Move syscall-conformance tests (SKIPPED - deleted with eyra)
- [x] Step 5: Rename test files (SKIPPED - deleted with eyra)
- [x] Step 6: Delete eyra directory
- [x] Step 7: Update .gitmodules (coreutils submodule entry removed)

## Phase 4 Progress

- [x] Delete dead code files
  - EYRA_INTEGRATION_COMPLETE.md
  - docs/development/eyra-porting-guide.md
  - scripts/build-eyra.sh
  - .windsurf/workflows/eyra-test-runner.md
  - tests/EYRA_TESTING_README.md
  - tests/eyra_*.{rs,sh,txt}
  - xtask/src/tests/eyra.rs
- [x] Update xtask commands (removed Eyra, added Sysroot/Coreutils/Brush)
- [x] Remove build_eyra() function

## Phase 5 Progress

- [x] Final build verification - kernel builds
- [x] Test suite passes - unit tests pass
- [x] Coreutils built and included in initramfs
- [x] Updated CLAUDE.md with c-gull instructions
- [x] Updated golden file to reflect new initramfs contents

## Gotchas Discovered

1. **Init depends on libsyscall path**: Had to update `crates/userspace/init/Cargo.toml` to point to new libsyscall location
2. **submodule deinit before rm**: Must run `git submodule deinit -f` before `git rm` for submodules
3. **Stale binaries in target directory**: After deleting source code, pre-built binaries remain in `crates/userspace/target/` and get copied to initramfs. Clean them manually or run `cargo clean` in userspace

## Files Changed

### New Files
- `xtask/src/build/sysroot.rs` - c-gull sysroot build commands
- `xtask/src/build/external.rs` - External project (coreutils, brush) build commands

### Modified Files
- `crates/userspace/Cargo.toml` - Added libsyscall to workspace members
- `crates/userspace/init/Cargo.toml` - Updated libsyscall path
- `crates/userspace/libsyscall/Cargo.toml` - Updated comments
- `xtask/src/build/mod.rs` - Added sysroot and external modules
- `xtask/src/build/commands.rs` - Replaced Eyra with c-gull approach
- `xtask/src/main.rs` - Updated build command handlers
- `xtask/src/tests/mod.rs` - Removed eyra module

### Deleted Files
- `crates/userspace/eyra/` (entire directory)
- Various eyra-related test/doc files

## Remaining Work

All tasks complete. See handoff notes below.

## Future Enhancement

**Binary Downloads (after ld.so implementation)**:
Once LevitateOS has a dynamic linker, we can download pre-built binaries instead of building from source:
- Download musl-static binaries (if available from upstream)
- Or download standard Linux binaries (requires libc.so)
This eliminates the clone+build step entirely.

## Handoff Notes

The Eyra removal is complete. Key changes:
1. `crates/userspace/eyra/` is gone
2. `libsyscall` is now at `crates/userspace/libsyscall/`
3. Build coreutils with: `cargo xtask build coreutils`
4. Build brush with: `cargo xtask build brush`
5. Build sysroot with: `cargo xtask build sysroot`

Coreutils/brush are cloned from upstream at build time (gitignored).

**Current initramfs contents** (x86_64):
- `init` - bare-metal init process
- `hello-cgull` - c-gull libc test binary
- `coreutils` - multi-call binary (2.4MB)
- Symlinks: cat, echo, head, mkdir, pwd, rm, tail, touch

**Note**: After removing Eyra source, clean stale binaries with:
```bash
rm -f crates/userspace/target/*/release/eyra-*
```
