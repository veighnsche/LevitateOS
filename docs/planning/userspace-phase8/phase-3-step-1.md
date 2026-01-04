# Phase 3 / Step 1: EL0 Transition

## Goal
Implement the mechanism to transition from kernel mode (EL1) to user mode (EL0).

## Parent Context
- [Phase 3](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md)
- [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)

## Design Reference
From Phase 2:
- Use `eret` instruction to enter EL0
- Configure `SPSR_EL1` for EL0 with interrupts enabled
- Set `ELR_EL1` to user entry point
- Set `SP_EL0` for user stack

## Units of Work

### UoW 1: User Entry Assembly
Create the assembly routine to perform the EL0 transition.

**Tasks:**
1. Create `kernel/src/task/user.rs`.
2. Implement `enter_user_mode(entry: usize, sp: usize) -> !`.
3. Configure `SPSR_EL1` = 0 (EL0, interrupts enabled).
4. Set `ELR_EL1` to entry point.
5. Set `SP_EL0` to user stack pointer.
6. Execute `eret`.

**Exit Criteria:**
- Function compiles.
- Can be called (but will crash without syscall handler).

### UoW 2: User Task Creation
Extend `TaskControlBlock` to support user tasks.

**Tasks:**
1. Add `UserTask` struct or extend `TaskControlBlock` with user fields.
2. Add `ttbr0` field for user page table.
3. Add `user_sp` field for user stack pointer.
4. Implement `UserTask::new(entry_point, stack, ttbr0)`.

**Exit Criteria:**
- User task can be created with allocated stack.

### UoW 3: Test Transition (Minimal)
Verify EL0 entry works (even though it will immediately fault).

**Tasks:**
1. Create a test that calls `enter_user_mode` with a simple address.
2. Expect a sync exception (since no user code exists yet).
3. Verify `ESR_EL1` shows exception from EL0.

**Exit Criteria:**
- Exception handler shows EL0 origin.
- System does not hang on EL0 entry.

## Expected Outputs
- `kernel/src/task/user.rs` with `enter_user_mode`.
- Extended `TaskControlBlock` for user tasks.
- Verification that EL0 transition occurs.
