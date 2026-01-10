# Phase 1: Discovery — Userspace SSE & statx

**TEAM_358** | Userspace SSE/FPU Enablement & statx Syscall  
**Created:** 2026-01-09

---

## 1. Feature Summary

### Problem Statement

PIE binaries compiled with standard Rust/C toolchains crash on LevitateOS x86_64 due to:

1. **SSE not enabled** — Instructions like `xorps xmm0, xmm0` trigger #UD (Invalid Opcode)
2. **statx not implemented** — Syscall 302 (x86_64) returns ENOSYS

### Who Benefits

- Users running any userspace binary compiled with `-C target-feature=+sse,+sse2` (default for x86_64)
- Rust std users (Eyra, standard libc) that use statx for file metadata

### Success Criteria

1. PIE binaries using XMM registers execute without #UD exceptions
2. `statx(2)` syscall returns valid file metadata
3. No regression in existing tests

---

## 2. Current State Analysis

### SSE/FPU State

**x86_64 boot sequence** (`crates/kernel/src/arch/x86_64/boot.S`):
- Currently enables PAE (CR4 bit 5) and paging (CR0 bits 0, 31)
- **Does NOT** set OSFXSR (CR4 bit 9) or enable SSE
- **Does NOT** clear CR0.EM (bit 2) which must be 0 for SSE

**Result:** Any SSE instruction in userspace causes #UD exception.

### statx Syscall

**Current implementation:** None. Syscall 302 (x86_64) / 332 (aarch64) not in `SyscallNumber` enum.

**Existing related syscalls:**
- `fstat` — Returns `struct stat` (legacy, smaller)
- `newfstatat` — Not implemented

**statx difference:** Returns `struct statx` with extended attributes (birth time, mount ID, etc.)

---

## 3. Codebase Reconnaissance

### Files for SSE/FPU

| File | Purpose | Changes Needed |
|------|---------|----------------|
| `crates/kernel/src/arch/x86_64/boot.S` | Early boot setup | Enable SSE in CR0/CR4 |
| `crates/kernel/src/arch/x86_64/task.rs` | Context switch | Save/restore FPU state (optional) |
| `crates/kernel/src/arch/x86_64/exceptions.rs` | Exception handlers | Handle #NM for lazy FPU |

### Files for statx

| File | Purpose | Changes Needed |
|------|---------|----------------|
| `crates/kernel/src/arch/x86_64/mod.rs` | Syscall numbers | Add `Statx = 332` |
| `crates/kernel/src/arch/aarch64/mod.rs` | Syscall numbers | Add `Statx = 291` |
| `crates/kernel/src/syscall/mod.rs` | Dispatcher | Add dispatch case |
| `crates/kernel/src/syscall/fs/stat.rs` | stat implementation | Add `sys_statx` |

---

## 4. Technical Background

### SSE Enablement Requirements (Intel SDM Vol 3, Ch 13)

To enable SSE instructions:

1. **CR0 settings:**
   - Clear `CR0.EM` (bit 2) = 0 — Must not emulate FPU
   - Set `CR0.MP` (bit 1) = 1 — Monitor coprocessor
   - Clear `CR0.TS` (bit 3) = 0 initially — No task-switched trap

2. **CR4 settings:**
   - Set `CR4.OSFXSR` (bit 9) = 1 — OS supports FXSAVE/FXRSTOR
   - Set `CR4.OSXMMEXCPT` (bit 10) = 1 — OS handles SIMD exceptions

3. **Optional (AVX/XSAVE):**
   - Set `CR4.OSXSAVE` (bit 18) if XSAVE used
   - Check CPUID for feature support

### Minimal SSE Code (boot.S)

```asm
    /* Enable SSE */
    mov eax, cr0
    and eax, 0xFFFFFFFB    /* Clear CR0.EM (bit 2) */
    or eax, 0x2            /* Set CR0.MP (bit 1) */
    mov cr0, eax

    mov eax, cr4
    or eax, 0x600          /* Set OSFXSR (bit 9) + OSXMMEXCPT (bit 10) */
    mov cr4, eax
```

### statx Syscall (Linux ABI)

```c
int statx(int dirfd, const char *pathname, int flags,
          unsigned int mask, struct statx *statxbuf);
```

**Syscall numbers:**
- x86_64: 332
- aarch64: 291

**Note:** I see from the user's message that the number is 302 for x86_64. Let me verify.

Actually, looking at Linux syscall tables:
- x86_64: `statx = 332` (not 302)
- User reported 302 — this might be from a different source

I'll confirm in the design phase.

---

## 5. Constraints

### SSE Constraints

1. **FPU state save/restore** — If tasks share XMM registers, context switches must save/restore 512 bytes (FXSAVE) or more (XSAVE)
2. **Lazy FPU switching** — Alternative: only save FPU state when a task actually uses it (uses #NM trap)
3. **Kernel FPU usage** — Rust compiler may emit SSE in kernel code; if so, kernel must save FPU before using

### statx Constraints

1. **Struct size** — `struct statx` is 256 bytes (larger than `stat`)
2. **Backward compatibility** — Can implement as wrapper around existing stat logic for MVP
3. **Extended fields** — Birth time, mount ID may not be available; return 0

---

## 6. Dependencies

- Neither feature depends on the other
- Both can be implemented independently
- SSE is higher priority (causes crashes)
- statx is needed for file operations

---

## 7. Next Steps (Phase 2)

1. Design SSE enablement approach (eager vs lazy FPU)
2. Define statx struct layout
3. List implementation steps
4. Generate questions for behavioral decisions
