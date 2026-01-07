# TEAM_242: Investigate Ctrl+C Failure

**Created**: 2026-01-07
**Symptom**: Ctrl+C does not interrupt `signal_test` running in `sys_pause()`.
**Context**:
- VirtIO Input Interrupt handler implemented (TEAM_241).
- `MAX_HANDLERS` increased to 34 to fix "Unhandled IRQ 78".
- IRQ 78 is now handled (messages gone).

## Hypotheses

1. **H1: Ctrl+C detection failed in ISR**: `poll()` might not be seeing the key or `check_and_signal` logic is flawed.
2. **H2: `FOREGROUND_PID` mismatch**: The `signal_test` process might not be registered as the foreground process.
3. **H3: Signal queuing failed**: `sys_kill` might be failing silently or the task state isn't updating.

## Investigation Plan

1. Instrument `kernel/src/input.rs`:
   - Log entry into `InputInterruptHandler::handle`.
   - Log when Ctrl+C is detected in `poll`.
2. Instrument `kernel/src/syscall/signal.rs`:
   - Log `fg_pid` in `signal_foreground_process`.
   - Log result of `sys_kill`.
3. Run `signal_test` and check logs.
