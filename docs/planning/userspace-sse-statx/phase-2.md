# Phase 2: Design — Userspace SSE & statx

**TEAM_358** | Userspace SSE/FPU Enablement & statx Syscall  
**Created:** 2026-01-09

---

## 1. Feature 1: SSE/FPU Enablement

### 1.1 Proposed Solution

**Approach: Eager FPU — Always Save/Restore**

Enable SSE at boot and always save/restore FPU state on context switch. This is simpler than lazy FPU switching and sufficient for LevitateOS's current scale.

**Boot Changes (boot.S):**
```asm
    /* Enable SSE - must be done before any SSE instructions */
    mov eax, cr0
    and eax, ~(1 << 2)     /* Clear CR0.EM (bit 2) - no FPU emulation */
    or eax, (1 << 1)       /* Set CR0.MP (bit 1) - monitor coprocessor */
    and eax, ~(1 << 3)     /* Clear CR0.TS (bit 3) - no task switch trap */
    mov cr0, eax

    mov eax, cr4
    or eax, (1 << 9)       /* Set CR4.OSFXSR (bit 9) - enable FXSAVE/FXRSTOR */
    or eax, (1 << 10)      /* Set CR4.OSXMMEXCPT (bit 10) - enable #XM */
    mov cr4, eax
```

**Context Switch:**
- Add 512-byte aligned `fxsave_area` to `Context` struct
- On switch-out: `fxsave [context.fxsave_area]`
- On switch-in: `fxrstor [context.fxsave_area]`

### 1.2 Implementation Steps

#### Step 1: Enable SSE in boot.S (~30 min)
- Location: `crates/kernel/src/arch/x86_64/boot.S`, after paging enabled
- Add CR0/CR4 manipulation code
- Place before any Rust code runs (Rust may emit SSE)

#### Step 2: Add FPU state to Context (~1 hr)
- Location: `crates/kernel/src/arch/x86_64/task.rs`
- Add `fxsave_area: [u8; 512]` aligned to 16 bytes
- Initialize to zeros (FINIT state)

#### Step 3: Save/Restore in context switch (~1 hr)
- Modify `switch_context` assembly
- Use `fxsave64` and `fxrstor64` (64-bit versions)

#### Step 4: Test with SSE binary (~30 min)
- Create test binary that uses XMM registers
- Verify no #UD exception

### 1.3 FPU State Layout (FXSAVE)

```
Offset  Size  Description
0x00    2     FCW - FPU Control Word
0x02    2     FSW - FPU Status Word
0x04    1     FTW - FPU Tag Word (abridged)
0x05    1     Reserved
0x06    2     FOP - FPU Opcode
0x08    8     FIP - FPU Instruction Pointer
0x10    8     FDP - FPU Data Pointer
0x18    4     MXCSR - SSE Control/Status
0x1C    4     MXCSR_MASK
0x20    160   ST0-ST7 / MM0-MM7 (8 x 16 bytes, 10-byte precision)
0xA0    256   XMM0-XMM15 (16 x 16 bytes)
0x1A0   96    Reserved (padding to 512 bytes)
```

### 1.4 Design Alternatives Considered

| Alternative | Pros | Cons | Decision |
|-------------|------|------|----------|
| **Eager FPU** (always save/restore) | Simple, predictable | 512 bytes per task, overhead on every switch | ✅ Chosen |
| **Lazy FPU** (trap on first use) | Saves memory if task doesn't use FPU | Complex, #NM handler needed | ❌ Too complex for now |
| **No FPU** (userspace only) | Minimal changes | Rust kernel code may use SSE | ❌ Unsafe |

---

## 2. Feature 2: statx Syscall

### 2.1 Proposed Solution

Implement `statx` as an extended version of `fstat`, returning a larger `struct statx` with additional fields. For MVP, populate core fields from existing stat logic; leave extended fields as zeros.

**Syscall signature:**
```rust
pub fn sys_statx(
    dirfd: i32,
    pathname: usize,
    flags: i32,
    mask: u32,
    statxbuf: usize,
) -> i64
```

**Syscall numbers:**
- x86_64: 332
- aarch64: 291

> **Note:** User reported 302 — this might be from strace output. Actual Linux x86_64 number is 332.

### 2.2 struct statx Layout

```rust
#[repr(C)]
pub struct Statx {
    pub stx_mask: u32,        // What fields are filled in
    pub stx_blksize: u32,     // Preferred block size for I/O
    pub stx_attributes: u64,  // File attributes
    pub stx_nlink: u32,       // Number of hard links
    pub stx_uid: u32,         // User ID
    pub stx_gid: u32,         // Group ID
    pub stx_mode: u16,        // File type and mode
    pub __spare0: [u16; 1],
    pub stx_ino: u64,         // Inode number
    pub stx_size: u64,        // File size
    pub stx_blocks: u64,      // Number of 512B blocks
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,  // Birth (creation) time
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
    pub stx_dio_mem_align: u32,
    pub stx_dio_offset_align: u32,
    pub __spare3: [u64; 12],  // Padding to 256 bytes
}

#[repr(C)]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    pub __reserved: i32,
}
```

### 2.3 Implementation Steps

