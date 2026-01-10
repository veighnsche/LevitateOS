# Phase 4 — Step 1: Userspace x86_64 Compatibility

## Parent
[Phase 4: Integration and Testing](phase-4.md)

## Goal
Port userspace libraries and binaries to build and run on x86_64.

## Prerequisites
- Phase 3 complete — x86_64 kernel boots and runs basic code
- Serial/VGA output working for debugging

---

## UoW 1.1: Add x86_64 Syscall Entry to libsyscall

**Goal**: Implement the `syscall` instruction wrapper for x86_64.

**File**: `userspace/libsyscall/src/arch/x86_64.rs` (new)

**Tasks**:
1. Create `userspace/libsyscall/src/arch/x86_64.rs`
2. Implement `syscall0` through `syscall6` functions:
   - System call number in RAX
   - Args in: RDI, RSI, RDX, R10, R8, R9 (x86-64 Linux ABI)
   - Return value in RAX
   - Clobbers: RCX, R11 (used by syscall instruction)
3. Use inline assembly with `syscall` instruction
4. Match function signatures with AArch64 versions

**Exit Criteria**:
- `syscall!` macro works on x86_64
- Calling convention matches Linux x86-64 ABI

**Verification**:
- Build libsyscall for x86_64 target

---

## UoW 1.2: Add x86_64 Module to libsyscall

**Goal**: Wire x86_64 arch module into libsyscall.

**Files**: 
- `userspace/libsyscall/src/arch/mod.rs` (new or modify)
- `userspace/libsyscall/src/lib.rs` (modify)

**Tasks**:
1. Create/update `src/arch/mod.rs`:
   - `#[cfg(target_arch = "aarch64")] mod aarch64;`
   - `#[cfg(target_arch = "x86_64")] mod x86_64;`
   - Re-export syscall functions
2. Ensure `syscall!` macro uses arch-specific functions
3. Verify no AArch64-specific code in common modules

**Exit Criteria**:
- `cargo build --target x86_64-unknown-none` succeeds for libsyscall

**Verification**:
```bash
cd userspace && cargo build -p libsyscall --target x86_64-unknown-none --release
```

---

## UoW 1.3: Port ulib Entry Point for x86_64

**Goal**: Implement `_start` for x86_64 userspace binaries.

**File**: `userspace/ulib/src/arch/x86_64.rs` (new)

**Tasks**:
1. Create `userspace/ulib/src/arch/x86_64.rs`
2. Implement `_start`:
   - Extract argc from [RSP]
   - Extract argv from [RSP + 8]
   - Call user's `main(argc, argv)`
   - Call `sys_exit(return_value)`
3. Handle TLS pointer if needed (FS segment base)
4. Add `#[cfg]` gates in `ulib/src/lib.rs`

**Exit Criteria**:
- User binaries have working entry point
- argc/argv accessible in main

**Verification**:
- Build a test binary, run in QEMU

---

## UoW 1.4: Verify Syscall Dispatch in Kernel

**Goal**: Ensure kernel handles x86_64 `syscall` instruction.

**File**: `kernel/src/arch/x86_64/syscall.rs` (new)

**Tasks**:
1. Create `syscall.rs` for x86_64
2. Implement MSR setup for SYSCALL/SYSRET:
   - IA32_STAR (0xC0000081): segment selectors
   - IA32_LSTAR (0xC0000082): syscall entry point
   - IA32_FMASK (0xC0000084): RFLAGS mask
3. Implement syscall entry point (assembly):
   - Save user RSP to per-CPU area
   - Switch to kernel stack
   - Save registers
   - Call Rust dispatcher
   - Restore and SYSRET
4. Wire to existing `syscall_dispatch` function

**Exit Criteria**:
- User syscall reaches kernel dispatcher
- Return value comes back correctly

**Verification**:
- `sys_write` to serial works from userspace

---

## UoW 1.5: Build levbox for x86_64

**Goal**: Compile all levbox binaries for x86_64.

**Files**: `userspace/levbox/Cargo.toml`

**Tasks**:
1. Verify no AArch64-specific code in levbox
2. Ensure libsyscall and ulib x86_64 support is complete
3. Run build:
   ```bash
   cd userspace && cargo build -p levbox --target x86_64-unknown-none --release
   ```
4. Fix any platform-specific issues

**Exit Criteria**:
- All levbox binaries compile for x86_64
- No runtime AArch64 assumptions

**Verification**:
- List binaries in `target/x86_64-unknown-none/release/`

---

## UoW 1.6: Port init Binary

**Goal**: Ensure init runs correctly on x86_64.

**File**: `userspace/init/src/main.rs`

**Tasks**:
1. Verify no hardcoded AArch64 assumptions
2. Ensure correct stack setup via ulib entry
3. Verify child spawning works (fork/exec or spawn syscall)
4. Test with minimal init that spawns shell

**Exit Criteria**:
- init runs on x86_64
- Can spawn child processes

**Verification**:
- Boot QEMU, see init output

---

## Progress Tracking
- [ ] UoW 1.1: Syscall Entry
- [ ] UoW 1.2: Arch Module
- [ ] UoW 1.3: ulib Entry
- [ ] UoW 1.4: Kernel Dispatch
- [ ] UoW 1.5: Build levbox
- [ ] UoW 1.6: Port init

## Dependencies Graph
```
UoW 1.1 ──→ UoW 1.2 ──→ UoW 1.3 ──→ UoW 1.5 ──→ UoW 1.6
                              ↓
UoW 1.4 ──────────────────────┘
```
