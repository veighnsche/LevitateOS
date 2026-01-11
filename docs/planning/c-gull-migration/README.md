# C-Gull Migration Plan: Removing Eyra Dependency

**Created**: 2026-01-11
**Status**: Planning
**Goal**: Run unmodified uutils/coreutils without Eyra

---

## Current Architecture (Eyra)

```
┌─────────────────────────────────────────────────────────────┐
│                    sunfishcode/coreutils                     │
│            (forked uutils with Eyra integration)            │
├─────────────────────────────────────────────────────────────┤
│   Cargo.toml: std = { package = "eyra", version = "0.22" }  │
├─────────────────────────────────────────────────────────────┤
│                         Eyra                                 │
│    (Rust std replacement - makes syscalls directly)         │
├─────────────────────────────────────────────────────────────┤
│                    Linux Syscalls                            │
├─────────────────────────────────────────────────────────────┤
│                    LevitateOS Kernel                         │
└─────────────────────────────────────────────────────────────┘
```

**Problem**: Every app needs `std = { package = "eyra" }` in Cargo.toml

---

## Target Architecture (c-gull)

```
┌─────────────────────────────────────────────────────────────┐
│              uutils/coreutils (UNMODIFIED)                   │
│                   No source changes needed                   │
├─────────────────────────────────────────────────────────────┤
│                     Rust std (stock)                         │
│                  Links against "libc"                        │
├─────────────────────────────────────────────────────────────┤
│                    c-gull (as libc)                          │
│     Provides C ABI libc functions, backed by Rust            │
├─────────────────────────────────────────────────────────────┤
│                    Linux Syscalls                            │
├─────────────────────────────────────────────────────────────┤
│                    LevitateOS Kernel                         │
└─────────────────────────────────────────────────────────────┘
```

**Benefit**: Any Linux program just works - no modifications needed

---

## Understanding the Projects

| Project | What It Is | Use Case |
|---------|-----------|----------|
| **Eyra** | Rust std replacement | Rust programs (requires Cargo.toml change) |
| **c-gull** | Rust libc implementation | C programs or libc-based std |
| **c-ward** | Parent project of c-gull | Contains c-gull + c-scape |
| **c-scape** | Low-level libc subset | Used by c-gull |
| **origin** | Program startup in Rust | Replaces crt1.o |
| **Mustang** | Build system for c-gull | Custom targets + build-std |

**Key Insight**: Eyra internally uses c-gull! They share the same syscall backend.

---

## Syscall Coverage Status

**cgull-test** (in `crates/userspace/eyra/cgull-test/`) tests the syscalls both Eyra and c-gull need.

### Test Results: 19/19 PASS

| Tier | Syscalls | Status |
|------|----------|--------|
| Basic I/O | write, writev | PASS |
| Memory | brk, mmap, munmap | PASS |
| Time | clock_gettime, nanosleep | PASS |
| Random | getrandom | PASS |
| Process | getpid, getuid | PASS |
| Environment | args, env, getcwd | PASS |
| Files | open, read, close, stat, mkdir, readdir | PASS |
| Pipes | pipe2 | PASS |
| Signals | sigprocmask | PASS |

### How to Run cgull-test

```bash
# 1. Build the test binary (if not already built)
cd crates/userspace/eyra
cargo build --release --target x86_64-unknown-linux-gnu -p cgull-test

# 2. Copy to initramfs (should already be there)
cp target/x86_64-unknown-linux-gnu/release/cgull-test ../../initrd_root/

# 3. Rebuild initramfs
cargo xtask build initramfs

# 4. Run in VM and execute test
cargo xtask run
# At shell prompt: cgull-test
```

