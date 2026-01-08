# TEAM_246: Implement TTY Termios and IOCTL

**Date:** 2026-01-07  
**Team ID:** TEAM_246  
**Status:** 🏃 IN PROGRESS  
**Objective:** Implement POSIX `termios` support and `SYS_IOCTL` to support terminal configuration.

---

## Progress Track

- [ ] Register Team (Rule 2)
- [x] Create team file `TEAM_246_implement_termios_ioctl.md`
- [x] Define `termios` struct and constants in kernel (`kernel/src/fs/tty/mod.rs`)
- [x] Implement `SYS_IOCTL` dispatcher in `kernel/src/syscall/mod.rs`
- [x] Implement `TCGETS`/`TCSETS` in `kernel/src/syscall/fs/fd.rs`
- [x] Refactor `read_stdin` to use TTY line discipline (ICANON, ECHO, ISIG, etc.)
- [x] Refactor `sys_write` to use TTY output processing (ONLCR)
- [x] Unskip and pass all `tty_test.rs` tests (15/15 passing)
- [x] Verify no regressions in other tests

## Logs

### 2026-01-07 12:45
- Starting work on TTY Termios.
- Verified test baseline: 15/15 core tests pass.
- Goal: Implement `ioctl` for `termios` structure.

---

## Handoff Notes (For Next Team)
TBD
