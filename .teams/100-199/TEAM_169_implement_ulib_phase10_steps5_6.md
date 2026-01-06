# TEAM_169: Implement ulib Phase 10 Steps 5-6

## Objective
Implement argument/environment passing (argc/argv/envp) to userspace.

## Context
- **Plan:** `docs/planning/ulib-phase10/phase-3.md`
- **Previous:** TEAM_168 (Steps 3-4)
- **Scope:** Steps 5-6 (arg/env passing + ulib abstractions)

## Implementation Progress

### Step 5: Kernel Arg/Env Passing ✅
- [x] UoW 1: Stack-based argument setup
  - Added `setup_stack_args()` to `kernel/src/task/user_mm.rs`
  - Linux ABI compatible stack layout
  - Supports argc, argv[], NULL, envp[], NULL
- [x] UoW 2: Pass argc/argv/envp to _start
  - Added `spawn_from_elf_with_args()` to `kernel/src/task/process.rs`
  - Original `spawn_from_elf()` delegates with empty args

### Step 6: ulib Environment Abstractions ✅
- [x] UoW 1: Create env module
  - Created `userspace/ulib/src/env.rs`
  - `init_args(sp)` parses stack at startup
- [x] UoW 2: Implement args() and vars()
  - `args()` -> `Args` iterator over argv
  - `vars()` -> `Vars` iterator over envp (KEY=VALUE pairs)
  - `arg(index)` -> specific argument
  - `var(name)` -> specific environment variable

## Status
- COMPLETE

## Files Modified

### Kernel
- `kernel/src/task/user_mm.rs` — Added `setup_stack_args()` function
- `kernel/src/task/process.rs` — Added `spawn_from_elf_with_args()`

### Userspace
- `userspace/ulib/src/lib.rs` — Added env module
- `userspace/ulib/src/env.rs` — NEW: Environment/argument access

## Usage

### In _start (crt0):
```rust
// Call before modifying SP
unsafe { ulib::env::init_args(sp as *const usize); }
```

### In application:
```rust
// Get all arguments
for arg in ulib::env::args() {
    println!("Arg: {}", arg);
}

// Get specific argument
if let Some(first) = ulib::env::arg(0) {
    println!("Program: {}", first);
}

// Get environment variable
if let Some(path) = ulib::env::var("PATH") {
    println!("PATH={}", path);
}
```

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [x] Code comments include TEAM_169

## Next Steps (for future teams)
1. Steps 7-8: Time syscalls (nanosleep) + Instant/Duration
2. Step 9: Integration demo
3. Update userspace binaries to call `env::init_args()` in _start
