# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Building

```bash
# Build everything (kernel + userspace + initramfs + coreutils)
cargo xtask build all

# Build specific components
cargo xtask build kernel      # Kernel only
cargo xtask build userspace   # Userspace + initramfs
cargo xtask build initramfs   # Create initramfs only
cargo xtask build iso         # Bootable Limine ISO (x86_64)

# Build c-gull sysroot and external projects
cargo xtask build sysroot     # Build libc.a from c-gull
cargo xtask build coreutils   # Build uutils/coreutils (clones if needed)
cargo xtask build brush       # Build brush shell (clones if needed)
```

### Running

```bash
# Basic run (default: x86_64 with GUI)
cargo xtask run

# Run with debugging
cargo xtask run --gdb          # Start GDB server on port 1234
cargo xtask run --gdb --wait   # Wait for GDB connection before starting

# Display modes
cargo xtask run --term         # Terminal mode (no GUI)
cargo xtask run --vnc          # VNC display for browser verification
cargo xtask run --headless     # No display

# Architecture selection
cargo xtask --arch aarch64 run
cargo xtask --arch x86_64 run  # Default

# Profiles (aarch64 only)
cargo xtask run --profile pixel6  # 8GB RAM, 8 cores, cortex-a76, GICv3

# Special modes
cargo xtask run --iso          # Force ISO boot
cargo xtask run --test         # Run internal OS tests
cargo xtask run --verify-gpu   # Verify GPU display via VNC
```

### Testing

```bash
# Run all tests
cargo xtask test

# Run specific test suites
cargo xtask test unit          # Host-side unit tests
cargo xtask test behavior      # Boot output comparison against golden reference
cargo xtask test regress       # Static analysis and regression checks

# Integration tests
cargo xtask test debug         # Debug tools integration tests
cargo xtask test serial        # Serial input tests
cargo xtask test keyboard      # Keyboard input tests
cargo xtask test shutdown      # Shutdown tests
cargo xtask test screenshot    # Alpine Linux screenshot tests
cargo xtask test levitate      # LevitateOS display tests

# Update golden files when behavior intentionally changes
cargo xtask test behavior --update
cargo xtask test debug --update

# Configure golden file ratings (gold = strict, silver = auto-update)
# Edit xtask.toml to set files as "silver" during active development
# See docs/development/silver-golden-files.md for details
```

### VM Interaction

```bash
# Persistent VM session
cargo xtask vm start           # Start VM session
cargo xtask vm stop            # Stop VM session
cargo xtask vm send "ls"       # Send keystrokes to running VM
cargo xtask vm exec "ls"       # Execute command in fresh VM (slower)
cargo xtask vm screenshot      # Take screenshot of running VM
cargo xtask vm regs            # Dump CPU registers
cargo xtask vm mem 0x1000      # Dump memory at address
```

### Disk Management

```bash
cargo xtask disk create        # Create disk image if missing
cargo xtask disk install       # Install userspace binaries to disk
cargo xtask disk status        # Show disk image status
```

### Utilities

```bash
cargo xtask check              # Run preflight checks
cargo xtask clean              # Clean artifacts and QEMU locks
cargo xtask kill               # Kill running QEMU instances
```

## Architecture Overview

**LevitateOS is a General Purpose Unix-Compatible Operating System.**

The goal is to **run any Unix program without modification** - programs compiled for Linux should just work.

### What "General Purpose" Means

| Requirement | Description |
|-------------|-------------|
| **No Source Modification** | Programs compiled for Linux just work |
| **Standard ABI** | Linux syscall interface, not a custom ABI |
| **libc Compatibility** | Provide libc.so that existing binaries link against |
| **POSIX Semantics** | fork, exec, pipes, signals, file descriptors work as expected |

**The Test**: Can a user download a Linux binary and run it? If yes, we're general purpose.

### Technical Overview

LevitateOS is a dual-architecture (AArch64 and x86_64) operating system kernel written in Rust. It uses a modular workspace structure with clear separation between kernel, HAL, and utilities.

### Workspace Structure

