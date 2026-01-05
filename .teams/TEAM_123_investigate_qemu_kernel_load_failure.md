# Team 123: Investigate QEMU Kernel Load Failure

## Phase 1: Understand the Symptom

### Symptom Description
User runs `bash ./run.sh`.
Build completes successfully.
QEMU launch fails with:
`qemu-system-aarch64: could not load kernel ''`

This implies the kernel path argument passed to QEMU is an empty string.

### Location
- Tool: `xtask`
- Command: `qemu-system-aarch64` argument generation.

### Trace
1. `run.sh` calls `xtask build` then presumably `xtask run`? Or `xtask` handles the run.
2. `xtask` builds the kernel.
3. `xtask` constructs QEMU arguments.
4. The path variable for the kernel seems to be empty or missing.

## Phase 2: Form Hypotheses

1. **Hypothesis 1:** The `run` subcommand in `xtask` fails to locate the built kernel binary path. -> **RULED OUT** (`xtask` is not used for running here).
2. **Hypothesis 2:** The kernel binary name changed (e.g., `levitate-kernel` vs `kernel`) and `xtask` hasn't been updated. -> **RULED OUT**.
3. **Hypothesis 3:** `run.sh` invokes `xtask` in a way that skips path detection. -> **CONFIRMED**. `run.sh` does not use `xtask run`. It manually invokes QEMU and uses an undefined variable `$BIN`.

## Phase 3: Test Hypotheses with Evidence

### Evidence
- File `run.sh` line 24: `-kernel "$BIN"`
- Variable `BIN` is never defined.
- `xtask` (lines 238-246) produces `kernel64_rust.bin`.

### Root Cause
`run.sh` tries to run the kernel using an uninitialized variable `$BIN`.

## Phase 4: Decision: Fix or Plan
**Fix immediately.**
The fix is trivial (define `BIN=kernel64_rust.bin`).

## Phase 5: Resolution

Fixed immediately by defining `BIN="kernel64_rust.bin"` in `run.sh`.

## Handoff Checklist

- [x] Team file is updated with symptom, hypotheses, and resolution.
- [x] Breadcrumbs placed (marked as CONFIRMED in team file).
- [x] Fix implemented in `run.sh`.


