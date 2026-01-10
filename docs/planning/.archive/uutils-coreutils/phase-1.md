# Phase 1: Discovery — uutils-coreutils Integration

**TEAM_364** | uutils-coreutils for LevitateOS  
**Created:** 2026-01-09

---

## 1. Feature Summary

### Problem Statement
LevitateOS is currently hand-writing userspace utilities (cat, ls, pwd, etc.) migrated to Eyra/std. This approach:
- Requires significant development effort per utility
- Results in feature-incomplete utilities vs GNU
- Creates maintenance burden
- ls is already blocked on missing `getdents64` syscall

### Proposed Solution
Use [uutils-coreutils](https://github.com/uutils/coreutils) — a cross-platform Rust rewrite of GNU coreutils with:
- 655+ contributors
- GNU test suite compatibility tracking
- Individual crates per utility (`uu_cat`, `uu_ls`, etc.)
- Active maintenance

### Who Benefits
- Users get full GNU-compatible utilities
- Developers avoid reinventing coreutils
- LevitateOS gains credibility as a real OS

---

## 2. Success Criteria

1. At least one uutils utility builds and runs on LevitateOS
2. Binary size is acceptable (< 2MB for multicall or < 500KB per utility)
3. Missing syscalls are identified and documented
4. Clear path to integrate remaining utilities

---

## 3. Current State Analysis

### Existing Eyra Utilities

| Utility | Status | Notes |
|---------|--------|-------|
| cat | ✅ Works | 326KB, hand-written |
| pwd | ✅ Works | Hand-written |
| mkdir | ✅ Works | Hand-written |
| ls | ❌ Blocked | Needs `getdents64` (syscall 217) |

### Eyra Build Configuration
- Toolchain: `nightly-2025-04-28`
- Target: `x86_64-unknown-linux-gnu`
- Build: `-Zbuild-std=std,panic_abort`
- Eyra 0.22 with `experimental-relocate`

---

## 4. uutils-coreutils Analysis

### Crate Structure
Each utility is a separate crate:
- `uu_cat` - cat implementation
- `uu_ls` - ls implementation  
- `uu_cp` - cp implementation
- etc.

All depend on `uucore` (shared library).

### Integration Options

#### Option A: Individual Binaries
```toml
[dependencies]
uu_cat = "0.2"
```
- Pro: Simple, minimal
- Con: Need wrapper main() for each

#### Option B: Multicall Binary (BusyBox-style)
```toml
[dependencies]
coreutils = "0.2"
```
- Pro: Single binary, symlink-based dispatch
- Con: Larger binary, all-or-nothing

#### Option C: Custom Multicall
Build our own multicall using selected `uu_*` crates
- Pro: Control over included utilities
- Con: More integration work

---

## 5. Potential Blockers

### Syscall Requirements
uutils likely requires syscalls we haven't implemented:
- `getdents64` (217) — directory listing
- `fstatat` (262) — file metadata
- `openat` (257) — open relative to directory
- `mkdirat` (258) — mkdir relative to directory
- Signal handling syscalls
- Terminal ioctls

### Build System Differences
- uutils uses stable Rust; Eyra requires nightly
- uutils may have dependencies incompatible with `-Zbuild-std`
- May need patches for Eyra compatibility

---

## 6. Discovery Tasks

### Step 1: Attempt to Build uu_cat with Eyra
Create a test project using `uu_cat` crate with Eyra.

### Step 2: Identify Missing Syscalls
Capture syscall failures and document what's missing.

### Step 3: Evaluate Binary Size
Compare hand-written cat (326KB) vs uu_cat.

### Step 4: Document Findings
Create phase-2 design document with recommendations.

---

## 7. Questions for User

See `docs/questions/TEAM_364_uutils_questions.md`
