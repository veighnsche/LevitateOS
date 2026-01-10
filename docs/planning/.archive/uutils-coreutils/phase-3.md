# Phase 3: Implementation — uutils-coreutils Integration

**TEAM_364** | uutils-coreutils for LevitateOS  
**Created:** 2026-01-09

---

## User Decisions

| Question | Answer |
|----------|--------|
| Q1: Binary size | A — Accept larger for GNU compatibility |
| Q2: Architecture | B — Individual binaries |
| Q3: Utilities | Minimum: cat, cp, mv, rm, mkdir, rmdir, ls, ln, touch, pwd, echo, env, true, false |
| Q5: Migration | A — Stop hand-written migration, pivot to uutils immediately |

---

## Implementation Order

### Step 1: uu_cat PoC
Replace hand-written cat with uutils version.

### Step 2: Simple Utilities (No I/O)
- true, false, echo, pwd, env

### Step 3: File Operations  
- mkdir, rmdir, touch, rm, ln

### Step 4: Complex Utilities (High syscall risk)
- ls, cp, mv

---

## Step 1: uu_cat PoC

### 1.1 Replace cat/Cargo.toml

```toml
# TEAM_364: uutils-based cat for LevitateOS
[package]
name = "cat"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }
uu_cat = "0.0.28"

[profile.release]
panic = "abort"
opt-level = "s"
lto = true
strip = true

[unstable]
build-std-features = ["panic_immediate_abort"]
```

### 1.2 Replace cat/src/main.rs

```rust
// TEAM_364: uutils cat wrapper for LevitateOS
extern crate eyra;

fn main() -> std::process::ExitCode {
    uu_cat::uumain(std::env::args_os())
}
```

### 1.3 Build and Test

```bash
cd crates/userspace/eyra/cat
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort
```

### 1.4 Success Criteria
- Builds successfully
- Runs `cat --help` and shows uutils help
- Runs `cat /hello.txt` and outputs file contents

---

## Utilities Target List

| Utility | Crate | Priority | Status |
|---------|-------|----------|--------|
| cat | uu_cat | 1 | 🔄 In Progress |
| true | uu_true | 2 | Pending |
| false | uu_false | 2 | Pending |
| echo | uu_echo | 2 | Pending |
| pwd | uu_pwd | 2 | Pending |
| env | uu_env | 2 | Pending |
| mkdir | uu_mkdir | 3 | Pending |
| rmdir | uu_rmdir | 3 | Pending |
| touch | uu_touch | 3 | Pending |
| rm | uu_rm | 3 | Pending |
| ln | uu_ln | 3 | Pending |
| ls | uu_ls | 4 | Pending |
| cp | uu_cp | 4 | Pending |
| mv | uu_mv | 4 | Pending |

---

## Directory Structure After Implementation

```
crates/userspace/eyra/
├── rust-toolchain.toml    # Shared
├── cat/                   # uu_cat wrapper
├── true/                  # uu_true wrapper
├── false/                 # uu_false wrapper
├── echo/                  # uu_echo wrapper
├── pwd/                   # uu_pwd wrapper
├── env/                   # uu_env wrapper
├── mkdir/                 # uu_mkdir wrapper
├── rmdir/                 # uu_rmdir wrapper
├── touch/                 # uu_touch wrapper
├── rm/                    # uu_rm wrapper
├── ln/                    # uu_ln wrapper
├── ls/                    # uu_ls wrapper
├── cp/                    # uu_cp wrapper
└── mv/                    # uu_mv wrapper
```

---

## Cleanup After Migration

Remove hand-written utilities that are replaced:
- Delete old `crates/userspace/eyra/cat/` (after backup/replacement)
- Delete old `crates/userspace/eyra/pwd/`
- Delete old `crates/userspace/eyra/mkdir/`
- Delete old `crates/userspace/eyra/ls/` (was blocked anyway)
