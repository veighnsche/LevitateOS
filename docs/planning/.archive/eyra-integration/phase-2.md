# Phase 2: Design — Eyra Integration Prerequisites

**TEAM_349** | Eyra Integration Planning  
**Created:** 2026-01-09

---

## 1. Prerequisites Overview

Before integrating Eyra, we must verify/implement these syscalls and features.

### Legend
- ✅ **Verified** — Implemented and matches Linux ABI
- ⚠️ **Needs Verification** — Exists but needs ABI check
- 🔶 **Needs Implementation** — Missing, must add
- ❌ **Blocked** — Depends on other work

---

## 2. Syscall Prerequisites by Priority

### P0 — Absolute Minimum (Hello World)

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `write` | 64 | 1 | ✅ | None |
| `writev` | 66 | 20 | ✅ | None |
| `exit_group` | 94 | 231 | 🔶 | **Implement** — different from `exit` |
| `mmap` | 222 | 9 | ✅ | Verify anonymous mapping |
| `munmap` | 215 | 11 | ✅ | None |
| `mprotect` | 226 | 10 | ✅ | None |
| `brk` | 214 | 12 | ✅ | None |
| `rt_sigaction` | 134 | 13 | ✅ | None |
| `rt_sigprocmask` | 135 | 14 | ✅ | None |

### P1 — Threading Support

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `clone` | 220 | 56 | ✅ | Verify all thread flags |
| `clone3` | 435 | 435 | 🔶 | **Implement** — newer clone API |
| `set_tid_address` | 96 | 218 | ✅ | None |
| `futex` | 98 | 202 | ✅ | Verify all ops |
| `exit` | 93 | 60 | ✅ | None |
| `gettid` | 178 | 186 | ⚠️ | **Verify exists** |
| `tgkill` | 131 | 234 | 🔶 | **Implement** — thread-directed kill |
| `arch_prctl` | N/A | 158 | 🔶 | **x86_64 only** — TLS setup |

### P2 — Filesystem & I/O

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `openat` | 56 | 257 | ✅ | None |
| `close` | 57 | 3 | ✅ | None |
| `read` | 63 | 0 | ✅ | None |
| `readv` | 65 | 19 | ✅ | None |
| `lseek` | 62 | 8 | ⚠️ | **Verify** |
| `fstat` | 80 | 5 | ✅ | None |
| `dup` | 23 | 32 | ✅ | None |
| `dup3` | 24 | 292 | ✅ | None |
| `pipe2` | 59 | 293 | ✅ | None |
| `fcntl` | 25 | 72 | ⚠️ | **Verify/Extend** |
| `ioctl` | 29 | 16 | ✅ | Verify TIOCGWINSZ |
| `readlinkat` | 78 | 267 | ✅ | None |
| `faccessat` | 48 | 269 | 🔶 | **Implement** |

### P3 — Time & Random

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `clock_gettime` | 113 | 228 | ✅ | None |
| `clock_getres` | 114 | 229 | ⚠️ | **Verify/Implement** |
| `nanosleep` | 101 | 35 | ✅ | None |
| `getrandom` | 278 | 318 | 🔶 | **Implement** — critical for std |

### P4 — Process Info

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `getpid` | 172 | 39 | ✅ | None |
| `getppid` | 173 | 110 | ✅ | None |
| `getuid` | 174 | 102 | 🔶 | **Implement** (can return 0) |
| `geteuid` | 175 | 107 | 🔶 | **Implement** (can return 0) |
| `getgid` | 176 | 104 | 🔶 | **Implement** (can return 0) |
| `getegid` | 177 | 108 | 🔶 | **Implement** (can return 0) |

### P5 — Memory (Optional but Recommended)

| Syscall | Nr (aarch64) | Nr (x86_64) | LevitateOS | Action Required |
|---------|--------------|-------------|------------|-----------------|
| `madvise` | 233 | 28 | 🔶 | **Implement stub** |
| `mremap` | 216 | 25 | 🔶 | **Optional** |

---

## 3. Implementation Work Items

### UoW 1: `exit_group` syscall (P0)
**Priority:** Critical  
**Effort:** Small (1-2 hours)

`exit_group` terminates all threads in the process, not just the calling thread.

