# TEAM_002: Investigate Headless Verification Failure

## 0. Pre-Investigation
- **Team ID**: TEAM_002
- **Bug Summary**: Headless verification script `verify_headless.sh` is likely failing. User is also manually deleting `build.rs` and `boot.S` which suggests they might be interfering with the build.

## 1. Phase 1 — Understand the Symptom
- **Expected Behavior**: `verify_headless.sh` should finish with "✓ Graphics Verification: SUCCESS".
- **Actual Behavior**: Pending run of the script.
- **Delta**: Pending.
- **Location**: `rust/scripts/verify_headless.sh`.

## 2. Phase 2 — Form Hypotheses
- **Hypothesis 1**: Old `build.rs` or `boot.S` files in the `rust/` directory are causing build conflicts or incorrect kernel generation.
- **Hypothesis 2**: The string "Drawing complete" is not being printed to the UART, or UART is not configured correctly in the headless run.
- **Hypothesis 3**: The kernel is crashing before reaching the drawing code.

## 3. Phase 3 — Test Hypotheses with Evidence
- [x] Run `verify_headless.sh` and check logs.
    - **Result**: SUCCESS (after manual cleanup by user).
    - **Serial Output**: Confirmed GPU and Input initialization.
- [x] Check `src/main.rs` and `c/kernel/arch/arm64/boot.S`.
    - **Result**: Confirmed `boot.S` in `rust/` root would provide duplicate `_start` and `_head` symbols. `main.rs` already contains its own boot assembly.

## 4. Phase 4 — Narrow Down to Root Cause
- **Root Cause**: Build pollution. Redundant `boot.S` and `build.rs` files in the `rust/` root directory caused symbol conflicts with the Rust-native assembly in `main.rs`.
- **Secondary Issue**: `println!` (via `core::fmt`) is not re-entrant or thread-safe, posing a risk (deadlocks/Sync Exceptions) if used in interrupt-polled loops or actual interrupt handlers.

## 5. Phase 5 — Decision: Fix or Plan
- **Decision**: Fix is already applied by user manual cleanup. No further code changes required to the build system right now, but a thread-safe `println!` is recommended.
- **Handoff**:
    - [x] Project builds cleanly (verified with `verify_headless.sh`)
    - [x] All tests pass
    - [x] Root cause documented
