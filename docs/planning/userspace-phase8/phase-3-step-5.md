# Phase 3 / Step 5: Integration & HelloWorld

## Goal
Integrate all components and run the first user program from initramfs.

## Parent Context
- [Phase 3](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md)
- [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)

## Prerequisites
- Step 1 (EL0 Transition) complete
- Step 2 (Syscall Handler) complete
- Step 3 (User Address Space) complete
- Step 4 (ELF Loader) complete

## Units of Work

### UoW 1: Create HelloWorld Binary
Build a minimal statically-linked ELF binary.

**Tasks:**
1. Create `userspace/hello/` directory.
2. Write minimal Rust `#![no_std]` `#![no_main]` binary:
   ```rust
   #[no_mangle]
   pub extern "C" fn _start() -> ! {
       // svc to write "Hello from userspace!"
       // svc to exit(0)
       loop {}
   }
   ```
3. Use `aarch64-unknown-none` target.
4. Link script to set entry at `0x1_0000` (example).
5. Add to initramfs as `/hello`.

**Exit Criteria:**
- `hello` ELF binary in initramfs.
- Binary is valid ELF64 for AArch64.

### UoW 2: Spawn User Process
Implement the `/init` process spawning.

**Tasks:**
1. Create `spawn_user_process(path: &str) -> Result<(), SpawnError>`.
2. Load ELF from initramfs.
3. Create user page table.
4. Load ELF into user address space.
5. Create `UserTask` with entry point and stack.
6. Add task to scheduler.

**Exit Criteria:**
- User process is queued for execution.
- Process structure is complete.

### UoW 3: First User Execution
Run the HelloWorld binary.

**Tasks:**
1. In `kmain`, after boot is complete, call `spawn_user_process("/hello")`.
2. Let scheduler switch to user task.
3. Observe "Hello from userspace!" on UART.
4. Observe clean exit.

**Exit Criteria:**
- HelloWorld prints message.
- Process exits cleanly.
- Kernel continues running.

### UoW 4: Add Behavior Tests
Document behaviors and update inventory.

**Tasks:**
1. Add Group 12 (Userspace) to `behavior-inventory.md`.
2. Document syscall behaviors.
3. Consider adding runtime verification tests.

**Exit Criteria:**
- Behavior inventory updated.
- Test coverage documented.

## Expected Outputs
- `userspace/hello/` with HelloWorld binary.
- `/hello` in initramfs.
- Successful "Hello from userspace!" execution.
- Updated behavior inventory.

## Success Verification

After completing all steps, run:
```bash
cargo xtask test
```

Expected:
- All existing tests pass (no regressions).
- HelloWorld prints to UART.
- Kernel remains stable after user exit.

## Handoff Notes

For the next phase (to implement additional syscalls, fork, exec, etc.):
- Syscall table is in `kernel/src/syscall.rs`
- User task structure is in `kernel/src/task/user.rs`
- ELF loader is in `kernel/src/loader/elf.rs`
- User memory management is in `kernel/src/task/user_mm.rs`