```
crates/
├── kernel/              # Main kernel binary (levitate-kernel)
├── hal/                 # Hardware Abstraction Layer (los_hal)
│   └── src/
│       ├── gic.rs       # GICv2/GICv3 auto-detection
│       ├── mmu.rs       # Page tables & translation
│       ├── console.rs   # Early console
│       ├── timer.rs     # Timer abstraction
│       └── ...
├── utils/               # Core utilities (los_utils)
│   └── src/
│       ├── spinlock.rs  # Spinlock
│       ├── ringbuffer.rs # Ring buffer
│       └── cpio.rs      # CPIO parser
├── term/                # ANSI terminal emulator (los_term)
├── gpu/                 # VirtIO GPU library (los_gpu)
├── pci/                 # PCI bus support (los_pci)
├── virtio-transport/    # VirtIO transport layer
├── drivers/             # Device drivers
│   ├── virtio-blk/
│   ├── virtio-gpu/
│   ├── virtio-input/
│   ├── virtio-net/
│   ├── nvme/
│   ├── xhci/
│   └── simple-gpu/
├── error/               # Error handling (los_error)
├── traits/              # Device traits
│   ├── input-device/
│   ├── network-device/
│   └── storage-device/
└── userspace/           # Userspace programs
    └── eyra/            # Eyra-based utilities (provides std support)
        └── coreutils/   # Git submodule: uutils coreutils

xtask/                   # Development task runner
docs/                    # Documentation
tests/                   # Golden files for behavior tests
```

### Multi-Architecture Support

LevitateOS supports both AArch64 and x86_64 through a layered abstraction:

1. **Kernel Architecture Layer** (`kernel/src/arch/`): Each architecture implements:
   - `SyscallFrame`: Register state during syscalls
   - `SyscallNumber`: Platform-specific syscall mapping
   - `Stat` / `Timespec`: Platform-specific metadata layouts
   - `cpu::wait_for_interrupt()`: Idle loop

2. **HAL Traits** (`los_hal/src/traits.rs`):
   - `InterruptController`: Generic interface for GIC (ARM) or APIC (x86)
   - `MmuInterface`: Generic interface for page tables

3. **Userspace ABI** (`userspace/libsyscall/src/arch/`):
   - AArch64: `svc #0` with x8-x0 registers
   - x86_64: `syscall` with rax, rdi, rsi registers

### Boot Sequence

1. **Assembly Entry** (`_start`): Disable interrupts, enable FP/SIMD, zero BSS, setup early page tables, enable MMU
2. **Heap Init**: Initialize `linked_list_allocator` from linker script bounds
3. **MMU**: Re-initialize with higher-half mappings (2MB blocks)
4. **Drivers**: Exception vectors → UART → GIC (auto-detect v2/v3) → Timer
5. **Memory**: Buddy allocator from memory map
6. **VirtIO**: Scan bus for GPU, Input, Block, Network devices
7. **Filesystem**: Mount filesystems, parse initramfs
8. **Main Loop**: Interactive shell with task scheduling

### Key Architectural Patterns

**Higher-Half Kernel**: Kernel runs in virtual address space `0xFFFF_8000_0000_0000`

**Linux ABI Compatibility**: LevitateOS implements the Linux AArch64 and x86_64 syscall ABIs to support unmodified Rust `std` binaries. Critical details:
- `Stat` struct must be exactly 128 bytes on AArch64
- Auxiliary vector (auxv) required on stack for `std::rt` initialization
- `TPIDR_EL0` (AArch64) / `FS` register (x86_64) must be context-switched for TLS
- `writev()`/`readv()` required for Rust `println!`
- Errno values must match Linux (e.g., `ENOSYS = 38`)

**VFS Layer**: Linux-inspired Virtual File System with:
- Superblock → Inode → Dentry → File hierarchy
- Mount support for multiple filesystems
- tmpfs, FAT32, ext4 (read-only), CPIO initramfs support

**Memory Management**:
- Buddy allocator for physical pages
- Slab allocator for kernel objects
- VMM (Virtual Memory Area) management inspired by Redox `rmm`
- Support for `mmap`, `brk`, demand paging

### Error Handling

All kernel errors use typed enums with numeric codes defined via `define_kernel_error!` macro:

```rust
define_kernel_error! {
    /// My subsystem errors (0x10xx)
    pub enum MyError(0x10) {
        /// Description
        SomethingWrong = 0x01 => "Message",
        /// Nested error
        Other(InnerError) = 0x02 => "Nested error",
    }
}
```

