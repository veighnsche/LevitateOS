# Phase 1: Discovery — Eyra Syscalls

**TEAM_359** | Eyra Syscalls (ppoll, tkill, pkey_alloc)  
**Created:** 2026-01-09

---

## 1. Feature Summary

### Problem Statement

The Eyra test (`cargo xtask test eyra`) revealed three missing syscalls that cause Eyra/std to panic during initialization:

```
[SYSCALL] Unknown syscall number: 302
[SYSCALL] Unknown syscall number: 271
[SYSCALL] Unknown syscall number: 200
EXCEPTION: INVALID OPCODE  # Rust panic (ud2)
```

### Who Benefits

- Users running Rust std binaries compiled with Eyra
- Any Linux-compatible binary that uses ppoll, tkill, or memory protection keys

### Success Criteria

1. `cargo xtask test eyra` runs without unknown syscall warnings
2. Eyra test prints success markers (`[OK]`)
3. No regressions in existing tests

---

## 2. Syscall Analysis

### 2.1 ppoll (271 x86_64 / 73 aarch64)

**Purpose:** Wait for events on file descriptors with nanosecond timeout precision.

**Linux Signature:**
```c
int ppoll(struct pollfd *fds, nfds_t nfds,
          const struct timespec *tmo_p,
          const sigset_t *sigmask);
```

**Why Eyra Needs It:**
- Used by Rust std's I/O system for non-blocking operations
- Called during stdout/stderr initialization
- Required for `println!` to work

**Complexity:** Medium — requires pollfd parsing and timeout handling

---

### 2.2 tkill (200 x86_64 / 130 aarch64)

**Purpose:** Send a signal to a specific thread (not process).

**Linux Signature:**
```c
int tkill(int tid, int sig);
```

**Why Eyra Needs It:**
- Used for thread-directed signals
- Called by `pthread_kill` equivalent
- May be called during thread cleanup

**Complexity:** Low — similar to existing `kill` syscall but targets thread ID

**Note:** Linux also has `tgkill(tgid, tid, sig)` which is more secure. Eyra may use either.

---

### 2.3 pkey_alloc (302 x86_64 / 289 aarch64)

**Purpose:** Allocate a memory protection key.

**Linux Signature:**
```c
int pkey_alloc(unsigned int flags, unsigned int access_rights);
```

**Why Eyra Needs It:**
- Part of Intel Memory Protection Keys (MPK) feature
- Eyra may probe for it during initialization
- Can fail gracefully (not required for operation)

**Complexity:** Low — can stub to return -ENOSYS or -EOPNOTSUPP

---

## 3. Codebase Reconnaissance

### Files to Modify

| File | Changes |
|------|---------|
| `crates/kernel/src/arch/x86_64/mod.rs` | Add syscall numbers |
| `crates/kernel/src/arch/aarch64/mod.rs` | Add syscall numbers |
| `crates/kernel/src/syscall/mod.rs` | Add dispatch cases |
| `crates/kernel/src/syscall/sync.rs` | Add `sys_ppoll` |
| `crates/kernel/src/syscall/signal.rs` | Add `sys_tkill` |
| `crates/kernel/src/syscall/mm.rs` | Add `sys_pkey_alloc` (stub) |

### Related Existing Code

- `sys_kill` in `signal.rs` — Pattern for tkill
- `sys_futex` in `sync.rs` — Pattern for timeout handling
- `sys_nanosleep` in `time.rs` — Timespec parsing

---

## 4. Constraints

### Must Support
- x86_64 and aarch64 architectures
- Linux-compatible ABI (struct layouts, return values)

### Can Defer
- Full signal mask handling in ppoll (can ignore sigmask initially)
- pkey_alloc can return -EOPNOTSUPP (feature not supported)

---

## 5. Implementation Priority

```
1. ppoll    [P0, ~2h] — Blocks Eyra initialization
2. tkill    [P1, ~30min] — Thread signals
3. pkey_alloc [P2, ~10min] — Stub only

Total: ~3 hours
```

---

## 6. Next Steps (Phase 2)

1. Design ppoll implementation strategy
2. Define struct pollfd layout
3. Determine which poll events to support
4. Document questions for user
