# TEAM_374: Eyra Test Infrastructure

## Objective
Roll back TEAM_371's vendoring mistake and establish proper Eyra testing infrastructure.

## Completed Work

### 1. Rolled Back Vendoring (Root Cause Fix)
- Removed `crates/userspace/eyra/vendor/` (~500 files)
- Removed `crates/userspace/eyra/cat/crates/` (duplicate nested structure)
- Removed `[patch.crates-io]` from Eyra workspace Cargo.toml
- The actual fix was already in place: toolchain isolation in `build_eyra()` using `+nightly-2025-04-28`

### 2. Fixed Eyra Static-PIE Build
- Created `.cargo/config.toml` in Eyra workspace with:
  ```toml
  [target.x86_64-unknown-linux-gnu]
  rustflags = ["-C", "target-feature=+crt-static", "-C", "relocation-model=pic"]
  ```
- Eyra binaries are now `static-pie linked` (no dynamic linker required)

### 3. Increased Stack Size
- Changed `STACK_SIZE` from 64KB to 2MB in `crates/kernel/src/memory/user.rs`
- Required for Eyra/Linux-compatible binaries

### 4. Created Eyra Test Runner
- New package: `crates/userspace/eyra/eyra-test-runner/`
- Tests std library functionality (Vec, String, Box, iterators, env args)
- Outputs `[TEST_RUNNER] RESULT: PASSED/FAILED` for xtask detection

### 5. Updated run-test.sh Infrastructure
- Modified `build_iso_test()` to use test initramfs
- Modified `create_test_initramfs()` to include init, shell, hello.txt, and eyra-test-runner
- Modified init to spawn eyra-test-runner before shell (if present)

### 6. Added init/shell to Userspace Workspace
- Updated `crates/userspace/Cargo.toml` to include init and shell in workspace members

## Known Issues

### Eyra stdout not reaching serial console
The eyra-test-runner spawns and executes syscalls, but its stdout output doesn't appear in the serial console output. This is likely a file descriptor routing issue:
- Bare-metal init/shell use libsyscall which writes directly to fd 1
- Eyra uses std::io which also writes to fd 1
- The fd table inheritance appears correct, but output isn't visible

This needs further investigation into:
1. How Eyra's write() syscall maps to kernel's sys_write
2. Whether stdout is properly inherited from init
3. Whether there's a buffering issue

## Files Modified
- `crates/userspace/eyra/Cargo.toml` - Removed patches, added eyra-test-runner
- `crates/userspace/eyra/.cargo/config.toml` - NEW: static-pie config
- `crates/userspace/eyra/eyra-test-runner/` - NEW: test runner package
- `crates/kernel/src/memory/user.rs` - Increased stack size
- `crates/userspace/Cargo.toml` - Added init/shell to workspace
- `crates/userspace/init/src/main.rs` - Spawn eyra-test-runner before shell
- `xtask/src/build/commands.rs` - Added build_iso_test, updated create_test_initramfs
- `xtask/src/run.rs` - Use build_iso_test for test mode

## Verification
- `./run.sh` builds and boots successfully
- Eyra binaries are static-pie linked
- Init spawns eyra-test-runner when present
- eyra-test-runner starts executing (syscalls visible in kernel log)

## Next Steps
1. Debug stdout routing for Eyra processes
2. Verify write() syscall reaches console for Eyra binaries
3. Once stdout works, tests should pass and run-test.sh will complete successfully

## Handoff Checklist
- [x] Project builds cleanly
- [x] ./run.sh boots to shell
- [ ] ./run-test.sh passes (blocked on stdout issue)
- [x] Team file updated
- [x] Changes documented
