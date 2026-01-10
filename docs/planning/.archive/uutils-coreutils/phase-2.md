# Phase 2: Design — uutils-coreutils Integration

**TEAM_364** | uutils-coreutils for LevitateOS  
**Created:** 2026-01-09

---

## 1. Proposed Architecture

### 1.1 Recommended Approach: Hybrid Individual Binaries

Use `uu_*` crates as library dependencies, with thin Eyra wrapper binaries.

```
crates/userspace/eyra/
├── cat/          # Uses uu_cat
├── ls/           # Uses uu_ls  
├── cp/           # Uses uu_cp
├── ...
```

Each app has:
```rust
extern crate eyra;
use uu_cat::uumain;

fn main() {
    std::process::exit(uumain(std::env::args()));
}
```

### 1.2 Why Not Multicall?

- Multicall requires more complex argv[0] dispatch
- Harder to debug individual utilities
- All-or-nothing inclusion
- Individual binaries allow mixing uutils + hand-written

---

## 2. Integration Pattern

### 2.1 Cargo.toml Template

```toml
[package]
name = "cat"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }
uu_cat = "0.2"

[profile.release]
panic = "abort"
opt-level = "s"
lto = true
strip = true

[unstable]
build-std-features = ["panic_immediate_abort"]
```

### 2.2 Main.rs Template

```rust
// TEAM_364: uutils wrapper for LevitateOS
extern crate eyra;

fn main() {
    std::process::exit(uu_cat::uumain(std::env::args_os()));
}
```

---

## 3. Utility Prioritization

### Tier 1: Essential (Phase 3)
| Utility | Crate | Syscall Risk |
|---------|-------|--------------|
| cat | uu_cat | Low |
| echo | uu_echo | Low |
| pwd | uu_pwd | Low |
| true | uu_true | None |
| false | uu_false | None |
| mkdir | uu_mkdir | Low |
| rmdir | uu_rmdir | Low |
| touch | uu_touch | Medium |

### Tier 2: Shell-Critical (Phase 4)
| Utility | Crate | Syscall Risk |
|---------|-------|--------------|
| ls | uu_ls | High (getdents64) |
| cp | uu_cp | Medium |
| mv | uu_mv | Medium |
| rm | uu_rm | Low |
| ln | uu_ln | Low |

### Tier 3: Extended (Future)
- head, tail, wc, sort, uniq, env, basename, dirname, etc.

---

## 4. Syscall Gap Analysis

### Known Missing (from TEAM_363)
- `getdents64` (217) — blocks ls

### Likely Required by uutils
- `fstatat` (262) — stat relative to dirfd
- `openat` (257) — open relative to dirfd
- `faccessat` (269) — access check relative to dirfd
- `readlinkat` (267) — readlink relative to dirfd
- `renameat` (264) — rename relative to dirfd
- `unlinkat` (263) — unlink relative to dirfd
- `linkat` (265) — link relative to dirfd
- `symlinkat` (266) — symlink relative to dirfd

### Strategy
1. Start with Tier 1 utilities (low syscall risk)
2. Document failures from Tier 2
3. Implement missing syscalls incrementally

---

## 5. Phase 3: Implementation Plan

### Step 1: Create uutils-cat PoC
- Replace hand-written cat with uu_cat wrapper
- Build and test
- Document binary size difference

### Step 2: Roll out Tier 1
- Migrate echo, pwd, true, false, mkdir, rmdir
- All low-risk, should work with current syscalls

### Step 3: Attempt Tier 2
- Try ls with uu_ls
- Capture syscall failures
- Create syscall implementation list

---

## 6. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Large binary size | LTO + strip + opt-level=s; accept tradeoff for features |
| Incompatible dependencies | Fork and patch if needed |
| Too many missing syscalls | Start with simple utilities, grow incrementally |
| Build failures with -Zbuild-std | Test early, report upstream if needed |

---

## 7. Success Metrics

1. **uu_cat builds and runs** with Eyra
2. **5+ utilities** working from Tier 1
3. **Binary size documented** for all utilities
4. **Missing syscalls identified** for Tier 2
5. **Clear roadmap** for full coreutils support

---

## 8. Decision Required

Before proceeding, need user input on questions in:
`docs/questions/TEAM_364_uutils_questions.md`

Key decisions:
- Q1: Binary size tradeoff acceptable?
- Q2: Individual binaries vs multicall?
- Q3: Which utility tier to target?
- Q5: Stop current migration or hybrid approach?