Expected output:
```
╔══════════════════════════════════════════════════════════════╗
║        C-GULL / EYRA SYSCALL COMPATIBILITY TEST              ║
╚══════════════════════════════════════════════════════════════╝

── TIER 1: Basic I/O ──────────────────────────────────────────
[PASS] write() - you're reading this
[PASS] writev() - println! works
...
╔══════════════════════════════════════════════════════════════╗
║                         SUMMARY                              ║
╠══════════════════════════════════════════════════════════════╣
║  Passed:  19                                                 ║
║  Failed:   0                                                 ║
║  Total:   19                                                 ║
╚══════════════════════════════════════════════════════════════╝

🎉 All tests passed! LevitateOS is ready for c-gull programs.
```

---

## Migration Path: Three Options

### Option 1: Mustang Targets (Recommended)

Use Mustang's build system to create custom targets for LevitateOS.

**Steps:**
1. Install Mustang: `cargo install mustang`
2. Set `RUST_TARGET_PATH` to mustang targets directory
3. Add to coreutils: `mustang::can_run_this!();` (one line)
4. Build: `cargo build --target=x86_64-mustang-linux-gnu -Z build-std`

**Pros:**
- Well-tested approach
- Minimal source modification (one macro)
- Works with nightly Rust

**Cons:**
- Requires nightly toolchain
- Requires `-Z build-std`
- Still needs that one macro line

### Option 2: Custom LevitateOS Target

Create a custom target JSON that uses c-gull as the libc.

**Steps:**
1. Create `x86_64-levitateos.json` target spec
2. Configure linker to use c-gull static library
3. Build with `-Z build-std=std,core,alloc`

**Pros:**
- No source modifications at all
- Full control over linking

**Cons:**
- Complex setup
- Need to maintain target specs
- Requires building std from source

### Option 3: Pre-built Sysroot

Build a complete sysroot with std pre-compiled against c-gull.

**Steps:**
1. Build c-gull as `libc.a`
2. Build Rust std against this libc
3. Package as sysroot
4. Point rustc at this sysroot

**Pros:**
- True "just works" for any Rust program
- Fast builds (no build-std)

**Cons:**
- Complex to set up initially
- Need to rebuild sysroot for Rust updates

---

## Recommended Path Forward

### Phase 1: Verify Syscall Coverage (DONE)
- [x] cgull-test passes 19/19
- [x] All Eyra-required syscalls implemented

### Phase 2: Test Mustang Approach
1. Install Mustang tooling
2. Test building a simple Rust program with mustang target
3. Verify it runs on LevitateOS

### Phase 3: Build Original Coreutils
1. Clone upstream uutils/coreutils (not sunfishcode's fork)
2. Add minimal mustang integration
3. Build for LevitateOS
4. Test all utilities work

### Phase 4: Remove Eyra
1. Switch coreutils submodule to our mustang-integrated fork
2. Remove sunfishcode/coreutils dependency
3. Remove Eyra from workspace
4. Update build system

---

## Files to Remove (After Migration)

Once c-gull/Mustang migration is complete:

```
crates/userspace/eyra/
├── cgull-test/        # KEEP - useful for testing
├── coreutils/         # REPLACE with Mustang-built version
├── brush/             # UPDATE to use Mustang
├── eyra-hello/        # REMOVE - no longer needed
├── eyra-test-runner/  # REMOVE
├── libsyscall/        # KEEP - raw syscalls still useful
├── libsyscall-tests/  # KEEP
└── syscall-conformance/ # KEEP
```

---

## References

- [c-ward](https://github.com/sunfishcode/c-ward) - Rust libc implementation
- [c-gull](https://github.com/sunfishcode/c-ward/tree/main/c-gull) - libc ABI layer
- [Eyra](https://github.com/sunfishcode/eyra) - Rust std replacement (uses c-gull)
- [Mustang](https://github.com/sunfishcode/mustang) - Build system for c-gull programs
- [origin](https://github.com/sunfishcode/origin) - Program startup in Rust

---

## Team Log

| Date | Team | Action |
|------|------|--------|
| 2026-01-11 | TEAM_432 | Created migration plan document |