#### Step 1: Add syscall numbers (~10 min)
- `crates/kernel/src/arch/x86_64/mod.rs`: Add `Statx = 332`
- `crates/kernel/src/arch/aarch64/mod.rs`: Add `Statx = 291`

#### Step 2: Define Statx structs (~20 min)
- Add to `crates/kernel/src/syscall/fs/stat.rs`
- Both architectures use the same layout

#### Step 3: Implement sys_statx (~1 hr)
- Support both pathname-based (with dirfd) and fd-based (AT_EMPTY_PATH)
- Use existing VFS stat logic
- Convert `Stat` → `Statx` fields

#### Step 4: Wire into dispatcher (~10 min)
- Add case to `syscall_dispatch` in `mod.rs`

### 2.4 statx Flags and Mask

**Flags (subset to support):**
```rust
pub const AT_EMPTY_PATH: i32 = 0x1000;     // pathname is empty, use dirfd as fd
pub const AT_SYMLINK_NOFOLLOW: i32 = 0x100;
pub const AT_STATX_SYNC_AS_STAT: i32 = 0x0000;
```

**Mask (what to return):**
```rust
pub const STATX_TYPE: u32 = 0x0001;
pub const STATX_MODE: u32 = 0x0002;
pub const STATX_NLINK: u32 = 0x0004;
pub const STATX_UID: u32 = 0x0008;
pub const STATX_GID: u32 = 0x0010;
pub const STATX_ATIME: u32 = 0x0020;
pub const STATX_MTIME: u32 = 0x0040;
pub const STATX_CTIME: u32 = 0x0080;
pub const STATX_INO: u32 = 0x0100;
pub const STATX_SIZE: u32 = 0x0200;
pub const STATX_BLOCKS: u32 = 0x0400;
pub const STATX_BASIC_STATS: u32 = 0x07FF;
pub const STATX_BTIME: u32 = 0x0800;  // Birth time - optional
```

---

## 3. Implementation Priority

| Feature | Priority | Effort | Reason |
|---------|----------|--------|--------|
| SSE/FPU | **P0** | 3 hrs | Causes crash - blocking |
| statx | P1 | 2 hrs | Needed for file ops |

**Recommended order:**
1. SSE enablement (boot.S changes)
2. FPU context save/restore
3. statx syscall

---

## 4. Open Questions

### Q1: XSAVE vs FXSAVE
**Question:** Should we use XSAVE (variable-size, supports AVX) or FXSAVE (fixed 512 bytes)?

**Options:**
- A) FXSAVE only — simpler, 512 bytes, SSE/SSE2 only
- B) XSAVE with CPUID detection — future-proof, but more complex
- C) FXSAVE now, XSAVE later — pragmatic

**Recommendation:** Option A (FXSAVE only) for MVP

---

### Q2: Kernel FPU Usage
**Question:** Does the Rust compiler emit SSE instructions in kernel code?

**Impact:** If yes, kernel must save/restore FPU when calling user space.

**Recommendation:** 
- x86_64-unknown-none target disables SSE by default (`-C target-feature=-sse`)
- If our kernel build already does this, no issue
- Need to verify our `rust-toolchain.toml` / `.cargo/config.toml`

---

### Q3: statx Syscall Number Discrepancy
**Question:** User reported syscall 302, but Linux x86_64 statx is 332. Which is correct?

**Options:**
- A) Trust Linux source: implement 332
- B) Check strace output from user's binary to confirm

**Recommendation:** Implement 332 (Linux canonical), but investigate 302

---

### Q4: Extended statx Fields
**Question:** Which optional statx fields should we populate?

**Fields in question:**
- `stx_btime` (birth time) — VFS may not track this
- `stx_mnt_id` (mount ID) — Not tracked currently
- `stx_attributes` (immutable, append-only, etc.) — Not supported

**Recommendation:** Return 0 for extended fields; set `stx_mask` to indicate which fields are valid.

---

## 5. Phase 3 Preview

Once design is approved:

### Phase 3-Step-1: SSE Boot Enablement
- Edit `boot.S` to set CR0/CR4 bits
- Verify kernel still boots
- Test with simple SSE binary

### Phase 3-Step-2: FPU Context Switch
- Add `fxsave_area` to Context
- Modify switch assembly
- Test multi-tasking with SSE

### Phase 3-Step-3: statx Implementation
- Add syscall numbers
- Define structs
- Implement `sys_statx`
- Wire to dispatcher

---

## 6. Testing Strategy

### SSE Tests
```rust
// Test binary: userspace/sse-test/src/main.rs
fn main() {
    // This should not crash
    let a: f64 = 3.14;
    let b: f64 = 2.72;
    let c = a * b;
    println!("SSE works: {} * {} = {}", a, b, c);
}
```

### statx Tests
```rust
// Test: statx on existing file
use std::fs;
let meta = fs::metadata("/etc/passwd");  // Uses statx internally
assert!(meta.is_ok());
```

---

## 7. Verification Checklist

Before Phase 3 completion:

- [ ] `cargo build` succeeds for both architectures
- [ ] `cargo xtask test` passes all existing tests
- [ ] PIE binary with float operations runs without #UD
- [ ] statx returns valid data for existing files
- [ ] No regressions in golden logs
