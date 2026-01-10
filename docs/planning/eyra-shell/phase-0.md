# Phase 0: Prerequisite — Implement epoll Syscalls

## Objective

Implement epoll syscalls in the LevitateOS kernel to enable tokio async runtime for brush shell.

## Why This Is Required

brush uses **tokio** for async operations (command execution, signal handling, etc.).
tokio requires **epoll** (or poll) for event-driven I/O on Linux.

Without epoll, brush will fail at runtime when tokio tries to create its event loop.

## Syscalls to Implement

| Syscall | Number (x86_64) | Number (aarch64) | Purpose |
|---------|-----------------|------------------|---------|
| `epoll_create1` | 291 | 20 | Create epoll instance |
| `epoll_ctl` | 233 | 21 | Add/modify/remove fd from epoll |
| `epoll_wait` | 232 | 22 | Wait for events |
| `eventfd2` | 290 | 19 | Inter-thread signaling (**required** for tokio) |

## Implementation Steps

### Step 1: Add Syscall Numbers
- [ ] Add `__NR_epoll_create1`, `__NR_epoll_ctl`, `__NR_epoll_wait`, `__NR_eventfd2` to `libsyscall/src/sysno.rs`
- [ ] Add syscall dispatch in kernel `syscall/mod.rs`

### Step 2: Implement Epoll Subsystem
- [ ] Create `crates/kernel/src/syscall/epoll.rs`
- [ ] Define `EpollInstance` structure (fd → event mapping)
- [ ] Implement `sys_epoll_create1(flags)` → returns epoll fd
- [ ] Implement `sys_epoll_ctl(epfd, op, fd, event)` → add/mod/del
- [ ] Implement `sys_epoll_wait(epfd, events, maxevents, timeout)` → wait for events

### Step 2b: Implement eventfd
- [ ] Implement `sys_eventfd2(initval, flags)` → returns eventfd
- [ ] EventFd structure with counter and waitqueue
- [ ] Read decrements counter (blocks if zero)
- [ ] Write increments counter (wakes waiters)

### Step 3: Integrate with VFS
- [ ] Epoll fds need to be tracked in process file descriptor table
- [ ] Hook into existing fd close path to clean up epoll registrations

### Step 4: Test Basic Functionality
- [ ] Create test program that uses epoll directly
- [ ] Verify epoll_create1, epoll_ctl, epoll_wait work
- [ ] Test with pipe fds (read/write readiness)

### Step 5: Test with Tokio (Optional)
- [ ] Create minimal tokio test program
- [ ] Verify tokio runtime initializes
- [ ] Verify basic async operations work

## Estimated Effort

| Step | Complexity | Time |
|------|------------|------|
| Step 1 | Low | 1-2 hours |
| Step 2 | Medium-High | 1-2 days |
| Step 3 | Medium | 4-8 hours |
| Step 4 | Low | 2-4 hours |
| Step 5 | Low | 2-4 hours |

**Total: 2-4 days**

## Success Criteria

- [ ] `epoll_create1` returns valid fd
- [ ] `epoll_ctl` can add/modify/remove fds
- [ ] `epoll_wait` blocks and returns events correctly
- [ ] Pipe read/write readiness detection works
- [ ] Basic tokio program runs (optional but recommended)

## References

- Linux epoll(7) man page: https://man7.org/linux/man-pages/man7/epoll.7.html
- epoll_create1(2): https://man7.org/linux/man-pages/man2/epoll_create1.2.html
- epoll_ctl(2): https://man7.org/linux/man-pages/man2/epoll_ctl.2.html
- epoll_wait(2): https://man7.org/linux/man-pages/man2/epoll_wait.2.html

## Notes

This phase can be done independently and benefits more than just brush:
- Any tokio-based application
- Any async Rust application using mio
- Future async services in LevitateOS