```rust
// crates/kernel/src/syscall/process.rs
pub fn sys_exit_group(status: i32) -> ! {
    // 1. Signal all threads in this process to exit
    // 2. Clean up process resources
    // 3. Call sys_exit(status)
}
```

**Linux ABI:**
- Nr: 94 (aarch64), 231 (x86_64)
- Args: `int status`
- Returns: Does not return

---

### UoW 2: `getrandom` syscall (P3)
**Priority:** Critical  
**Effort:** Medium (2-4 hours)

Required for `std::collections::HashMap` (random seed), `rand` crate, etc.

```rust
// crates/kernel/src/syscall/misc.rs
pub fn sys_getrandom(buf: usize, buflen: usize, flags: u32) -> i64 {
    // flags: GRND_RANDOM (1), GRND_NONBLOCK (2)
    // Fill buffer with random bytes
    // Use hardware RNG or PRNG seeded at boot
}
```

**Linux ABI:**
- Nr: 278 (aarch64), 318 (x86_64)
- Args: `void *buf, size_t buflen, unsigned int flags`
- Returns: Number of bytes written, or -errno

**Implementation options:**
1. Use AArch64 `RNDR` instruction (ARMv8.5+)
2. Use x86_64 `RDRAND` instruction
3. Fallback to PRNG seeded from timer

---

### UoW 3: User/Group ID syscalls (P4)
**Priority:** Medium  
**Effort:** Small (1 hour)

LevitateOS is single-user, so these can return constants.

```rust
pub fn sys_getuid() -> i64 { 0 }   // root
pub fn sys_geteuid() -> i64 { 0 }
pub fn sys_getgid() -> i64 { 0 }
pub fn sys_getegid() -> i64 { 0 }
```

**Linux ABI:**
- Nr: 174-177 (aarch64), 102/107/104/108 (x86_64)
- Returns: UID/GID as i64

---

### UoW 4: `gettid` syscall (P1)
**Priority:** High  
**Effort:** Trivial

```rust
pub fn sys_gettid() -> i64 {
    crate::task::current_task().id.0 as i64
}
```

**Linux ABI:**
- Nr: 178 (aarch64), 186 (x86_64)
- Returns: Thread ID

---

### UoW 5: `faccessat` syscall (P2)
**Priority:** Medium  
**Effort:** Small (1-2 hours)

Check file accessibility (read/write/execute permissions).

```rust
pub fn sys_faccessat(dirfd: i32, pathname: usize, mode: i32, flags: i32) -> i64 {
    // mode: R_OK (4), W_OK (2), X_OK (1), F_OK (0)
    // For now: check if file exists, return 0 or -ENOENT
}
```

---

### UoW 6: `tgkill` syscall (P1)
**Priority:** Medium  
**Effort:** Small (1-2 hours)

Send signal to specific thread in a process.

```rust
pub fn sys_tgkill(tgid: i32, tid: i32, sig: i32) -> i64 {
    // tgid = process ID, tid = thread ID
    // Verify tgid matches process, then deliver signal to tid
}
```

---

### UoW 7: `arch_prctl` syscall — x86_64 only (P1)
**Priority:** Critical for x86_64  
**Effort:** Medium (2-3 hours)

x86_64 uses `arch_prctl` to set TLS base (FS/GS registers).

```rust
pub fn sys_arch_prctl(code: i32, addr: usize) -> i64 {
    match code {
        ARCH_SET_FS => { /* Set FS base to addr */ }
        ARCH_GET_FS => { /* Write FS base to addr */ }
        ARCH_SET_GS => { /* Set GS base to addr */ }
        ARCH_GET_GS => { /* Write GS base to addr */ }
        _ => errno::EINVAL
    }
}
```

---

### UoW 8: `clock_getres` syscall (P3)
**Priority:** Low  
**Effort:** Trivial

Return clock resolution.

```rust
pub fn sys_clock_getres(clockid: i32, res: usize) -> i64 {
    // Return 1ns resolution for CLOCK_MONOTONIC/REALTIME
}
```

---

### UoW 9: `madvise` stub (P5)
**Priority:** Low  
**Effort:** Trivial

Many allocators call `madvise` but can tolerate it failing.

```rust
pub fn sys_madvise(addr: usize, len: usize, advice: i32) -> i64 {
    // Stub: ignore advice, return success
    0
}
```

---

