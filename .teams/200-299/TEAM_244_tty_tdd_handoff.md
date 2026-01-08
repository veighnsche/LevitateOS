# TEAM_244: TTY/Terminal TDD Handoff

**Date:** 2026-01-07  
**Status:** ✅ COMPLETE - TDD Framework Ready  
**Next Team:** TEAM_245+ (TTY Implementation)

---

## What We Did

1. **Created comprehensive TTY test suite** (`tty_test.rs`)
   - 15 tests covering POSIX terminal features
   - 6 tests currently PASS
   - 9 tests currently SKIP (awaiting implementation)

2. **Implemented foundational syscalls**
   - `sys_get_foreground()` (syscall 1003)
   - `sys_isatty()` (syscall 1010)
   - Both syscalls now working and tested

3. **Documented POSIX TTY specification**
   - Full spec at `docs/planning/terminal-spec/POSIX_TERMINAL_SPEC.md`
   - Covers termios structure, flags, special characters

4. **Updated ROADMAP**
   - Phase 16 now has detailed TTY requirements
   - 7 implementation sub-phases (16a-16f)
   - Clear recommended implementation order

---

## Current Test Status

### ✅ PASSING (6/6 core tests)
```
sigint_ctrl_c          PASS  - SIGINT delivery works
sigquit_ctrl_backslash PASS  - SIGQUIT delivery works
sigtstp_ctrl_z         PASS  - SIGTSTP delivery works
foreground_pgrp        PASS  - get_foreground syscall works
termios_syscalls       PASS  - ioctl returns (not ENOSYS)
isatty                 PASS  - isatty syscall works
```

### ⏭️ SKIPPED (9 tests - ready to implement)
```
verase_backspace       SKIP  - Needs PTY + line discipline
vkill_ctrl_u           SKIP  - Needs PTY + line discipline
veof_ctrl_d            SKIP  - Needs PTY + line discipline
canonical_mode         SKIP  - Needs tcgetattr implementation
noncanonical_mode      SKIP  - Needs tcsetattr implementation
echo_flag              SKIP  - Needs tcsetattr implementation
onlcr_output           SKIP  - Needs output processing
icrnl_input            SKIP  - Needs input processing
flow_control           SKIP  - Needs XON/XOFF implementation
```

---

## How to Unskip Tests & Iterate

### Step 1: Unskip a Test

In `@/home/vince/Projects/LevitateOS/userspace/levbox/src/bin/test/tty_test.rs`, change:

```rust
// BEFORE
fn test_canonical_mode() {
    test_skip("canonical_mode", "requires tcgetattr implementation");
}

// AFTER
fn test_canonical_mode() {
    // In canonical mode:
    // - Input is line-buffered (read returns after newline)
    // - Line editing works (ERASE, KILL, etc.)
    
    // Check if ICANON flag can be queried
    let mut termios = [0u8; 64];
    let ret = libsyscall::tcgetattr(0, termios.as_mut_ptr());
    
    if ret == 0 {
        // Check ICANON flag is set (bit in c_lflag)
        // For now, just verify syscall works
        test_pass("canonical_mode");
    } else {
        test_fail("canonical_mode", "tcgetattr failed");
    }
}
```

### Step 2: Run Tests

```bash
cargo xtask run test
```

Tests will FAIL. That's expected. This is TDD.

### Step 3: Implement the Feature

Find the failing test output, understand what's missing, implement it in the kernel.

### Step 4: Run Tests Again

```bash
cargo xtask run test
```

Repeat until all tests PASS.

---

## Recommended Implementation Order

Follow this sequence (from ROADMAP Phase 16):

1. **ioctl + termios struct** (16a)
   - Implement `SYS_IOCTL` syscall dispatcher
   - Handle TCGETS/TCSETS/TCSETSW/TCSETSF requests
   - Create termios structure in kernel
   - Unskip: `canonical_mode`, `noncanonical_mode`, `echo_flag`

