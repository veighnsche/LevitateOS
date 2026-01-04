# TEAM_082: Investigate Linker Memory Allocation Failure

## Team Purpose
Investigate why rust-lld fails with "Cannot allocate memory" when building userspace binaries.

## Status: ✅ FIXED

---

## Bug Report

**Symptom:** When building `userspace/hello`, the linker fails:
```
rust-lld: error: failed to open .../hello-...: Cannot allocate memory
```

**Expected:** Binary should link successfully.
**Actual:** Linker exits with memory allocation error.

---

## Phase 1: Understand the Symptom ✅

### Initial Evidence
- System has **41GB available RAM** — NOT a memory issue
- File descriptors: 524288 (plenty)
- Disk space: 779GB free on /home
- File creation in target dir works fine

### Key Observation
The error is "failed to **open**" not "failed to allocate" — the linker can't CREATE the output file.

---

## Phase 2: Form Hypotheses ✅

1. **H1: System resource exhaustion** — RULED OUT (plenty of RAM, disk, FDs)
2. **H2: Corrupted target directory** — RULED OUT (rm -rf and rebuild same result)
3. **H3: Conflicting linker scripts** — CONFIRMED ✅

---

## Phase 3: Test Hypotheses ✅

Examined linker command line:
```
rust-lld ... -Tlinker.ld -Tlink.ld
```

TWO linker scripts are being passed!

---

## Phase 4: Root Cause ✅

### The Problem
- Root `.cargo/config.toml` adds `-Tlinker.ld` for ALL `aarch64-unknown-none` targets
- Userspace `.cargo/config.toml` adds `-Tlink.ld`
- Cargo **merges** config files, so BOTH are passed to the linker

### Why Memory Allocation Fails
- `linker.ld` (kernel): Virtual base at `0xFFFF800000000000` (higher-half)
- `link.ld` (userspace): Start at `0x10000` (user address)

The linker tries to create an output file spanning this **massive address range** (multiple terabytes of virtual address space), which fails with "Cannot allocate memory".

---

## Phase 5: Fix ✅

### Solution Applied
Created an **empty** `linker.ld` in userspace to satisfy the root config's `-Tlinker.ld` requirement without adding conflicting sections:

```ld
/* TEAM_082: Empty linker script to satisfy root config's -Tlinker.ld */
/* The actual linker script is link.ld */
```

### Result
- ✅ `userspace/hello` builds successfully
- ✅ Shell binary created (77KB)
- ✅ Kernel with TEAM_081 changes rebuilds
- ✅ Dual console message appears in boot log

---

## Files Changed

| File | Change |
|------|--------|
| `userspace/hello/linker.ld` | Empty stub to prevent conflict |
| `userspace/hello/.cargo/config.toml` | Added comment explaining the issue |

---

## Lessons Learned

1. Cargo config files are **merged hierarchically**, not replaced
2. "Cannot allocate memory" during linking can mean address space exhaustion, not RAM
3. Multiple linker scripts with conflicting memory layouts cause massive virtual address ranges