Error code format: `0xSSCC` where `SS` = subsystem, `CC` = code within subsystem.

## Development Guidelines

### Rule 0: Quality Over Speed

**Take the correct architectural path, never the shortcut.**

- Prefer clean designs over quick fixes
- Avoid wrappers, shims, indirection unless truly necessary
- Leave the codebase better than you found it
- Future teams inherit your decisions — choose debt-free solutions

### Core Rules (from `.agent/rules/kernel-development.md`)

**Rule 4: Silence is Golden**
- Production builds produce NO output on success
- Errors must be loud and immediate
- Use `--features verbose` for behavior testing
- Default log level: `warn` (production), `trace` (verbose builds)

**Rule 5: Memory Safety**
- Minimize `unsafe` blocks
- Every `unsafe` requires `// SAFETY:` comment explaining soundness
- Wrap unsafe in safe abstractions using Newtype pattern
- Use RAII for all resource management

**Rule 6: Robust Error Handling**
- All fallible operations return `Result<T, E>`
- Use `Option<T>` for potentially missing values
- NO `unwrap()`, `expect()`, or `panic!` (enforced by clippy)
- Use `?` operator for error propagation

**Rule 14: Fail Loud, Fail Fast**
- When you must fail, fail noisily and immediately
- Return specific `Err` variants
- Do not attempt partial recovery if internal state is corrupted
- Use `debug_assert!` for internal invariants

**Rule 20: Simplicity > Perfection**
- Implementation simplicity is the highest priority
- Favor clear Rust code over complex perfect solutions
- If handling a rare edge case doubles complexity, return an `Err`

### Code Quality Rules

**Breaking Changes > Fragile Compatibility**
- Favor clean breaks over compatibility hacks
- Move or rename the type/function, let the compiler fail
- Fix import sites one by one
- Remove temporary re-exports or legacy names
- If writing adapters to "keep old code working," stop — fix the actual call sites

**No Dead Code**
- Remove unused functions, modules, commented-out code
- "Kept for reference" logic belongs in git history, not the codebase
- The repository must contain only living, active code

**Modular Refactoring**
- When splitting large modules, each module owns its own state
- Keep fields private — expose intentional APIs
- Avoid deep relative imports
- File sizes: < 1000 lines preferred, < 500 ideal
- Organize by responsibility, not convenience

### Testing Philosophy (from `.agent/rules/behavior-testing.md`)

**All Behaviors Must Be Documented**
- Maintain `docs/testing/behavior-inventory.md` with all testable behaviors
- Each behavior gets a unique ID (e.g., `[MMU1]`, `[R2]`)
- IDs must appear in source code, test code, AND inventory

**Traceability Example**:
```rust
/// [R2] Push adds element, [R4] returns false when full
pub fn push(&mut self, byte: u8) -> bool { /* ... */ }

#[test]
/// Tests: [R1] new empty, [R2] push, [R3] FIFO, [R4] full
fn test_ring_buffer_fifo() { /* ... */ }
```

**Critical Rule: ALL TESTS MUST PASS**
- NEVER dismiss a failing test as "pre-existing" without investigation
- Test failures require root cause analysis:
  - Did YOUR changes cause it? → Fix your code
  - Is the change intentional? → Update golden files with `--update`
  - Is the test buggy? → Fix the test
  - Truly unrelated? → PROVE IT with git blame
- The cost of dismissal is high: you will be asked to redo the work

**Test Levels**:
- **Unit Tests**: Host-side tests with `--features std` (mocks for hardware)
- **Behavior Tests**: Boot kernel with `--features verbose`, compare to golden log
- **Regression Tests**: Static analysis for API consistency, constant sync, code patterns

**Golden File Updates**:
When kernel behavior intentionally changes:
1. Run test suite (it will fail with diff)
2. Review diff carefully
3. If correct, update: `cargo xtask test behavior --update`
4. Commit with explanation

**Silver Mode** (Active Development):
Golden logs can be marked "silver" in `xtask.toml` during active development:
- Silver files auto-update when tests fail
- Use when behavior is still stabilizing
- Promote to "gold" once behavior is finalized
- See `docs/development/silver-golden-files.md` for details

