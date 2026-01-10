# TEAM_380: Review Eyra Crate Deduplication

## Status: Complete

## Problem Statement

The `/crates/userspace/eyra/` directory contains 15+ crates with massive duplication:

| Pattern | Lines/Crate | Total Duplicated |
|---------|-------------|------------------|
| `build.rs` | 22 identical lines | ~330 lines |
| `Cargo.toml` boilerplate | ~10 lines | ~150 lines |
| `main.rs` wrapper | 6 lines | ~90 lines |

Each crate is a thin wrapper around a `uu_*` crate from uutils-coreutils.

## Research Findings

### 1. Sunfishcode's uutils-coreutils Mustang Port (KEY FINDING)

**Sunfishcode has already ported uutils-coreutils to work with Eyra/Mustang:**
- Repository: https://github.com/sunfishcode/coreutils/tree/mustang
- The port is a **SINGLE COMMIT**: "Port to Mustang" (40f8958)
- This means the changes are minimal

**Implication:** We can use uutils-coreutils directly — no wrapper crates needed at all.

### 2. Multi-call Binary Pattern (uutils-coreutils)

uutils-coreutils itself uses a **multi-call binary** pattern:
- Single `coreutils` binary contains ALL utilities
- Dispatch based on `argv[0]` or first argument
- Creates symlinks: `ls -> coreutils`, `cat -> coreutils`, etc.
- **Dramatically reduces** binary count, build complexity, and duplication

Source: https://uutils.github.io/coreutils/docs/multicall.html

### 3. Large Rust Workspaces (matklad)

Key recommendations from rust-analyzer maintainer:
- Flat layout (`crates/*`) with glob members
- Virtual manifest at root
- Workspace dependency inheritance
- Single xtask crate for automation

Source: https://matklad.github.io/2021/08/22/large-rust-workspaces.html

---

## REVISED Proposed Solution: Use uutils-coreutils Directly

Instead of 15 wrapper crates OR a custom multi-call shim, **use uutils-coreutils directly**:

### Option A: Fork sunfishcode/coreutils (Recommended)

```
crates/userspace/eyra/
├── Cargo.toml              # workspace
├── coreutils/              # Git submodule or vendored fork of sunfishcode/coreutils
│   └── ...                 # Full uutils-coreutils with Eyra support
├── eyra-hello/             # Keep: test binary
└── eyra-test-runner/       # Keep: test infrastructure
```

**Pros:**
- Zero wrapper code
- Full uutils multi-call binary
- All 100+ coreutils commands available
- Upstream updates easy to merge

**Cons:**
- Large dependency
- May include commands you don't need

### Option B: Minimal Multi-call Binary (Fallback)

If Option A is too heavy, create a minimal multi-call crate that depends on individual `uu_*` crates:

```rust
// src/main.rs - dispatch to selected utilities only
extern crate eyra;

fn main() {
    let cmd = std::env::args().next().unwrap();
    let code = match cmd.rsplit('/').next().unwrap() {
        "ls"    => uu_ls::uumain(std::env::args_os()),
        "cat"   => uu_cat::uumain(std::env::args_os()),
        // ... only commands you need
        _ => 1,
    };
    std::process::exit(code);
}
```

---

## Comparison

| Approach | Wrapper Crates | Build.rs Files | Binaries | Maintainability |
|----------|---------------|----------------|----------|-----------------|
| **Current** | 15 | 15 | 15 | ❌ Poor |
| **Option A** (fork uutils) | 0 | 1 | 1 | ✅ Best |
| **Option B** (custom multicall) | 1 | 1 | 1 | ✅ Good |

---

## Migration Path (Option A)

1. Add sunfishcode/coreutils as git submodule or vendor it
2. Configure build with Eyra (already done in mustang branch)
3. Build single `coreutils` binary
4. Update ramdisk to create symlinks: `ls -> coreutils`, etc.
5. Delete the 15 individual wrapper crates
6. Update workspace members

## Questions for User

1. **Option A or B?** Fork full uutils or minimal multi-call?
2. **Git submodule or vendor?** Submodule tracks upstream; vendor is self-contained
3. **Symlinks created where?** Ramdisk build time (recommended)

## Implementation Complete

### What was done:

1. **Added uutils-coreutils as git submodule** (sunfishcode/coreutils mustang branch)
2. **Converted from Mustang to Eyra:**
   - Replaced `mustang` dependency with `eyra`
   - Changed `mustang::can_run_this!()` to `extern crate eyra;`
   - Added `-nostartfiles` and aarch64 libgcc_eh.a stub to build.rs
   - Added static-PIE rustflags to .cargo/config
3. **Built 4.8MB static-PIE multi-call binary** with 14 utilities
4. **Deleted 15 individual wrapper crates** (cat, cp, echo, env, false, ln, ls, mkdir, mv, pwd, rm, rmdir, touch, true)
5. **Updated eyra workspace Cargo.toml** to only include eyra-hello and eyra-test-runner
6. **Updated xtask build system:**
   - `build_eyra()` now builds coreutils multi-call binary
   - `create_initramfs()` copies coreutils + creates symlinks
   - `create_test_initramfs()` same pattern

### Results:

| Before | After |
|--------|-------|
| 15 crates | 1 submodule |
| 15 build.rs (330 lines) | 1 build.rs |
| 15 Cargo.toml | 1 Cargo.toml |
| 15 main.rs (90 lines) | Reused upstream |
| 15 binaries | 1 binary + 14 symlinks |
| ~70KB per util | 4.8MB total (shared) |

### Files changed:

- Added: `crates/userspace/eyra/coreutils/` (submodule)
- Modified: `crates/userspace/eyra/coreutils/Cargo.toml` (mustang → eyra)
- Modified: `crates/userspace/eyra/coreutils/src/bin/coreutils.rs` (mustang → eyra)
- Modified: `crates/userspace/eyra/coreutils/build.rs` (added Eyra linker flags)
- Modified: `crates/userspace/eyra/coreutils/.cargo/config` (added static-PIE)
- Modified: `crates/userspace/eyra/Cargo.toml` (removed 14 members)
- Deleted: 14 utility directories (cat/, cp/, etc.)
- Modified: `xtask/src/build/commands.rs` (use coreutils + symlinks)

### Handoff Checklist:
- [x] Project builds cleanly
- [x] Coreutils binary works
- [x] Initramfs creates correctly with symlinks
- [x] Team file updated
