# Phase 1: Discovery - OS Test Runner Mode

## Feature Summary

Create a test runner variant of LevitateOS that:
1. Boots the OS in headless mode
2. Runs internal tests (userspace test binaries) automatically
3. Outputs all results to stdout/serial
4. Exits with appropriate exit code for CI/AI agent consumption

**Problem:** Currently, testing OS internals requires manual shell interaction or complex behavior test infrastructure. AI agents need a simple way to run tests and verify results from stdout.

**Who benefits:** 
- AI agents testing OS changes
- CI/CD pipelines
- Developers wanting quick verification

## Success Criteria

1. `cargo xtask run test` or `./run-test.sh` boots OS and runs all internal tests
2. All test output appears on stdout (serial)
3. OS exits with code 0 on success, non-zero on failure
4. Test results follow a parseable format: `[TEST] name: PASS/FAIL`
5. Total runtime < 30 seconds for full test suite

## Current State Analysis

### Existing Infrastructure

**Run modes (run.sh variants):**
- `run-gui.sh` - GUI with SDL window
- `run-term.sh` - Terminal-only, WSL-like (keyboard to stdin)
- `run-vnc.sh` - VNC for browser verification

**Test infrastructure:**
- `xtask test behavior` - Runs OS, captures serial to file, compares to golden
- `xtask test unit` - Rust unit tests (host-side)
- Userspace test binaries exist but must be run manually via shell

**Userspace test binaries (in initramfs):**
- `mmap_test` - Memory mapping syscall tests
- `signal_test` - Signal handling tests  
- `pipe_test` - Pipe/IPC tests
- `clone_test` - Process cloning tests

### Current Workaround
Users must:
1. Boot OS with `run-term.sh`
2. Wait for shell prompt
3. Manually type test commands: `mmap_test`, `signal_test`, etc.
4. Visually inspect output

## Codebase Reconnaissance

### Files to Modify/Create

| File | Purpose |
|------|---------|
| `xtask/src/run.rs` | Add `Test` variant to `RunCommands` |
| `xtask/src/main.rs` | Wire up `run test` command |
| `run-test.sh` | Shell script wrapper (like run-term.sh) |
| `userspace/init/src/main.rs` | Add test-mode init variant OR... |
| `userspace/levbox/src/bin/test_runner.rs` | **NEW** - unified test runner binary |

### Key Code Paths

1. **Boot sequence:** `kernel/src/init.rs::run()` → spawns init → init spawns shell
2. **Init behavior:** `userspace/init/src/main.rs` - currently just spawns shell
3. **Test binaries:** `userspace/levbox/src/bin/*.rs` - self-contained tests

### Tests/Golden Files Impacted

- `tests/golden_boot.txt` - Should NOT change (test mode is separate)
- May need new golden file: `tests/golden_test.txt` for test runner output

## Constraints

1. **No kernel modifications required** - This can be userspace-only
2. **Must preserve existing behavior** - `run-term.sh` unchanged
3. **Timeout required** - Tests must complete within reasonable time
4. **Clean exit** - OS must shutdown cleanly after tests

## Design Options Preview

### Option A: Test Runner Init
- Modify init to check for "test mode" flag
- If test mode, run tests instead of shell
- Pros: Simple, no new binaries
- Cons: Complicates init

### Option B: Dedicated Test Runner Binary
- Create `test_runner` binary in levbox
- Init spawns `test_runner` instead of `shell` in test mode
- Pros: Clean separation
- Cons: Need mechanism to select mode

### Option C: Kernel Feature Flag
- `--features test-mode` changes kernel behavior
- Kernel spawns test runner directly
- Pros: Most control
- Cons: Kernel changes, separate builds

**Recommendation:** Option B - cleanest separation, leverages existing spawn infrastructure

## Open Questions

1. **Q1:** How should test mode be activated?
   - a) Kernel command line argument
   - b) Separate init binary (`init-test`)
   - c) Environment variable / flag file in initramfs
   
2. **Q2:** Should tests run in parallel or sequentially?
   - Sequential is safer, parallel is faster
   
3. **Q3:** How to signal test completion to QEMU?
   - a) Special serial output pattern that xtask detects
   - b) QEMU `-device isa-debug-exit` (x86 only, not ARM)
   - c) Timeout-based (current behavior test approach)
   - d) poweroff syscall / PSCI shutdown

## Next Steps

→ Move to Phase 2: Design after questions answered or assumptions made
