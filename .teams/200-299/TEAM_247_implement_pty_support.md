# Team 247: Implement PTY Support

## Objective
Implement Pseudo-Terminal (PTY) support to enable terminal emulation and better shell interaction.

## Progress
- [x] Research PTY architecture in Unix/Linux.
- [x] Implement `PtyPair` and PTY allocation in `kernel/src/fs/tty/pty.rs`.
- [x] Add PTY master/slave support to `FdType`.
- [x] Implement `TIOCGPTN` and `TIOCSPTLCK` ioctls.
- [x] Implement bidirectional communication between master and slave.
- [x] Update `TtyState` to support output redirection for PTY echoing.
- [x] Create `pty_test` userspace binary to verify functionality.
- [x] Update `libsyscall` with PTY-compatible wrappers (`openat`, `ioctl`).
- [x] Verified all tests pass.

## Remaining TODOs
- [ ] Implement `TIOCGWINSZ` and `TIOCSWINSZ` for window sizing.
- [ ] Integrate shell to use PTY for sub-processes.
