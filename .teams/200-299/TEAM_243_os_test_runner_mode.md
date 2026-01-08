# TEAM_243: OS Test Runner Mode

## Mission
Create a variant of the OS that runs internal tests and outputs everything to stdout for AI agent verification.

## Status
**Phase:** COMPLETE ✅

## Context
- User wants `run-test.sh` like `run-term.sh` but for automated OS internal testing
- Target: AI agents can verify OS functionality by parsing stdout
- Internal tests exist in userspace: `mmap_test`, `signal_test`, `pipe_test`, `clone_test`

## Progress Log

### 2026-01-07 - Implementation Complete
- Created `test_runner` binary in levbox
- Added `cargo xtask run test` command
- Created `run-test.sh` convenience script
- All 4 internal tests passing: mmap_test, pipe_test, signal_test, clone_test

## Files Created/Modified
- `userspace/levbox/src/bin/test_runner.rs` - NEW: Test orchestrator
- `userspace/levbox/Cargo.toml` - Added test_runner binary
- `xtask/src/run.rs` - Added Test variant and run_qemu_test()
- `xtask/src/build.rs` - Added create_test_initramfs()
- `xtask/src/main.rs` - Wired up run test command
- `run-test.sh` - NEW: Convenience script
- `docs/planning/os-test-runner/` - Planning docs

## Usage
```bash
# Run all internal OS tests
cargo xtask run test
# OR
./run-test.sh
```

## Test Output Format (AI-parseable)
```
[TEST_RUNNER] SUMMARY
[TEST_RUNNER]   mmap_test: PASS
[TEST_RUNNER]   pipe_test: PASS
[TEST_RUNNER]   signal_test: PASS
[TEST_RUNNER]   clone_test: PASS
[TEST_RUNNER] Total: 4/4 tests passed
[TEST_RUNNER] RESULT: PASSED
```

## Known Issues
- `cargo xtask test behavior` fails due to golden file drift (pre-existing, unrelated to this feature)

## Handoff Notes
- Feature complete and tested
- Planning documents in `docs/planning/os-test-runner/`
