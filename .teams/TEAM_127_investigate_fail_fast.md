# TEAM_127: Investigate 'Fail Fast' Violation

## Symptom
User reported that relaxing the behavior test (masking the missing shell prompt) was a "false positive" and violated "FAIL FAST" principles.
The shell prompt (`# `) is missing from the serial output in `tests/actual_boot.txt`.
Previous team (TEAM_126) assumed this was intentional ("shell outputs to GPU"), but the user's reaction suggests otherwise.

## Context
- File: `/home/vince/Projects/LevitateOS/.agent/rules/kernel-development.md` (likely Rule 14).
- Test File: `xtask/src/tests/behavior.rs`

## Action Plan
1. Revert the changes to `xtask/src/tests/behavior.rs` that turned the error into a warning.
2. Confirm the test fails again (Fail Fast).
3. Investigate why `stdout` from the shell is not reaching the serial console.
    - Hypothesis A: Userspace `println!` only writes to one descriptor (GPU/Terminal) and ignores UART.
    - Hypothesis B: Kernel `sys_write` or similar is not multiplexing output.
