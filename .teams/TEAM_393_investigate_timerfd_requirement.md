# TEAM_393 — Investigate: Does tokio require timerfd?

**Created:** 2026-01-10  
**Status:** ✅ COMPLETE  
**Context:** eyra-shell plan Phase 0 lists epoll + eventfd but not timerfd. Phase 1 mentions timerfd as potentially needed. Need to verify.

## Question

Does tokio's async runtime require `timerfd_create` / `timerfd_settime` syscalls, or can it function with just epoll + eventfd?

## Answer

**NO — tokio does NOT require timerfd.**

## Evidence

From official Tokio documentation:

> "The timer is implemented in **user land** (i.e., **does not use an operating system timer like timerfd on linux**). It uses a **hierarchical hashed timer wheel** implementation, which provides efficient constant time complexity when creating, canceling, and firing timeouts."

**Source:** https://v0-1--tokio.netlify.app/docs/going-deeper/timers/

## How Tokio Timers Work

1. **Timer wheel** — Userland data structure tracks all pending timers
2. **epoll_wait timeout** — Uses the timeout parameter of `epoll_wait()` for sleeping
3. **No timerfd syscalls** — Does not use `timerfd_create`, `timerfd_settime`, `timerfd_gettime`

## Syscalls Actually Needed for Tokio

| Syscall | Required? | Purpose |
|---------|-----------|---------|  
| `epoll_create1` | ✅ YES | Event loop |
| `epoll_ctl` | ✅ YES | Register/unregister fds |
| `epoll_wait` | ✅ YES | Wait for events (timeout param for timers) |
| `eventfd2` | ✅ YES | Thread wakeups |
| `timerfd_*` | ❌ NO | Not used by tokio |

## Note on tokio-timerfd crate

There is a third-party crate called `tokio-timerfd` but it:
- Is **NOT part of the tokio project**
- Is an **optional alternative** for applications needing nanosecond precision
- brush shell does NOT require it

## Conclusion

Phase 0 of eyra-shell plan is **correct** — only epoll + eventfd are needed. timerfd is NOT required.

## Action

Remove timerfd from Phase 1's syscall requirements table (it was listed as potentially needed).
