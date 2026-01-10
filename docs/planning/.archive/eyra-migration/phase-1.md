# Phase 1: Discovery and Safeguards

**TEAM_362** | Refactor Userspace to Eyra/std  
**Created:** 2026-01-09

---

## 1. Refactor Summary

**What:** Replace all no_std userspace apps with Eyra/std equivalents  
**Why:** ulib was a temporary handrolled std; Eyra provides real std support  
**Outcome:** Clean, idiomatic Rust userspace with standard library

---

## 2. Success Criteria

| Criterion | Measurement |
|-----------|-------------|
| All apps build | `cargo build --release` succeeds |
| All apps work | Manual testing of each utility |
| Eyra test passes | `cargo xtask test eyra` |
| Behavior tests pass | `cargo xtask test` |
| ulib removed | Directory deleted, no references |

---

## 3. Behavioral Contracts

### 3.1 Apps That Must Work Identically

| App | Critical Behaviors |
|-----|-------------------|
| **init** | Spawns shell, never exits |
| **shell** | Prompt, command execution, builtins |
| **cat** | Read file, print to stdout |
| **cp** | Copy file A to B |
| **ls** | List directory contents |
| **mkdir** | Create directory |
| **rm** | Remove file |
| **rmdir** | Remove directory |
| **touch** | Create/update file timestamp |
| **pwd** | Print working directory |
| **ln** | Create link |
| **mv** | Move/rename file |

### 3.2 Syscalls Required

All syscalls needed for Eyra are already implemented (TEAM_360):
- read, write, open, close
- getdents, getcwd, chdir
- mkdir, unlink, rename, link, symlink
- stat, fstat, statx
- ppoll, tkill, pkey_alloc, sigaltstack
- clock_gettime, nanosleep
- spawn (LevitateOS custom)

---

## 4. Golden/Regression Tests

### 4.1 Existing Baselines

| Test | File | Must Pass |
|------|------|-----------|
| Behavior test | `tests/golden_boot_x86_64.txt` | ✅ |
| Eyra test | `cargo xtask test eyra` | ✅ |
| Regression tests | 39 checks | ✅ |

### 4.2 New Baselines Needed

None — existing tests cover boot and Eyra functionality.

---

## 5. Current Architecture

```
crates/userspace/
├── init/           # PID 1, spawns shell (38 lines)
├── shell/          # Interactive shell (323 lines)
├── levbox/         # 10 utilities (busybox-style)
│   └── src/bin/core/
│       ├── cat.rs, cp.rs, ln.rs, ls.rs
│       ├── mkdir.rs, mv.rs, pwd.rs
│       ├── rm.rs, rmdir.rs, touch.rs
├── libsyscall/     # Raw syscall wrappers (KEEP)
├── systest/        # Test binaries
└── repro_crash/    # Debug binaries
```

### 5.1 Dependencies Before (BROKEN)

```
init, shell, levbox → ulib → libsyscall → kernel
                      ^^^^
                      REMOVED
```

### 5.2 Dependencies After

```
init, shell, levbox → Eyra (std) → kernel
```

---

## 6. Constraints

| Constraint | Detail |
|------------|--------|
| Binary size | Accept ~300KB per app (vs ~25KB before) |
| Build time | Eyra first build is slow (~2 min), cached after |
| Toolchain | Requires nightly-2025-04-28 + rust-src |
| Target | x86_64-unknown-linux-gnu (not -none) |

---

## 7. Open Questions

### Q1: Levbox Approach

**Question:** Keep levbox as multi-binary crate, or split into individual apps?

**Options:**
- A) Keep levbox structure, each binary uses Eyra
- B) Split into separate crates (cat, ls, etc.)

**Recommendation:** Option A — less restructuring

### Q2: libsyscall Future

**Question:** What happens to libsyscall after migration?

**Options:**
- A) Keep for LevitateOS-specific syscalls (spawn, shutdown)
- B) Remove entirely, use raw syscalls in apps that need them

**Recommendation:** Option A — cleaner interface for custom syscalls

---

## 8. Phase 1 Steps

### Step 1: Verify Current State is Broken
- Confirm build fails without ulib
- Document exact errors

### Step 2: Verify Eyra Test Still Passes
- Run `cargo xtask test eyra`
- Confirm syscall support is sufficient

### Step 3: Create Eyra App Template
- Document Cargo.toml pattern
- Document build command
- Create example app structure