### Code Organization

**Module Boundaries**:
- Kernel logic: `crates/kernel/src/`
- Hardware abstraction: `crates/hal/src/`
- Shared utilities: `crates/utils/src/`
- Device drivers: `crates/drivers/*/`

**Feature Flags** (kernel):
- `verbose`: Enable boot messages for testing
- `diskless`: Skip initrd requirement
- `multitask-demo`: Enable demo tasks
- `verbose-syscalls`: Enable syscall logging

**Crate Naming**: All library crates use `los_` prefix (LevitateOS).

### Important Patterns

**Architecture-Specific Code**:
```rust
#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::...;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::...;
```

**Verbose Logging**:
```rust
#[cfg(feature = "verbose")]
log::debug!("Verbose message");
```

**Safe Hardware Abstraction**:
```rust
// SAFETY: Writing to MMIO address 0x... is safe because the
// device specification guarantees this register is write-only
// and has no side effects on other system state.
unsafe { ptr::write_volatile(addr, value) }
```

**Error Definition**:
See `docs/planning/error-macro/phase-1.md` for subsystem allocation.

## Critical Knowledge Areas

### Linux ABI Compatibility

When modifying syscalls or userspace interaction, consult:
- `docs/specs/LINUX_ABI_GUIDE.md` - Critical gotchas and patterns
- `docs/specs/userspace-abi.md` - Definitive ABI specification

Key considerations:
- Structure padding and alignment (especially `Stat` at 128 bytes)
- Auxiliary vector on process stack (`AT_PAGESZ`, `AT_RANDOM`, `AT_PHDR`, etc.)
- TLS register context switching (`TPIDR_EL0` on AArch64, `FS` on x86_64)
- Errno value alignment with Linux (use constants from `kernel/src/syscall/mod.rs::errno`)
- Vectored I/O for `println!` (`writev`/`readv`)

### c-gull Sysroot (CRITICAL - READ THIS)

