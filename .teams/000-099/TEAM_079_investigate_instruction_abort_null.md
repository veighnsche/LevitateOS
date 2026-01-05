# TEAM_079: Investigate Instruction Abort at Address 0x0

## Status
**COMPLETE** - Bug fixed

## Bug Report

### Symptom
User process hits instruction abort at address 0x0 after entering user mode.

### Error Output
```
*** USER EXCEPTION ***
Exception Class: 0x20
ESR: 0x0000000082000007
ELR (fault address): 0x0000000000000000
Type: Instruction Abort
Terminating user process.
```

### Expected Behavior
User process executes "hello" ELF binary and prints "Hello from userspace!"

### Actual Behavior
User process jumps to address 0x0 and faults.

### Environment
- QEMU virt machine, AArch64
- Kernel with TEAM_078 MMIO fix applied
- "hello" ELF binary loaded from initramfs

### Reproduction
```bash
cargo xtask run
```

## Investigation Log

### Phase 1: Understand the Symptom
- Exception Class 0x20 = Instruction Abort from lower exception level
- ESR 0x82000007 = IFSC (Instruction Fault Status Code) indicates translation fault
- ELR = 0x0 = The CPU tried to execute code at NULL
- This means the entry point or a branch led to address 0x0

### Code Areas to Investigate
1. ELF loader - is entry point parsed correctly?
2. User page table setup - is code mapped correctly?
3. ERET to user mode - are registers set correctly?
4. The "hello" ELF binary itself - is it valid?

## Hypotheses

1. **Entry point passed as 0** - RULED OUT (logs showed entry=0x10000)
2. **Registers cleared before use in enter_user_mode** - CONFIRMED (Bug 1)
3. **Page mapping permission issue** - CONFIRMED (Bug 2)

## Root Cause Analysis

### Bug 1: enter_user_mode register clobbering
- `enter_user_mode()` cleared registers x2-x30 BEFORE using input operands
- Compiler could place `entry_point` in any register via `in(reg)`
- If placed in x2-x30, value was zeroed before `msr elr_el1, {entry}`
- **Fix:** Set system registers FIRST, then clear GPRs

### Bug 2: ELF loader page flag overwriting  
- Two PT_LOAD segments shared page 0x10000:
  - .text at 0x10000 (R E) → mapped with USER_CODE (executable)
  - .rodata at 0x10040 (R) → remapped same page with USER_DATA (non-executable)
- Second mapping overwrote first, removing execute permission
- ESR 0x8200000f = Permission fault, level 3
- **Fix:** Skip mapping pages that are already mapped

## Files Modified

- `kernel/src/task/user.rs` - Fixed enter_user_mode to set system registers before clearing GPRs
- `kernel/src/loader/elf.rs` - Skip remapping already-mapped pages

## Verification

```
Hello from userspace!
LevitateOS Phase 8: Userspace support working!
[SYSCALL] exit(0)
```

Userspace program executes successfully and exits cleanly.
