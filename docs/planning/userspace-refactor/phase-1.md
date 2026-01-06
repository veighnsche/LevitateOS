# Phase 1 — Discovery and Safeguards

**Parent:** Userspace Architecture Refactor  
**File:** `docs/planning/userspace-refactor/phase-1.md`

---

## 1. Refactor Summary

**What:** Refactor `userspace/` from independent, duplicated apps into a proper multi-crate workspace with shared libraries.

**Pain Points:**
1. Syscall wrappers are duplicated in every userspace app (~60 lines each)
2. `userspace/hello` and `userspace/shell` are nearly identical (both are shells)
3. No shared `libsyscall` crate
4. No proper `init` process (PID 1)
5. Build is fragmented — must build each app manually, copy to initrd_root, run make_initramfs.sh
6. Kernel hardcodes which binary to run ("hello")

**Motivation:** Rule 1 (SSOT) — syscall ABI should be defined once. Rule 7 (Modular) — each crate should have one responsibility.

---

## 2. Success Criteria

### Before (Current State)
```
userspace/
├── hello/           # Independent workspace, duplicated shell code
│   ├── Cargo.toml   # [workspace]
│   └── src/main.rs  # Full syscall + shell implementation
└── shell/           # Independent workspace, primary shell
    ├── Cargo.toml   # [workspace]
    └── src/main.rs  # Full syscall + shell implementation
```

### After (Target State)
```
userspace/
├── Cargo.toml           # Workspace definition
├── libsyscall/          # Shared syscall library
│   ├── Cargo.toml
│   └── src/lib.rs       # read(), write(), exit(), getpid(), sbrk()
├── init/                # PID 1 process
│   ├── Cargo.toml       # depends on libsyscall
│   └── src/main.rs      # Spawns shell, respawns on exit
├── shell/               # Interactive shell
│   ├── Cargo.toml       # depends on libsyscall
│   └── src/main.rs      # Shell logic only
└── levbox/           # Optional: separate levbox
    └── echo/            # Each util as separate binary
```

**Key Metrics:**
- [ ] `libsyscall` is the single source of truth for syscall ABI
- [ ] Zero duplicated syscall code across userspace apps
- [ ] `cargo xtask build-userspace` builds all apps and updates initramfs
- [ ] Kernel runs `init` which spawns `shell` (not hardcoded "hello")

---

## 3. Behavioral Contracts

### 3.1 Syscall ABI (Must Not Change)
| Syscall | Number | Signature |
|---------|--------|-----------|
| read | 0 | `read(fd: usize, buf: *mut u8, len: usize) -> isize` |
| write | 1 | `write(fd: usize, buf: *const u8, len: usize) -> isize` |
| exit | 2 | `exit(code: i32) -> !` |
| getpid | 3 | `getpid() -> i64` |
| sbrk | 4 | `sbrk(increment: isize) -> i64` |

### 3.2 Shell Behavior (Must Remain Identical)
- Boot → Show banner → Show `# ` prompt
- Builtins: `echo`, `help`, `clear`, `exit`
- Unknown command → "Unknown command: ..."

### 3.3 Build Integration
- `cargo xtask test all` must still pass
- Behavior golden file must match
- VNC visual verification must show shell

---

## 4. Golden/Regression Tests

### 4.1 Existing Tests That Must Pass
- `cargo xtask test behavior` — Golden boot log comparison
- `cargo xtask test regression` — 27 regression checks
- `cargo xtask run-vnc` — Visual shell verification

### 4.2 New Baseline to Capture
After refactor, boot output should show:
```
[INIT] Starting init process...
[INIT] Spawning shell...

LevitateOS Shell (lsh) v0.1
Type 'help' for available commands.

# 
```

---

## 5. Current Architecture Notes

### 5.1 Dependency Graph
```
kernel
  └── embedded initramfs.cpio
        ├── hello (ELF binary - currently shell)
        └── hello.txt, test.txt (test files)
```

### 5.2 Build Flow (Current)
1. `cd userspace/hello && cargo build --release`
2. `cp target/aarch64-unknown-none/release/hello ../../initrd_root/`
3. `./scripts/make_initramfs.sh`
4. `cargo build --release` (kernel embeds initramfs.cpio)

### 5.3 Known Couplings
- Kernel hardcodes `"hello"` as first process name
- Linker scripts (`link.ld`) define userspace memory layout
- Root `.cargo/config.toml` adds kernel linker script, which conflicts with userspace

### 5.4 Linker Script Workaround (GOTCHAS.md #2)
Each userspace app needs an empty `linker.ld` stub to satisfy root config without adding conflicting sections.

---

## 6. Constraints

| Constraint | Requirement |
|------------|-------------|
| Syscall ABI | Must remain identical (binary compatibility) |
| Shell behavior | Must be identical (golden test) |
| Memory layout | User process at 0x1000, stack at 0x7FFF_FFFF_F000 |
| No heap in userspace | Current apps are heap-free (no allocator) |
| Target triple | `aarch64-unknown-none` |

---

## 7. Open Questions

> [!IMPORTANT]
> **Q1: Should we implement spawn syscall as part of this refactor?**
> 
> Without spawn, `init` can only print a banner and then become the shell itself (exec-style). With spawn, `init` can fork/spawn shell and respawn on exit.
>
> **Options:**
> - A) Skip spawn for now — init just execs into shell (simpler)
> - B) Implement spawn as part of this refactor (more complete)
> - C) Defer spawn to Phase 8c, keep init minimal for now
>
> **Suggested:** Option C — keep init minimal, defer spawn to Phase 8c

> [!NOTE]  
> **Q2: Should userspace use a shared workspace or stay independent?**
>
> Shared workspace would allow `libsyscall` as a dependency. But root `.cargo/config.toml` has kernel-specific settings.
>
> **Options:**
> - A) Create `userspace/Cargo.toml` as independent workspace with its own `.cargo/config.toml`
> - B) Keep apps independent but share `libsyscall` via path dependency
>
> **Suggested:** Option A — userspace is its own workspace

---

## 8. Phase 1 Steps

### Step 1: Map Current Behavior and Boundaries
- [x] Document current directory structure
- [x] Document syscall ABI
- [x] Document shell behavior
- [x] Identify duplication

### Step 2: Lock in Golden Tests
- [ ] Run `cargo xtask test all` — confirm passing
- [ ] Capture current boot log as baseline
- [ ] Add shell-specific behavior tests if missing

### Step 3: Prepare for Extraction
- [ ] Identify exact lines to extract to `libsyscall`
- [ ] Identify shell-only code
- [ ] Plan init process responsibilities