### UoW 10: Verify `fcntl` coverage (P2)
**Priority:** Medium  
**Effort:** Medium (audit)

Eyra may use `fcntl` for:
- `F_DUPFD` / `F_DUPFD_CLOEXEC`
- `F_GETFD` / `F_SETFD` (close-on-exec)
- `F_GETFL` / `F_SETFL` (file status flags)

---

## 4. Verification Tasks

### V1: Syscall Number Audit
Verify all syscall numbers in `los_abi` match `linux-raw-sys`:
```bash
cargo test -p los_abi
```

### V2: Struct Layout Verification
Add compile-time assertions for critical structs:
```rust
const_assert!(size_of::<Stat>() == 128);
const_assert!(size_of::<Timespec>() == 16);
```

### V3: mmap Flag Support
Verify these flags work:
- `MAP_PRIVATE` (0x02)
- `MAP_ANONYMOUS` (0x20)
- `MAP_FIXED` (0x10) — optional but helpful

### V4: clone Flag Support
Verify these flags work:
- `CLONE_VM` (0x100)
- `CLONE_FS` (0x200)
- `CLONE_FILES` (0x400)
- `CLONE_SIGHAND` (0x800)
- `CLONE_THREAD` (0x10000)
- `CLONE_SETTLS` (0x80000)
- `CLONE_PARENT_SETTID` (0x100000)
- `CLONE_CHILD_CLEARTID` (0x200000)

---

## 5. Implementation Priority Order

```
Phase 3 Implementation Order:

1. exit_group        [P0, Critical, 1h]
2. getrandom         [P0, Critical, 3h]
3. gettid            [P1, High, 0.5h]
4. getuid/geteuid/getgid/getegid [P4, Medium, 1h]
5. arch_prctl        [P1, x86_64 critical, 2h]
6. tgkill            [P1, Medium, 1h]
7. faccessat         [P2, Medium, 1h]
8. clock_getres      [P3, Low, 0.5h]
9. madvise           [P5, Low, 0.5h]

Total estimated: ~12 hours of implementation
```

---

## 6. Open Questions

### Q1: Random Number Generation
**Options:**
- A) Use hardware RNG (`RNDR` on ARM, `RDRAND` on x86)
- B) Use PRNG seeded from boot time + memory layout
- C) Both: hardware if available, fallback to PRNG

**Recommendation:** Option C

### Q2: clone3 vs clone
**Question:** Is `clone3` required, or can Origin use `clone`?

**Impact:** If `clone3` required, need to implement newer API with `struct clone_args`

### Q3: /proc/self/exe
**Question:** Does Eyra need `/proc/self/exe` for `std::env::current_exe()`?

**Impact:** May need basic procfs or workaround

### Q4: Signal Delivery to Threads
**Question:** Does `tgkill` need full per-thread signal queues?

**Recommendation:** Start with simple implementation, enhance if needed

---

## 7. Testing Strategy

### Test 1: Eyra Hello World
```rust
// Build with Eyra
fn main() {
    println!("Hello from Eyra on LevitateOS!");
}
```

### Test 2: Threading
```rust
fn main() {
    let handle = std::thread::spawn(|| {
        println!("Hello from thread!");
    });
    handle.join().unwrap();
}
```

### Test 3: File I/O
```rust
fn main() {
    std::fs::write("/tmp/test.txt", "Hello").unwrap();
    let content = std::fs::read_to_string("/tmp/test.txt").unwrap();
    assert_eq!(content, "Hello");
}
```

### Test 4: Time
```rust
fn main() {
    let start = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(start.elapsed() >= std::time::Duration::from_millis(100));
}
```

---

## 8. Phase 3 Preview

Once prerequisites are implemented:
1. Create Eyra test binary in `userspace/eyra-test/`
2. Build with `-Zbuild-std` and custom target
3. Add to initramfs
4. Boot and verify output

---

## 9. References

| Resource | Link |
|----------|------|
| Eyra repo | https://github.com/sunfishcode/eyra |
| Origin (startup) | https://github.com/sunfishcode/origin |
| rustix (syscalls) | https://github.com/bytecodealliance/rustix |
| linux-raw-sys | https://crates.io/crates/linux-raw-sys |
| LevitateOS std-support archive | `docs/planning/.archive/std-support/` |
