# TEAM_149: Investigation - Interactive Input Failure

## 1. Pre-Investigation

- **Symptom:** User reports "Typing in the vm shell nothing happens". `run-term.sh` fails (likely secondary cleanup issue). Tests (`serial`, `behavior`) pass.
- **Context:**
  - `sys_read` was modified to use a **Busy-Yield** loop (Poll -> Yield -> Repeat).
  - `init.rs` TimerHandler had `yield_now()` removed (Preemption disabled).
  - User suspects `init.rs`.

## 2. Phase 1 - Understand the Symptom

- **Expected:** Typing in `run-term.sh` echoes characters.
- **Actual:** No response.
- **Delta:** Passiveness/Hang.

## 3. Phase 2 - Hypotheses

### Hypothesis A: Interrupt Starvation (High Confidence)
- **Theory:** Syscalls (Exceptions) in AArch64 automatically **mask (disable) interrupts** (PSTATE.I=1).
- **Evidence:**
  - We are in `sys_read` (Syscall Handler).
  - The loop is: `loop { poll(); yield_now(); }`.
  - `yield_now()` switches tasks but restores their saved Context.
  - If all tasks yield from Syscall context (`init` yields via `sys_yield`, `shell` via `sys_read`), they all save/restore **Masked Interrupts**.
  - Review showed I **removed** the explicit `enable()`/`disable()` block in Step 211.
  - Result: The CPU spins/yields forever with Interrupts DISABLED.
  - Why tests pass: Maybe `serial` test completes fast enough or data is pre-buffered? Or purely luck of timing?

### Hypothesis B: `init` Starvation
- **Theory:** `init` loops tightly on `yield_cpu()`. If the scheduler is round-robin without priorities, and `sys_read` yields, they just trade perfectly.
- **Problem:** This shouldn't prevent input *if* IRQs are enabled. But combined with A, it's fatal.

### Hypothesis C: Userspace Buffer
- **Theory:** Shell userspace buffering is swallowing input.
- **Status:** Unlikely given `sys_read` refactor was the major change.

## 4. Phase 3 - Test & Verification

1. **Inspect `sys_read` state:** Confirm if `enable()` is missing.
2. **Breadcrumb:** Add breadcrumb to `sys_read`.
3. **Fix Plan:** Re-introduce explicit interrupt window in `sys_read` loop.

## 5. Breadcrumbs
- `kernel/src/syscall.rs`

## 6. Decision
- If confirmed, this is a Critical Fix (Rule 14).