**What is c-gull?** c-gull (from [c-ward](https://github.com/sunfishcode/c-ward)) provides a pure-Rust libc implementation. We build it as a static library (`libc.a`) that any Rust program can link against, enabling **UNMODIFIED** upstream projects to run on LevitateOS.

**Status**: c-gull sysroot implemented. External projects (coreutils, brush) are cloned at build time.

#### How It Works

1. **Sysroot**: `toolchain/sysroot/lib/libc.a` - pre-built libc
2. **External projects**: Cloned to `toolchain/` at build time (gitignored)
3. **Build**: Projects built with RUSTFLAGS pointing to sysroot
4. **No source modifications** - upstream repos work as-is

#### Build Commands
```bash
cargo xtask build sysroot      # Build libc.a from c-gull
cargo xtask build coreutils    # Clone & build uutils/coreutils
cargo xtask build brush        # Clone & build brush shell
cargo xtask build all          # Everything (auto-builds deps if missing)
```

#### Directory Layout
```
toolchain/
├── libc-levitateos/     # Our wrapper crate (committed)
├── c-ward/              # Cloned at build time (gitignored)
├── coreutils/           # Cloned at build time (gitignored)
├── brush/               # Cloned at build time (gitignored)
├── sysroot/lib/libc.a   # Built output (gitignored)
└── coreutils-out/       # Built binaries (gitignored)
```

#### Current Limitations

- **Static linking only** - no dynamic linker (`ld.so`) yet
- **Limited utilities** - some coreutils need libc functions c-gull doesn't have yet (e.g., `getpwuid` for `ls`)
- **x86_64 tested** - aarch64 may need linker flag adjustments

#### Future: Dynamic Linking

Once we implement `ld.so`, we can:
- Support dynamically linked binaries
- Download pre-built binaries instead of building from source
- Full Linux binary compatibility

### Memory Layout

| Region | Physical Address | Virtual Address | Notes |
|--------|------------------|-----------------|-------|
| Device MMIO | `0x0000_0000..0x4000_0000` | Identity mapped | VirtIO, UART, GIC |
| Kernel Start | `0x4008_0000` (AArch64) | `0xFFFF_8000_4008_0000` | Higher-half |
| Kernel Heap | After kernel | Higher-half mapped | Buddy allocator |

### QEMU Profiles

| Profile | RAM | Cores | CPU | GIC/APIC |
|---------|-----|-------|-----|----------|
| Default (aarch64) | 512MB | 1 | cortex-a53 | GICv2 |
| Pixel 6 (aarch64) | 8GB | 8 | cortex-a76 | GICv3 |
| Default (x86_64) | 32GB | 2+ | i3 | APIC |

## Team Workflow

### Team Files Are Critical Historical Context

**Team files (`.teams/TEAM_XXX_*.md`) are the primary source of historical context for this project.** Future teams (including future conversations) rely on these files to understand:
- What was attempted and why
- What worked and what didn't
- Key decisions and their rationale
- Gotchas discovered during implementation
- Unfinished work and next steps

**You MUST maintain your team file throughout the conversation**, not just at the end.

### Team Registration

Every distinct AI conversation = one team. Implementation details and decisions are tracked in `.teams/` with TEAM_XXX identifiers.

**Creating a Team** (do this EARLY in the conversation):
1. Check `.teams/` subdirectories (000-099, 100-199, 200-299, 300-399, 400+) for existing teams
2. Your number = highest existing + 1
3. Create log file: `.teams/TEAM_XXX_<brief_summary>.md`
4. Team ID is permanent for the lifetime of the conversation

**Team File Format**:
```markdown
# TEAM_XXX: Brief Title

## Objective
What this team is trying to accomplish.

## Progress Log
### Session 1 (YYYY-MM-DD)
- What was done
- Key decisions made
- Problems encountered

## Key Decisions
- Decision 1: Rationale
- Decision 2: Rationale

## Gotchas Discovered
- Issue 1: How to avoid/fix

## Remaining Work
- [ ] Task 1
- [ ] Task 2

## Handoff Notes
Summary for the next team.
```

**Code Comments**: When modifying code, add traceability:
```rust
// TEAM_XXX: Reason for change
```

### Continuous Team File Updates

**Update your team file as you work, not just at the end:**

| When | What to Log |
|------|-------------|
| Starting work | Objective, initial approach |
| Making key decisions | Decision + rationale |
| Hitting a problem | What went wrong, how you solved it |
| Discovering a gotcha | The issue and how to avoid it |
| Completing a milestone | What was done, what's next |
| Finishing | Handoff notes, remaining work |

**Why this matters**: Context gets lost between conversations. A detailed team file lets the next team (or a resumed conversation) pick up exactly where you left off without re-discovering the same issues.

### Before Starting Work

Every team must:
1. Read the main project overview
2. Read the current active phase in `docs/planning/`
3. **Check recent team logs in `.teams/`** - understand what previous teams did
4. Check open questions in `docs/questions/`
5. **Claim a team number and create a team file immediately**
6. Ensure all tests pass before making changes
7. Only then begin implementation

### Before Finishing

Every team must:
- **Update their team file with complete progress and handoff notes**
- Ensure project builds (`cargo xtask build all`)
- Ensure all tests pass (`cargo xtask test`)
- Document remaining problems or blockers
- Add any discovered gotchas to `docs/GOTCHAS.md`

**Handoff Checklist**:
- [ ] Project builds cleanly
- [ ] All tests pass
- [ ] Behavioral regression tests pass
- [ ] **Team file has complete progress log**
- [ ] **Remaining TODOs documented in team file**
- [ ] **Key decisions documented with rationale**
- [ ] **Any gotchas added to docs/GOTCHAS.md**

### Questions

If requirements are ambiguous, decisions conflict, or something feels "off":
- Create a question file in `docs/questions/TEAM_XXX_*.md`
- Ask the USER before proceeding
- Never guess on major decisions

### Planning Documents

Planning documents live in `docs/planning/<feature-name>/`, NOT in `.plans/` or other locations.

## References

- **Roadmap**: `docs/ROADMAP.md` - Development phases
- **Architecture**: `docs/ARCHITECTURE.md` - Design principles and workspace structure
- **Behavior Inventory**: `docs/testing/behavior-inventory.md` - All testable behaviors
- **QEMU Profiles**: `docs/QEMU_PROFILES.md` - Hardware emulation configurations
- **Contributing**: `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`
- **Security**: `SECURITY.md` - Security vulnerability reporting
