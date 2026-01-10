# Phase 3: Implementation

## Objective

Port brush shell to run on Eyra/LevitateOS.

## Prerequisites

- **Phase 0 complete** (epoll + eventfd syscalls implemented in kernel)
- Phase 1 complete (discovery)
- Phase 2 complete (design decisions made)
- All blocking questions answered

## ✅ Resolved: `_start` Symbol Conflict

TEAM_381 centralized the `-nostartfiles` flag in `.cargo/config.toml` at the Eyra workspace level. This prevents the duplicate `_start` symbol conflict (Origin vs GCC's crt1.o).

**No additional work needed** — brush will automatically inherit this configuration.

## Implementation Steps

### Step 1: Setup brush in Eyra Workspace
- [ ] Add brush-shell as dependency or vendor
- [ ] Create `crates/userspace/eyra/brush/` structure
- [ ] Add to Eyra workspace Cargo.toml
- [ ] Create build.rs with libgcc_eh stub (for aarch64, see eyra-hello/build.rs)

### Step 2: Verify Eyra Compatibility
- [ ] Test compilation with Eyra toolchain
- [ ] Check nix crate syscall compatibility
- [ ] Implement any missing syscalls in kernel
- [ ] Test basic compilation

### Step 3: Verify reedline Works
- [ ] Test terminal input/output via Eyra
- [ ] Verify escape sequences work
- [ ] Test cursor movement

### Step 4: Core Shell Features
- [ ] Basic REPL working
- [ ] Command execution (fork/exec)
- [ ] Builtin commands (cd, echo, exit, etc.)
- [ ] Environment variables

### Step 5: Scripting Features
- [ ] Variable assignment and expansion ($VAR)
- [ ] Control flow (if/then/fi, for, while)
- [ ] Functions
- [ ] Script file execution (bash script.sh)

### Step 6: Advanced Features
- [ ] Tab completion (bash-completion)
- [ ] History
- [ ] Job control (bg, fg, jobs)
- [ ] Signal handling (Ctrl+C, Ctrl+Z)

## Estimated Effort

| Step | Complexity | Time Estimate |
|------|------------|---------------|
| Step 1 | Low | 1-2 hours |
| Step 2 | Medium | 2-4 hours |
| Step 3 | Low | 1-2 hours |
| Step 4 | Medium | 2-4 hours |
| Step 5 | Low | 1-2 hours (brush handles this) |
| Step 6 | Medium | 2-4 hours |

**Total: 10-18 hours** (brush already implements most features)

## Success Criteria

- [ ] brush compiles for Eyra target
- [ ] Basic REPL works on LevitateOS
- [ ] Can execute external commands
- [ ] Bash scripts execute correctly
- [ ] .bashrc-style config works
