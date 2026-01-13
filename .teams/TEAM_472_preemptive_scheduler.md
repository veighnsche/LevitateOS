# TEAM_472: Preemptive Scheduler

## Objective

Replace cooperative multitasking with timer-based preemption. Tasks are forcibly switched when their time quantum expires.

## Plan Reference

See `docs/planning/preemptive-scheduler/plan.md` for detailed implementation plan.

## Progress Log

### Session 1 (2026-01-13)

**COMPLETED** - All 5 phases implemented successfully.

#### Phase 1: Data Structures
- Added `QUANTUM_TICKS` constant (10 ticks = 100ms at 100Hz)
- Added `ticks_remaining: AtomicU32` and `total_ticks: AtomicU64` to TaskControlBlock
- Updated Default, From<UserTask>, fork.rs, and thread.rs initializers
- Added `needs_reschedule: AtomicBool` and `preempt_count: AtomicU32` to PCR (both architectures)
- Added `try_current_task()` helper that returns None during early boot

#### Phase 2: Timer Quantum Tracking
- Timer handler (`init.rs:TimerHandler`) now decrements `ticks_remaining`
- Sets `needs_reschedule` flag in PCR when quantum expires
- Removed old TEAM_148 commented-out preemption code
- `switch_to()` resets quantum for incoming task

#### Phase 3: AArch64 Preemption Check
- Added `check_preemption_hook` extern function declaration
- `check_preemption()` in exceptions.rs delegates to kernel hook
- Called from `handle_irq()` after `check_signals()`
- Hook implementation in `main.rs::aarch64_handlers`

#### Phase 4: x86_64 Preemption Check
- Added preemption hook mechanism to `los_hal::x86_64::interrupts::apic`
- Modified `irq_handler` macro to compute `from_userspace` flag and pass to dispatch
- `dispatch_with_preempt()` calls hook after IRQ handling
- `check_preemption_x86()` registered in kernel init

#### Phase 5: PreemptGuard API
- Added `PreemptGuard` RAII struct to `los_sched`
- Increments `preempt_count` on creation, decrements on drop
- Added `preempt_disable()` convenience function
- Check functions skip preemption when `preempt_count > 0`

## Key Decisions

- **Userspace-only preemption** - Don't preempt kernel code initially (simpler, safer)
- **Deferred reschedule** - Timer sets flag, exception return path does switch
- **10-tick quantum** - 100ms at 100Hz timer rate
- **No feature flag** - This is the default behavior for a general-purpose OS
- **Hook pattern** - Both architectures use extern functions for preemption check to avoid circular dependencies

## Files Modified

| File | Changes |
|------|---------|
| `crates/kernel/sched/src/lib.rs` | QUANTUM_TICKS, TCB fields, try_current_task, PreemptGuard |
| `crates/kernel/sched/src/fork.rs` | Initialize quantum fields |
| `crates/kernel/sched/src/thread.rs` | Initialize quantum fields |
| `crates/kernel/arch/aarch64/src/cpu.rs` | PCR: needs_reschedule, preempt_count |
| `crates/kernel/arch/x86_64/src/cpu.rs` | PCR: needs_reschedule, preempt_count |
| `crates/kernel/arch/aarch64/src/exceptions.rs` | check_preemption, extern hook |
| `crates/kernel/lib/hal/src/x86_64/interrupts/apic.rs` | Hook mechanism, dispatch_with_preempt |
| `crates/kernel/lib/hal/src/x86_64/cpu/exceptions.rs` | irq_handler macro, irq_dispatch signature |
| `crates/kernel/levitate/src/init.rs` | Timer quantum tracking, register x86_64 hook |
| `crates/kernel/levitate/src/main.rs` | check_preemption_hook for AArch64 |
| `docs/planning/preemptive-scheduler/plan.md` | Implementation plan |

## Gotchas Discovered

1. **Architecture crate dependencies** - `los_arch_aarch64` doesn't depend on `los_sched`, so preemption check logic must use extern hooks provided by the kernel
2. **x86_64 IRQ dispatch** - The `irq_dispatch` function in los_hal doesn't receive frame or userspace flag, had to modify the macro to compute and pass it
3. **PCR field offsets** - When adding new fields to PCR, must maintain 16-byte alignment

## Verification

- Both x86_64 and aarch64 builds succeed
- Behavior tests pass with expected output changes
- Boot reaches Stage 4 without crashes

## Remaining Work

None - implementation complete. Future enhancements:
- [ ] Priority-based scheduling (nice values)
- [ ] Kernel preemption at safe points
- [ ] SMP support (per-CPU run queues)
- [ ] Real-time policies (SCHED_FIFO, SCHED_RR)

## Handoff Notes

Preemptive scheduling is now the default behavior. The key flow is:

1. Timer fires at 100Hz
2. `TimerHandler::handle()` decrements `ticks_remaining` for current task
3. When quantum expires, sets `needs_reschedule` flag in PCR
4. On return to userspace, `check_preemption` checks flag and switches tasks if needed
5. New task gets fresh quantum via `switch_to()`

To disable preemption in kernel code, use `let _guard = los_sched::preempt_disable();`
