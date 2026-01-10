# Phase 1: Understanding and Scoping

**TEAM_365** | Eyra/uutils Linker Conflict  
**Created:** 2026-01-10

---

## 1. Bug Summary

**Description:** When building certain uutils-coreutils utilities with Eyra (pure-Rust libc), the linker fails with duplicate symbol errors for `_start` and `__dso_handle`.

**Severity:** High — blocks 10 of 14 target utilities from building

**Impact:** 
- Cannot use battle-tested GNU-compatible utilities
- Must hand-write replacements for blocked utilities
- Reduces LevitateOS functionality

---

## 2. Reproduction Status

**Reproducible:** Yes, 100% consistent

### Reproduction Steps

```bash
cd crates/userspace/eyra/echo
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort
```

### Expected Behavior
Binary compiles successfully like cat, pwd, mkdir, ls

### Actual Behavior
```
rust-lld: error: duplicate symbol: _start
>>> defined at /usr/lib/gcc/x86_64-redhat-linux/15/../../../../lib64/Scrt1.o:(_start)
>>> defined at echo.xxx-cgu.0 (...rcgu.o:(.text._start+0x0))

rust-lld: error: duplicate symbol: __dso_handle
>>> defined at /usr/lib/gcc/x86_64-redhat-linux/15/crtbeginS.o:(.data.rel.ro.local+0x0)
>>> defined at echo.xxx-cgu.0 (...rcgu.o:(.data.rel.ro.__dso_handle+0x0))
```

---

## 3. Affected vs Working Utilities

| Status | Utilities |
|--------|-----------|
| ✅ Working | cat, pwd, mkdir, ls |
| ❌ Blocked | true, false, echo, env, rmdir, touch, rm, ln, cp, mv |

**Pattern observation:** 4 work, 10 don't — need to find what differentiates them.

---

## 4. Context

### Code Areas Suspected
1. **Eyra entry point handling** — Eyra provides `_start` via origin crate
2. **uutils crate structure** — Some crates may embed entry point code
3. **Cargo.toml configuration** — Differences between working/blocked utilities
4. **Build flags** — `-Zbuild-std` interaction with uutils

### Recent Changes
- TEAM_364 created uutils wrapper binaries
- All use identical Cargo.toml pattern and main.rs structure

### Relevant Files
- Working: `crates/userspace/eyra/cat/Cargo.toml`
- Blocked: `crates/userspace/eyra/echo/Cargo.toml`

---

## 5. Constraints

- **Eyra version:** 0.22 with `experimental-relocate` feature
- **Toolchain:** nightly-2025-04-28
- **Target:** x86_64-unknown-linux-gnu
- **Build:** `-Zbuild-std=std,panic_abort`

---

## 6. Open Questions

### Q1: What differentiates working vs blocked uutils crates?
- Same Cargo.toml pattern
- Same main.rs wrapper pattern
- Different uu_* dependencies

### Q2: Do blocked crates have different transitive dependencies?
- Need to compare dependency trees

### Q3: Is this an Eyra bug or uutils bug?
- Eyra normally handles `_start`
- Something in blocked crates is providing a conflicting `_start`

### Q4: Can we use linker flags to resolve the conflict?
- `--allow-multiple-definition`?
- Symbol versioning?

---

## 7. Phase 1 Steps

### Step 1: Compare Working vs Blocked Dependency Trees
Compare `cargo tree` output for cat (working) vs echo (blocked).

### Step 2: Identify the Source of Duplicate _start
Find which crate/object file is providing the conflicting `_start` symbol.

### Step 3: Check Eyra/Origin Configuration
Review how Eyra's origin crate provides entry point and why it conflicts.