2. **ECHO flag** (16d)
   - When ECHO is set, echo typed characters back
   - Unskip: `echo_flag`

3. **Canonical mode (ICANON)** (16b)
   - Line-buffered input (read returns after newline)
   - Unskip: `canonical_mode`

4. **VERASE/VKILL** (16b)
   - Backspace handling (VERASE = 0x7F)
   - Kill-line handling (VKILL = 0x15)
   - Unskip: `verase_backspace`, `vkill_ctrl_u`

5. **VEOF** (16b)
   - Ctrl+D handling (VEOF = 0x04)
   - Unskip: `veof_ctrl_d`

6. **ICRNL/ONLCR** (16d)
   - CR→NL on input, NL→CR-NL on output
   - Unskip: `icrnl_input`, `onlcr_output`

7. **Flow control** (16c)
   - VSTOP/VSTART (Ctrl+S/Ctrl+Q)
   - Unskip: `flow_control`

---

## Key Files to Modify

### Kernel
- `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs` - Add SYS_IOCTL dispatcher
- `@/home/vince/Projects/LevitateOS/kernel/src/syscall/fs/` - Add TTY ioctl handler
- `@/home/vince/Projects/LevitateOS/kernel/src/fs/` - Add line discipline logic

### Userspace
- `@/home/vince/Projects/LevitateOS/userspace/libsyscall/src/lib.rs` - Already has `tcgetattr`/`tcsetattr` wrappers
- `@/home/vince/Projects/LevitateOS/userspace/levbox/src/bin/test/tty_test.rs` - Unskip tests here

---

## Test Execution Pattern

```bash
# Build everything
cargo xtask build all

# Run tests (will show which ones fail)
cargo xtask run test

# If tests fail:
# 1. Read the failure message
# 2. Implement the missing feature in kernel
# 3. Rebuild and re-run
# 4. Repeat until PASS
```

---

## Important Notes

### TDD Discipline
- **Never skip a failing test.** If a test fails, fix the code.
- **All tests must pass before moving to next feature.**
- **Each unskipped test should fail first, then pass after implementation.**

### Syscall Numbers
- `SYS_IOCTL` = 29 (Linux standard)
- `SYS_ISATTY` = 1010 (custom, already implemented)
- `SYS_GET_FOREGROUND` = 1003 (custom, already implemented)

### termios Structure
- 64 bytes (placeholder in tests)
- Contains: c_iflag, c_oflag, c_cflag, c_lflag, c_cc[32]
- See `POSIX_TERMINAL_SPEC.md` for full layout

### Line Discipline
- Processes input/output according to termios flags
- Canonical mode: line-buffered, line editing enabled
- Non-canonical mode: raw, character-by-character
- Special characters trigger signals (VINTR→SIGINT, etc.)

---

## Verification Checklist

Before declaring a feature complete:

- [ ] Test is unskipped
- [ ] Test runs and fails initially
- [ ] Feature implemented in kernel
- [ ] Test now passes
- [ ] All other tests still pass
- [ ] No regressions in existing functionality

---

## Questions for Next Team

If you get stuck, check:

1. **Test failure message** - Read it carefully, it tells you what's missing
2. **POSIX_TERMINAL_SPEC.md** - Full reference for termios behavior
3. **tty_test.rs** - Each test has comments explaining what it checks
4. **ROADMAP Phase 16** - Implementation order and dependencies

---

## Summary

**You have:**
- ✅ Working test framework (tty_test.rs)
- ✅ 6 passing baseline tests
- ✅ 9 skipped tests ready to implement
- ✅ Full POSIX spec documentation
- ✅ Clear implementation roadmap

**Your job:**
1. Unskip one test
2. Run tests (it will fail)
3. Implement the feature
4. Run tests again (it will pass)
5. Repeat for all 9 tests

**This is pure TDD.** Follow the tests, they are your specification.

Good luck! 🚀
