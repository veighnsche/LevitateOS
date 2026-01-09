# TEAM_347 — Investigate std Support Requirements

**Created:** 2026-01-09
**Status:** Complete

## Summary

LevitateOS has **significant std support infrastructure already implemented**. The path to full `std` support is clear and well-documented in archived planning docs.

---

## Current State (What's Done)

| Category | Status | Implementation |
|----------|--------|----------------|
| **Auxv (P0)** | ✅ Complete | TEAM_217 - `AT_PAGESZ`, `AT_HWCAP`, `AT_RANDOM`, etc. |
| **mmap/munmap (P0)** | ✅ Complete | TEAM_228/238 - Anonymous private mappings |
| **mprotect (P0)** | ✅ Complete | TEAM_239 - Page protection changes |
| **clone (P1)** | ✅ Complete | TEAM_230 - Thread-style clones (CLONE_VM) |
| **TLS (P1)** | ✅ Complete | TPIDR_EL0 context-switched |
| **set_tid_address (P1)** | ✅ Complete | TEAM_228 - Clear-child-tid support |
| **futex (P1)** | ✅ Complete | TEAM_208 - WAIT/WAKE operations |
| **writev/readv (P1)** | ✅ Complete | TEAM_217 - Vectored I/O |
| **pipe2 (P2)** | ✅ Complete | TEAM_233 |
| **dup/dup3 (P2)** | ✅ Complete | TEAM_233 |
| **Signals** | ✅ Complete | kill, sigaction, sigprocmask, sigreturn, pause |
| **ioctl** | ✅ Complete | TTY operations |
| **Linux ABI Syscalls** | ✅ Complete | TEAM_345 - openat, mkdirat, unlinkat, etc. |

---

## What's Missing for Full std Support

### 1. **Build a std-compatible binary** (Phase 7)

The kernel syscall support is largely complete. What's needed is:

1. **Custom target JSON** for LevitateOS
2. **Build std for the target** (cross-compile Rust std)
3. **Test with a simple std program**

### 2. **Remaining Verification (Low Priority)**

| Item | Status | Notes |
|------|--------|-------|
| Termios struct layout | ⚠️ Unverified | Compare with Linux |
| Stat struct alignment | ⚠️ Unverified | Compile-time assertions |
| Timespec struct | ⚠️ Unverified | Compare with linux-raw-sys |

### 3. **Fork-style clone** (Optional)

Currently only thread-style clones (CLONE_VM) are supported. Full fork() would need:
- Copy-on-write page tables
- Not required for `std::thread`

### 4. **File-backed mmap** (Optional)

Only anonymous mappings supported. File mappings would need:
- VFS integration with mmap
- Not required for basic std

---

## How to Get std Working

### Option A: Build std from source (Hard)

1. Create custom target JSON:
   ```json
   {
     "llvm-target": "aarch64-unknown-none",
     "data-layout": "...",
     "os": "levitate",
     "executables": true,
     ...
   }
   ```

2. Build std with `-Z build-std`:
   ```bash
   cargo +nightly build -Z build-std=std,core,alloc --target levitate.json
   ```

3. Implement `std::sys::levitate` backend (significant work)

### Option B: Use Eyra/Origin approach (Recommended)

The **sunfishcode ecosystem** (`eyra` + `origin` + `rustix`) provides std-without-libc on Linux:

1. **Eyra** - Drop-in std replacement that uses Linux syscalls directly
2. **Origin** - Bare-metal entry point, TLS, thread creation  
3. **Rustix** - Pure Rust syscall wrappers

Since LevitateOS implements Linux ABI syscalls, these crates could work with minimal adaptation.

### Option C: Build a "compat" crate (Pragmatic)

Create `levitate-std` that wraps your existing `libsyscall`:
- Expose `std`-like API
- Use your existing syscalls underneath
- No need to upstream to Rust

---

## References

| Document | Location |
|----------|----------|
| Full Plan | `docs/planning/.archive/std-support/PLAN.md` |
| Requirements | `docs/planning/.archive/std-support/requirements.md` |
| Phase 7 (Validation) | `docs/planning/.archive/std-support/phase-7.md` |
| TEAM_239 Status | `.teams/200-299/TEAM_239_std_compatibility.md` |

---

## Recommendation

**The kernel work is ~90% complete.** The remaining work is:

1. **Test with an actual std binary** - Try Option B (Eyra)
2. **Verify struct layouts** - Add compile-time size assertions
3. **Document known limitations** - File-backed mmap, fork, etc.

The archived plan in `docs/planning/.archive/std-support/phase-7.md` has detailed UoWs for this final validation work.
