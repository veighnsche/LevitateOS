# Phase 3 / Step 2: Syscall Handler

## Goal
Implement the SVC exception handler and syscall dispatch table.

## Parent Context
- [Phase 3](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md)
- [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)

## Design Reference
From Phase 2:
- Syscall number in `x8`, arguments in `x0-x5`
- Return value in `x0`
- Initial syscalls: `write`, `exit`, `getpid`
- Invalid syscall returns `-ENOSYS`

## Units of Work

### UoW 1: SVC Vector Routing
Route SVC exceptions to the syscall handler.

**Tasks:**
1. Modify `kernel/src/exceptions.rs`.
2. In `sync_handler_entry`, check `ESR_EL1` for SVC (EC = 0b010101).
3. If SVC, call `syscall_dispatch` instead of `handle_sync_exception`.
4. Pass `x8` (syscall number) and `x0-x5` (arguments) to handler.

**Exit Criteria:**
- SVC from EL0 routes to `syscall_dispatch`.
- Other sync exceptions still go to `handle_sync_exception`.

### UoW 2: Syscall Dispatch Table
Create the syscall table and dispatch logic.

**Tasks:**
1. Create `kernel/src/syscall.rs`.
2. Define `Syscall` enum with numbers from Phase 2.
3. Implement `syscall_dispatch(nr: u64, args: [u64; 6]) -> i64`.
4. Match on syscall number and call appropriate handler.
5. Return `-ENOSYS` for unknown syscalls.

**Exit Criteria:**
- `syscall_dispatch` compiles and routes syscalls.
- Unknown syscalls return error (not crash).

### UoW 3: Implement `exit` Syscall
First working syscall.

**Tasks:**
1. Implement `sys_exit(code: u64) -> !`.
2. Mark current task as `Exited`.
3. Call `task_exit()` (reuse Phase 7 infrastructure).
4. Log exit code to UART for debugging.

**Exit Criteria:**
- `exit(0)` from userspace cleanly terminates process.
- Kernel continues running.

### UoW 4: Implement `write` Syscall
Console output from userspace.

**Tasks:**
1. Implement `sys_write(fd: u64, buf: *const u8, len: u64) -> i64`.
2. For fd=1 (stdout) or fd=2 (stderr): print to UART.
3. Validate `buf` is in user address space (security!).
4. Return bytes written or `-EFAULT` if invalid buffer.

**Exit Criteria:**
- `write(1, "Hello", 5)` prints "Hello" to UART.
- Invalid buffer returns error (not crash).

## Expected Outputs
- `kernel/src/syscall.rs` with dispatch table.
- Modified `exceptions.rs` routing SVC to syscall handler.
- Working `exit` and `write` syscalls.
