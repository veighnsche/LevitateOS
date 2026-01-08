# Team 251: Refactor libsyscall

## Goal
Refactor `userspace/libsyscall/src/lib.rs` to break it down into smaller, logical modules.

## Plan
We followed the `make-a-refactor-plan` workflow.

## Status
- [x] Planning
- [x] Execution
  - Created key modules: `sysno`, `errno`, `mm`, `process`, `sched`, `time`, `io`, `fs`, `sync`, `signal`, `tty`.
  - Updated `lib.rs` to re-export all symbols.
- [x] Verification
  - `cargo build -p libsyscall` succeeded.
  - `cargo build -p levbox` succeeded (proving API compatibility).

## Changes
- `userspace/libsyscall/src/lib.rs` is now a facade.
- New file structure in `userspace/libsyscall/src/`.
- No changes required in consumers (`core_utils`, `levbox`).
