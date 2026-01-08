# Team 253: Implement libsyscall tests

## Goal
Implement the test binaries defined in the `libsyscall-tests` plan.

## Plan
1. Add `readlinkat` wrapper to `libsyscall`.
2. Implement `stat_test.rs`.
3. Implement `link_test.rs`.
4. Implement `time_test.rs`.
5. Implement `sched_yield_test.rs`.
6. Implement `error_test.rs`.
7. Move test binaries to a dedicated `userspace/systest` crate.
8. Register all binaries in `systest/Cargo.toml`.
9. Verify everything builds and passes.

## Status
- [x] Implemented `readlinkat` in `libsyscall`.
- [x] Implemented all 5 test binaries in `userspace/systest/src/bin/`.
- [x] Created `userspace/systest` crate and added to workspace.
- [x] Updated `userspace/levbox/Cargo.toml` to remove test binaries.
- [x] Verified build success for both crates.
