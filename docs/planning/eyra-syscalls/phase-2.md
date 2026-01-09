# Phase 2: Design — Eyra Syscalls

**TEAM_359** | Eyra Syscalls (ppoll, tkill, pkey_alloc)  
**Created:** 2026-01-09

---

## 1. ppoll Implementation

### 1.1 Linux ABI

```c
int ppoll(struct pollfd *fds, nfds_t nfds,
          const struct timespec *tmo_p,
          const sigset_t *sigmask);
```

**Syscall Numbers:**
- x86_64: 271
- aarch64: 73

**Arguments:**
- `fds` — User pointer to array of pollfd structs
- `nfds` — Number of file descriptors
- `tmo_p` — User pointer to timeout (NULL = block forever)
- `sigmask` — Signal mask (can ignore for MVP)

**Returns:**
- Number of fds with events, or 0 on timeout, or -errno

### 1.2 struct pollfd

```rust
#[repr(C)]
pub struct Pollfd {
    pub fd: i32,       // File descriptor
    pub events: i16,   // Requested events
    pub revents: i16,  // Returned events
}
```

**Size:** 8 bytes

### 1.3 Poll Events

```rust
pub const POLLIN: i16 = 0x0001;     // Data to read
pub const POLLPRI: i16 = 0x0002;    // Urgent data
pub const POLLOUT: i16 = 0x0004;    // Writing possible
pub const POLLERR: i16 = 0x0008;    // Error (output only)
pub const POLLHUP: i16 = 0x0010;    // Hang up (output only)
pub const POLLNVAL: i16 = 0x0020;   // Invalid fd (output only)
```

### 1.4 Implementation Strategy

**MVP Approach (non-blocking):**
1. Parse pollfd array from user space
2. For each fd, check current state:
   - Stdin: POLLIN if input available
   - Stdout/Stderr: Always POLLOUT
   - Pipes: POLLIN if data available, POLLOUT if not full
   - Files: Always POLLIN | POLLOUT
3. Set revents for each fd
4. Return count of fds with non-zero revents

**Timeout Handling:**
- If `tmo_p` is NULL → block forever (but we can return immediately for MVP)
- If `tmo_p` is zero → non-blocking poll
- If `tmo_p` is positive → sleep then re-check

**Signal Mask:** Ignore for MVP (most callers don't use it)

### 1.5 Implementation Code

```rust
// crates/kernel/src/syscall/sync.rs

pub fn sys_ppoll(fds_ptr: usize, nfds: usize, tmo_ptr: usize, _sigmask: usize) -> i64 {
    let task = crate::task::current_task();
    
    // Validate fds buffer
    let pollfd_size = core::mem::size_of::<Pollfd>();
    let buf_size = nfds * pollfd_size;
    if mm_user::validate_user_buffer(task.ttbr0, fds_ptr, buf_size, true).is_err() {
        return errno::EFAULT;
    }
    
    // Read and process each pollfd
    let mut ready_count = 0i64;
    for i in 0..nfds {
        let pfd_addr = fds_ptr + i * pollfd_size;
        let mut pfd = read_pollfd(task.ttbr0, pfd_addr);
        
        // Check fd state
        pfd.revents = poll_fd_state(pfd.fd, pfd.events);
        
        if pfd.revents != 0 {
            ready_count += 1;
        }
        
        // Write back revents
        write_pollfd_revents(task.ttbr0, pfd_addr, pfd.revents);
    }
    
    ready_count
}
```

---

## 2. tkill Implementation

### 2.1 Linux ABI

```c
int tkill(int tid, int sig);
```

**Syscall Numbers:**
- x86_64: 200
- aarch64: 130

**Arguments:**
- `tid` — Thread ID to signal
- `sig` — Signal number

**Returns:**
- 0 on success, -errno on failure

### 2.2 Implementation Strategy

Reuse existing signal infrastructure from `sys_kill`, but target thread ID instead of process ID.

```rust
// crates/kernel/src/syscall/signal.rs

pub fn sys_tkill(tid: i32, sig: i32) -> i64 {
    // tid == 0 is invalid for tkill
    if tid <= 0 {
        return errno::EINVAL;
    }
    
    // sig == 0 means just check if thread exists
    if sig == 0 {
        return if task_exists(tid as usize) { 0 } else { errno::ESRCH };
    }
    
    // Find thread and deliver signal
    match find_task_by_tid(tid as usize) {
        Some(task) => {
            deliver_signal_to_task(&task, sig);
            0
        }
        None => errno::ESRCH,
    }
}
```

---

## 3. pkey_alloc Implementation

### 3.1 Linux ABI

```c
int pkey_alloc(unsigned int flags, unsigned int access_rights);
```

**Syscall Numbers:**
- x86_64: 302
- aarch64: 289

**Arguments:**
- `flags` — Must be 0
- `access_rights` — PKEY_DISABLE_ACCESS or PKEY_DISABLE_WRITE

**Returns:**
- Protection key (>= 0), or -errno

### 3.2 Implementation Strategy

**Stub Only:** Memory protection keys require hardware support (Intel PKU) and significant kernel infrastructure. Return -ENOSYS to indicate not supported.

```rust
// crates/kernel/src/syscall/mm.rs

pub fn sys_pkey_alloc(_flags: u32, _access_rights: u32) -> i64 {
    // Memory protection keys not supported
    errno::ENOSYS
}
```

---

## 4. Implementation Order

| Step | Syscall | Effort | Files |
|------|---------|--------|-------|
| 1 | `pkey_alloc` | 10 min | mm.rs, mod.rs, arch mods |
| 2 | `tkill` | 30 min | signal.rs, mod.rs, arch mods |
| 3 | `ppoll` | 2 hours | sync.rs, mod.rs, arch mods |

---

## 5. Open Questions

### Q1: ppoll Blocking Behavior

**Question:** Should ppoll block waiting for events, or return immediately?

**Options:**
- A) Always return immediately (non-blocking poll)
- B) Implement proper blocking with timeout
- C) Start with A, enhance to B if needed

**Recommendation:** Option C — Eyra may work with non-blocking poll initially

---

### Q2: tkill Thread Lookup

**Question:** How do we look up a thread by TID?

**Current State:** `current_task()` exists, but no `find_task_by_tid()`

**Options:**
- A) Add TID lookup to task scheduler
- B) Iterate task list to find matching TID
- C) Return ESRCH always (stub)

**Recommendation:** Option B for MVP

---

### Q3: ppoll Signal Mask

**Question:** Should we handle the sigmask parameter?

**Impact:** Most callers pass NULL. Handling it properly requires atomic signal mask swapping.

**Recommendation:** Ignore sigmask for MVP, document as limitation

---

## 6. Verification

After implementation, run:
```bash
cargo xtask test eyra --arch x86_64
```

**Expected result:** No unknown syscall warnings, Eyra prints `[OK]` markers.

---

## 7. Phase 3 Preview

Implementation order:
1. Add syscall numbers to arch modules
2. Implement `sys_pkey_alloc` stub
3. Implement `sys_tkill`
4. Implement `sys_ppoll`
5. Wire into dispatcher
6. Test with Eyra
