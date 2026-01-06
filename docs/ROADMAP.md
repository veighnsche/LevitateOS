# LevitateOS Roadmap

**Last Updated:** 2026-01-06 (TEAM_208)

This document outlines the planned development phases for LevitateOS. Each completed item includes the responsible team for traceability.

---

## ✅ Phase 1: Foundation & Refactoring (Completed)

- **Objective**: Establish a modular, idiomatic Rust codebase.
- **Achievements**:
  - [x] Migrated to Cargo Workspace (`levitate-kernel`, `levitate-hal`, `levitate-utils`). (TEAM_009)
  - [x] Integrated `linked_list_allocator` for heap management.
  - [x] Basic UART (Console) and GIC (Interrupt) drivers.
  - [x] Basic VirtIO GPU and Input support.

---

## ✅ Phase 2: Idiomatic HAL & Basic Drivers (Completed)

- **Objective**: Harden the Hardware Abstraction Layer (HAL) and implement robust drivers.
- **Tasks**:
  - [x] **Timer**: AArch64 Generic Timer driver. (TEAM_010, TEAM_011)
  - [x] **PL011 UART**: Full PL011 driver with interrupt handling (RX/TX buffers). (TEAM_012, TEAM_014)
  - [x] **GICv2/v3**: Expanded GIC support with typed IRQ routing and FDT discovery. (TEAM_015, TEAM_048)
  - [x] **Safety**: All MMIO uses `volatile`, wrapper structs prevent unsafe state. (TEAM_016, TEAM_017, TEAM_048)

---

## ✅ Phase 3: Memory Management (MMU) (Completed)

- **Objective**: Enable virtual memory and implement higher-half kernel architecture.
- **Tasks**:
  - [x] **Page Tables**: AArch64 page table walking, modification, and optimized 2MB block mappings. (TEAM_018, TEAM_019, TEAM_020)
  - [x] **Identity Mapping**: Initial boot mapping for transition.
  - [x] **Higher-Half Kernel**: Kernel runs at `0xFFFF800000000000` using TTBR1. (TEAM_025, TEAM_026, TEAM_027)
  - [x] **HAL Integration**: `mmu.rs` with `virt_to_phys`/`phys_to_virt` helpers. (TEAM_028)
  - [x] **Kernel Audit**: Documented all behaviors for Phase 2-3 freeze. (TEAM_021, TEAM_022)

---

## ✅ Phase 4: Storage & Filesystem (Completed)

- **Objective**: Persistent storage and basic filesystem access.
- **Tasks**:
  - [x] **VirtIO Block**: Disk driver for QEMU `virtio-blk`. (TEAM_029, TEAM_030)
  - [x] **Filesystem**: FAT32 filesystem using `embedded-sdmmc`. (TEAM_032)
  - [x] **Initramfs**: Load an initial ramdisk for early userspace. (TEAM_035, TEAM_036, TEAM_038, TEAM_039)

---

## ✅ Phase 5: Memory Management II — Dynamic Allocator (Completed)

- **Objective**: Replace the static heap with scalable kernel allocators.
- **Achievements**:
  - [x] **Buddy Allocator**: Physical page allocator for large allocations. (TEAM_048: Dynamic Map)
  - [x] **Slab Allocator**: Fast allocation for fixed-size kernel objects (tasks, file handles). (TEAM_051: Complete)
  - [x] **Page Frame Allocator**: Integration with MMU for on-demand mapping. (TEAM_054: Complete)

---

## ✅ Phase 6: VirtIO Ecosystem Expansion & Hybrid Boot (Completed)

- **Objective**: Expand hardware support and formalize boot architecture.
- **Achievements**:
  - [x] **VirtIO Net**: Basic network packet transmission/reception (`virtio-net`). (TEAM_057)
  - [x] **GPU Refinement**: Text rendering on GPU framebuffer with ANSI support. (TEAM_058, TEAM_059, TEAM_060)
  - [x] **Hybrid Boot Specification**: Formalized boot stages (SEC/PEI/DXE/BDS) and interactive console. (TEAM_061, TEAM_063, TEAM_065)
  - [x] **Keyboard Support**: Direct input from QEMU window via `virtio-keyboard`. (TEAM_032, TEAM_060)
  - [x] **Warning Fixes**: Zero-warning build on bare-metal target. (TEAM_066)
  - [ ] **9P Filesystem**: Mount host directories via `virtio-9p`. (Deferred — see `docs/planning/virtio-ecosystem-phase6/task-6.3-9p-filesystem.md`)

---

## ✅ Phase 7: Multitasking & Scheduler (Completed)

- **Objective**: Run multiple tasks concurrently.
- **Achievements**:
  - [x] **Virtual Memory Reclamation**: `unmap_page()` with TLB invalidation and table reclamation. (TEAM_070)
  - [x] **Context Switching**: Assembly `cpu_switch_to` saves/restores callee-saved registers. (TEAM_070)
  - [x] **Scheduler**: Cooperative `yield_now()` and preemptive Round-Robin via timer interrupts. (TEAM_070)
  - [x] **Task Primitives**: `TaskControlBlock`, `Context`, `TaskId`, `TaskState` with atomic state management. (TEAM_070, TEAM_071)
  - [x] **Idle Task**: Power-efficient `idle_loop()` with `wfi` instruction (Rule 16). (TEAM_071)
  - [x] **Task Exit**: Proper `task_exit()` with state transition and cleanup. (TEAM_071)

> [!NOTE]
> **Demo Mode:** Build with `--features multitask-demo` to enable preemption verification tasks.
> **Plan Docs:** See `docs/planning/multitasking-phase7/` for design decisions and UoW breakdown.

---

## ✅ Phase 8a: Userspace Foundation (Completed)

- **Objective**: Run unprivileged user programs.
- **Achievements**:
  - [x] **EL0 Transition**: Switch CPU from EL1 (Kernel) to EL0 (User). (TEAM_073)
  - [x] **Syscall Interface**: `svc` handler with custom ABI (x8=nr, x0-x5=args). (TEAM_073)
  - [x] **ELF Loader**: Parse and load ELF64 binaries from initramfs. (TEAM_073, TEAM_079)
  - [x] **Device MMIO via TTBR1**: Devices accessible after TTBR0 switch. (TEAM_078)
  - [x] **Basic Syscalls**: `write`, `exit`, `getpid`. (TEAM_073)

> [!NOTE]
> **Milestone:** "Hello from userspace!" executes successfully.

---

## ✅ Phase 8b: Interactive Shell (COMPLETED)

- **Objective**: Boot to an interactive shell prompt with basic levbox.
- **Tasks**:
  - [x] **GPU Terminal Fix**: Fixed userspace output not appearing on GPU. (TEAM_115)
  - [x] **Read Syscall**: Implemented `read(fd, buf, len)` for stdin/keyboard input. (TEAM_081)
  - [x] **Shell Binary**: Userspace `lsh` with prompt, line editing, command parsing. (TEAM_073)
  - [x] **Coreutils**: `echo`, `help`, `clear`, `exit`. (TEAM_073)
  - [ ] **Spawn Syscall**: Execute external programs from initramfs. (Future)

> [!NOTE]
> **Milestone:** Boot → see log on GPU → get `# ` prompt → run commands. ✅ ACHIEVED
> **Verification:** `cargo xtask run-vnc` → Browser → VNC → Shell interactive

---

## ✅ Phase 8c: Userspace Refactor (Completed)

- **Objective**: Eliminate code duplication and establish a modular userspace architecture.
- **Achievements**:
  - [x] **Workspace**: Converted `userspace/` to a Cargo workspace. (TEAM_118)
  - [x] **libsyscall**: Created shared library for syscall wrappers and panic handling. (TEAM_118)
  - [x] **Migration**: Refactored `shell` to use `libsyscall` and cleaned up legacy `hello`. (TEAM_118)
  - [x] **Linker Scripts**: Fixed conflict using per-crate build scripts. (TEAM_118)

---

## ✅ Phase 8d: Process Management (Completed)

- **Objective**: Implement multi-process management and process lifecycle.
- **Achievements**:
  - [x] **Init Process (PID 1)**: Established proper userspace boot sequence. (TEAM_120)
  - [x] **Spawn Syscall**: Kernel support for launching programs from initramfs. (TEAM_120)
  - [x] **Linter Sync**: Synchronize userspace lints with kernel's strict rules. (TEAM_120)
  - [x] **Build Integration**: Standardized userspace build in `xtask`. (TEAM_120)

> [!NOTE]
> **Milestone:** Boot → `init` starts → `init` spawns `shell` → shell is interactive.

---

## 📱 Phase 9: Hardware Targets

- **Current**: QEMU (`virt` machine, AArch64).
- **Next Step**: **Raspberry Pi 4/5** (Standard AArch64, widely documented, accessible UART).
- **Moonshot**: **Pixel 6 (Tensor GS101)**.
  - *Challenges*: Proprietary boot chain (pBL/sBL/ABL).
  - *Strategy*: Align LevitateOS stages (EarlyHAL, Memory, Console) with GS101 hardware (UART via SBU pins, SimpleFB) to ensure "Pixel-ready" architecture. (TEAM_061, TEAM_063)

---

## 🏗️ PART II: USERSPACE EXPANSION & APPS

The goal of Part II is to build a rich, POSIX-like userspace environment on top of the Phase 8 foundations, ultimately enabling **[uutils-levbox](https://github.com/uutils/levbox)** — the Rust reimplementation of GNU levbox.

### 🎯 Target: uutils-levbox Compatibility

> [!IMPORTANT]
> **End Goal**: Run unmodified `uutils-levbox` binaries on LevitateOS.
> 
> **Strategy**: Build our own "Busybox-style" levbox first (Phase 11) to validate the syscall layer, then port Rust `std` to enable uutils.

#### Dependency Chain

```
┌─────────────────────────────────────────────────────────────────────┐
│                    uutils-levbox                                  │
│              (Rust rewrite of GNU levbox)                         │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ depends on
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Rust std library                                 │
│           (std::fs, std::io, std::process, std::env, etc.)          │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ depends on
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│             libc (relibc) + OS-specific backend                     │
│               (std::sys::pal::unix on Linux/POSIX)                  │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ depends on
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│               ~50+ Syscalls with Linux ABI                          │
│   (open, read, write, mmap, brk, getdents, stat, clone, futex...)   │
└─────────────────────────────────────────────────────────────────────┘
```

#### Syscall Gap Analysis for `std` / uutils

| Syscall Category | Required For | Current Status | Phase |
|------------------|--------------|----------------|-------|
| **Memory** | | | |
| `mmap` / `munmap` | Allocator, file mapping | 🔴 Not implemented | 14+ |
| `mprotect` | Stack guard pages | 🔴 Not implemented | 14+ |
| `brk` | Heap allocation | 🟢 Implemented | 10 |
| **Threading** | | | |
| `clone` | Thread creation | 🔴 Not implemented | 14+ |
| `futex` | Mutex, condvar | � Implemented | 17a |
| TLS (`TPIDR_EL0`) | Thread-local storage | 🔴 Not implemented | 14+ |
| `set_tid_address` | Thread ID management | 🔴 Not implemented | 14+ |
| **Signals** | | | |
| `rt_sigaction` | Signal handlers | 🔴 Not implemented | 12 |
| `rt_sigprocmask` | Signal masking | 🔴 Not implemented | 12 |
| `kill` / `tgkill` | Send signals | 🔴 Not implemented | 12 |
| **Process** | | | |
| `fork` / `vfork` | Process creation | 🔴 Not implemented | 12 |
| `execve` | Program execution | 🟡 Have `spawn` | 12 |
| `wait4` / `waitpid` | Child reaping | � Implemented | 8d |
| `getpid` / `getppid` | Process IDs | 🟢 Implemented | 8 |
| **I/O** | | | |
| `pipe` / `pipe2` | Shell pipelines | 🔴 Not implemented | 12 |
| `dup` / `dup2` / `dup3` | FD duplication | 🔴 Not implemented | 12 |
| `ioctl` | TTY control | 🔴 Not implemented | 13 |
| `poll` / `select` | I/O multiplexing | 🔴 Not implemented | 13 |
| **Filesystem** | | | |
| `openat` | Open files | 🟢 Implemented | 10 |
| `read` / `write` | Basic I/O | 🟢 Implemented | 8 |
| `fstat` / `lstat` | File metadata | 🟢 Implemented | 10 |
| `getdents64` | Read directory | 🟢 Implemented | 10 |
| `unlinkat` | Remove files | 🟢 Implemented (tmpfs) | 11 |
| `mkdirat` | Create directory | 🟢 Implemented (tmpfs) | 11 |
| `renameat` | Rename/move | 🟢 Implemented (tmpfs) | 11 |
| `linkat` / `symlinkat` | Create links | � symlinkat impl | 11 |
| `getcwd` | Current directory | 🟢 Implemented | 11 |
| `chdir` / `fchdir` | Change directory | 🔴 Not implemented | 11 |
| `utimensat` | Set timestamps | � Implemented | 11 |

Legend: 🟢 Complete | 🟡 Partial/Wrapper Only | 🔴 Not Started

---

### ⚠️ Phase 11 Blockers (Levbox Utilities)

> **Updated:** 2026-01-06 (TEAM_197)

#### ✅ Resolved Blockers (Tmpfs Complete)

TEAM_194 implemented tmpfs at `/tmp` with full write support:

| Syscall | Status | Notes |
|---------|--------|-------|
| `mkdirat` (34) | 🟢 Complete | Works for `/tmp/*` paths |
| `unlinkat` (35) | 🟢 Complete | Works for `/tmp/*` paths |
| `renameat` (38) | 🟢 Complete | Works for `/tmp/*` paths |
| `openat` with O_CREAT | 🟢 Complete | Creates files in `/tmp` |
| `openat` with O_TRUNC | 🟢 Complete | Truncates files in `/tmp` |
| `read`/`write` for tmpfs | 🟢 Complete | Full read/write support |

#### ✅ All Syscall Blockers Resolved

| Blocker | Status | Team |
|---------|--------|------|
| `mkdirat` (34) | � Complete | TEAM_192 |
| `unlinkat` (35) | 🟢 Complete | TEAM_192 |
| `renameat` (38) | � Complete | TEAM_192 |
| `utimensat` (88) | 🟢 Complete | TEAM_198 |
| `symlinkat` (36) | � Complete | TEAM_198 |
| `readlinkat` (37) | 🟢 Complete | TEAM_204 |

#### Current Utility Status

| Utility | Status | Blocker |
|---------|--------|----------|
| `cat` | 🟢 Complete | None |
| `ls` | 🟢 Complete | None |
| `pwd` | 🟢 Complete | None |
| `mkdir` | 🟢 Works | Tmpfs at `/tmp` |
| `rmdir` | 🟢 Works | Tmpfs at `/tmp` |
| `rm` | 🟢 Works | Tmpfs at `/tmp` |
| `mv` | 🟢 Works | Tmpfs at `/tmp` |
| `cp` | 🟢 Works | Tmpfs at `/tmp` |
| `touch` | � Ready | Syscall ready, utility pending |
| `ln` | � Ready | symlinkat ready, utility pending |

---

### 📋 Phase 10: The Userspace Standard Library (`ulib`) — IN PROGRESS

> **Planning:** See `docs/planning/ulib-phase10/`  
> **Questions:** See `.questions/TEAM_164_ulib_design.md` (7 questions awaiting answers)

- **Objective**: Create a robust `std`-like library to support complex applications.
- **Specification**: See [`docs/specs/userspace-abi.md`](file:///home/vince/Projects/LevitateOS/docs/specs/userspace-abi.md)
- **Units of Work**:
  - [x] **Global Allocator**: Bump allocator (`LosAllocator`) backed by `sbrk` syscall.
  - [x] **File Abstractions**: `File`, `Metadata`, `Read::read()` with initramfs file support (TEAM_178).
  - [x] **Directory Iteration**: `ReadDir`, `DirEntry`, `FileType` with `sys_getdents` (TEAM_176).
  - [x] **Buffered I/O**: `BufReader` and `BufWriter` with `read_line()` support (TEAM_180).
  - [x] **Environment**: `args()`, `vars()`, `var()` parsing from stack (Linux ABI compatible).
  - [x] **Time**: `Duration`, `Instant`, `sleep()`, `sleep_ms()` via `clock_gettime`/`nanosleep` syscalls.
  - [x] **Error Handling**: `Error`, `ErrorKind`, `Result`, `Read`, `Write` traits.

---

### 🛠️ Phase 11: Core Utilities (The "Busybox" Phase)

> **Specifications:** See [`docs/specs/levbox/`](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/README.md) for POSIX-compliant utility specs.

- **Objective**: Implement essential file management and text tools using `ulib` (no `std` dependency).
- **Purpose**: Validate syscall implementation before attempting full `std` port.

#### Kernel Syscalls Required (Phase 11)

| Syscall | Nr (AArch64) | Used By |
|---------|--------------|---------|
| `mkdirat` | 34 | mkdir |
| `unlinkat` | 35 | rm, rmdir |
| `symlinkat` | 36 | ln -s |
| `linkat` | 37 | ln |
| `renameat` | 38 | mv |
| `getcwd` | 17 | pwd |
| `chdir` | 49 | cd (shell) |
| `fchdir` | 50 | cd (shell) |
| `utimensat` | 88 | touch |

#### Utilities

| Utility | Spec | Kernel Deps | Priority |
|---------|------|-------------|----------|
| `cat` | [cat.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/cat.md) | read, write | P0 |
| `ls` | [ls.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/ls.md) | getdents64, fstat | P0 |
| `pwd` | [pwd.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/pwd.md) | getcwd | P0 |
| `mkdir` | [mkdir.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/mkdir.md) | mkdirat | P1 |
| `rmdir` | [rmdir.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/rmdir.md) | unlinkat (AT_REMOVEDIR) | P1 |
| `rm` | [rm.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/rm.md) | unlinkat, getdents64 | P1 |
| `touch` | [touch.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/touch.md) | openat, utimensat | P1 |
| `cp` | [cp.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/cp.md) | read, write, fstat | P2 |
| `mv` | [mv.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/mv.md) | renameat | P2 |
| `ln` | [ln.md](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/ln.md) | linkat, symlinkat | P2 |

---

### ✅ Phase 12: VFS Foundation (Completed)

> **Planning:** See `docs/planning/vfs/`  
> **Team:** TEAM_200+

- **Objective**: Build infrastructure required for a proper Linux-style Virtual Filesystem.
- **Completed**: RwLock, Path abstraction, Mount table, Extended Stat, File mode constants
- **Why Now**: Current ad-hoc file handling (FdType dispatch) doesn't scale. VFS is required for proper multi-filesystem support.

#### Prerequisites to Build

| Component | Priority | Description |
|-----------|----------|-------------|
| **RwLock** | P0 | Readers-writer lock for inode access |
| **Path abstraction** | P0 | Proper path parsing, normalization, resolution |
| **Mount table** | P0 | Track mounted filesystems at mount points |
| **Extended Stat** | P1 | Full POSIX stat: st_dev, st_ino, st_nlink, st_uid, st_gid, st_rdev, st_blksize, st_blocks |
| **File mode constants** | P1 | S_IFREG, S_IFDIR, S_IFLNK, permission bits |

#### Units of Work

- [ ] Implement `RwLock` in `los_utils` or `los_hal`
- [ ] Create `kernel/src/fs/path.rs` with `Path` struct
- [ ] Create `kernel/src/fs/mount.rs` with mount table
- [ ] Extend `Stat` struct in `kernel/src/syscall/mod.rs`
- [ ] Add file mode constants to `kernel/src/fs/mod.rs`

---

### ✅ Phase 13: Core VFS Implementation (Completed)

- **Objective**: Implement Linux-style Virtual Filesystem layer.
- **Critical for**: Unified file abstraction, proper filesystem extensibility.

#### Core Abstractions

| Component | Description |
|-----------|-------------|
| **Inode** | Represents a file/directory on disk (with operations trait) |
| **Dentry** | Directory entry cache (path → inode mapping) |
| **File** | Open file handle (inode + offset + flags) |
| **Superblock** | Filesystem instance metadata |
| **FileSystem** | Filesystem type (factory for superblocks) |

#### Proposed Trait Design

```rust
pub trait InodeOps: Send + Sync {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>>;
    fn unlink(&self, name: &str) -> Result<()>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<dyn Inode>>;
    fn rmdir(&self, name: &str) -> Result<()>;
    fn symlink(&self, name: &str, target: &str) -> Result<Arc<dyn Inode>>;
    fn readlink(&self) -> Result<String>;
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize>;
    fn truncate(&self, size: u64) -> Result<()>;
    fn stat(&self) -> Result<Stat>;
}

pub trait SuperblockOps: Send + Sync {
    fn root(&self) -> Arc<dyn Inode>;
    fn statfs(&self) -> Result<StatFs>;
}
```

#### Units of Work

- [ ] Define `Inode` trait in `kernel/src/fs/inode.rs`
- [ ] Define `Superblock` trait in `kernel/src/fs/superblock.rs`
- [ ] Create `File` struct in `kernel/src/fs/file.rs`
- [ ] Create `Dentry` cache in `kernel/src/fs/dentry.rs`
- [ ] Implement VFS dispatch layer in `kernel/src/fs/vfs.rs`
- [ ] Refactor `FdType` to use `Arc<File>` instead of per-fs variants

---

### ✅ Phase 14: Filesystem Migration (Completed)

- **Objective**: Migrate existing filesystems to VFS layer.
- **Completed**: tmpfs, initramfs, mount/umount syscalls (TEAM_206)

#### Migrations

| Filesystem | Current | Target |
|------------|---------|--------|
| **tmpfs** | `TmpfsNode` + FdType dispatch | Implements `InodeOps` |
| **initramfs** | CPIO index + FdType dispatch | Implements `InodeOps` (read-only) |
| **FAT32** | `embedded-sdmmc` wrapper | Implements `InodeOps` |
| **ext4** | `ext4-view` wrapper | Implements `InodeOps` (read-only) |

#### Units of Work

- [ ] Wrap tmpfs as `TmpfsInode` implementing `InodeOps`
- [ ] Wrap initramfs as `InitramfsInode` implementing `InodeOps`
- [ ] Remove old `FdType::TmpfsFile`, `FdType::InitramfsFile` variants
- [ ] Update all syscalls to use VFS layer
- [ ] Add mount/umount syscalls

---

### 🚦 Phase 15: Process & System Management

- **Objective**: Full POSIX process lifecycle, signals, and shell pipeline support.
- **Critical for**: Shell job control, multi-process applications, uutils compatibility.

#### Kernel Syscalls Required (Phase 15)

| Category | Syscall | Nr (AArch64) | Purpose |
|----------|---------|--------------|---------|
| **Process** | `fork` / `clone` | 220 | Create child process |
| | `execve` | 221 | Execute new program |
| | `wait4` | 260 | Wait for child termination |
| | `getppid` | 173 | Get parent PID |
| | `exit_group` | 94 | Terminate all threads |
| **Signals** | `rt_sigaction` | 134 | Install signal handler |
| | `rt_sigprocmask` | 135 | Block/unblock signals |
| | `rt_sigreturn` | 139 | Return from signal handler |
| | `kill` | 129 | Send signal to process |
| | `tgkill` | 131 | Send signal to thread |
| **Pipes & FDs** | `pipe2` | 59 | Create pipe pair |
| | `dup` | 23 | Duplicate fd |
| | `dup3` | 24 | Duplicate to specific fd |

#### Utilities

| Utility | Dependencies | Notes |
|---------|--------------|-------|
| `ps` | `/proc` or `sys_info` | List running processes |
| `kill` | `sys_kill` | Send signals |
| `top` | `/proc`, terminal raw mode | Real-time process view |
| `free` | Memory stats syscall | Memory usage |
| `uptime` | `clock_gettime` | System uptime |
| `shutdown` / `reboot` | PSCI / ACPI | Power control |

---

### 📝 Phase 16: Text Editing & Interaction

- **Objective**: Productive text manipulation within the OS.
- **Units of Work**:
  - [ ] **Terminal Raw Mode**: `sys_ioctl` (or similar) to control TTY driver behavior.
  - [ ] **`grep`**: Basic pattern matching.
  - [ ] **`more`** / **`less`**: Paging through long text.
  - [ ] **`vi` (micro)**: A tiny screen-oriented text editor.
    - Buffer management
    - Cursor movement
    - Insert/Normal modes
    - File saving

### 📦 Phase 17: Rust `std` Port & uutils-levbox (The Graduation)

> [!NOTE]
> **Milestone**: Successfully compile and run `uutils-levbox` on LevitateOS.
> 
> This phase represents "graduation" — proving LevitateOS has a fully functional POSIX-like userspace.

- **Objective**: Port Rust's standard library to LevitateOS and run production Rust binaries.
- **Prerequisites**: All syscalls from the gap analysis table must be implemented.

#### Phase 17a: Threading & Synchronization

| Task | Syscall/Feature | Notes |
|------|-----------------|-------|
| `clone` syscall | Full thread creation | CLONE_VM, CLONE_THREAD flags |
| TLS support | `TPIDR_EL0` setup | Per-thread pointer |
| `futex` syscall | Blocking sync | FUTEX_WAIT, FUTEX_WAKE |
| `set_tid_address` | Thread exit notification | For pthread_join |

#### Phase 17b: Memory Management Extension

| Task | Syscall/Feature | Notes |
|------|-----------------|-------|
| `mmap` / `munmap` | Anonymous & file-backed | Required by allocators |
| `mprotect` | Guard pages | Stack overflow protection |
| `mremap` | Resize mappings | Optional, for realloc |

#### Phase 17c: Rust `std` Backend

| Component | Implementation Approach |
|-----------|------------------------|
| **libc layer** | Use [relibc](https://github.com/redox-os/relibc) as reference |
| **std::sys** | Implement `src/sys/pal/unix` for LevitateOS target |
| **Target spec** | Create `aarch64-unknown-levitateos` target |
| **Build toolchain** | Cross-compile std with custom target JSON |

#### Phase 17d: uutils Integration

| Task | Notes |
|------|-------|
| Cross-compile uutils | Using LevitateOS target |
| Run test suite | Validate levbox behavior |
| Integration | Replace busybox utils with uutils |

#### References

- [uutils-levbox](https://github.com/uutils/levbox) — Target project
- [relibc](https://github.com/redox-os/relibc) — Rust libc implementation
- [rust-lang/libc](https://github.com/rust-lang/libc) — FFI bindings reference
- [Redox OS std port](https://gitlab.redox-os.org/redox-os/rust) — Prior art

---

## 🔐 PART III: MULTI-USER SECURITY & AUTHENTICATION (Future)

Once the userspace foundation is solid, we move to secure multi-user support.

### 🛡️ Phase 18: Identity & Authentication

- **Objective**: Identify users and protect resources.
- **Units of Work**:
  - [ ] **User Database**: Implement `/etc/passwd` and `/etc/group` logic.
  - [ ] **Secret Management**: Implement `/etc/shadow` with Argon2 hashing.
  - [ ] **`login`**: The gatekeeper program (replacing direct shell spawn).
  - [ ] **`su`**: Switch User functionality.

### 🔑 Phase 19: Privilege Escalation & Access Control

- **Objective**: Controlled administration access.
- **Units of Work**:
  - [ ] **`doas`**: A minimal, config-based privilege escalation tool (simpler than `sudo`).
  - [ ] **Permission Enforcement**: Kernel-level check of UID/GID against file modes (`rwx`).
  - [ ] **Capabilities**: Fine-grained permissions (e.g., `CAP_NET_ADMIN`) to avoid full root requirements.
  - [ ] **Session Management**: Session IDs and Process Groups (for shell job control).

---

## 🛡️ Phase 20: Advanced Kernel Hardening (Hostile Userspace Model)

- **Objective**: Implement a Zero-Trust security model where the kernel treats all EL0 (Userspace) interactions as potentially malicious.
- **Goal**: Establish LevitateOS as a "Secure-by-Design" kernel using Rust's type system to enforce boundary invariants.

### Tasks & Invariants

- [ ] **Type-Safe User Pointers**: Implement `UserPtr<T>` and `UserSlice<T>` wrappers to replace raw pointers in syscall handlers, enforcing validation before dereference.
- [ ] **Hardware-Backed Isolation (PAN/PXN)**: Enable AArch64 Privileged Access Never (PAN) and Privileged Execute Never (PXN) to prevent lateral kernel corruption from EL0.
- [ ] **Strict TOCTOU Prevention**: Enforce a mandatory Copy-In pattern for all syscall buffers to eliminate Time-of-Check to Time-of-Use vulnerabilities.
- [ ] **Capability-Based Resource Access**: Migrate from enumerable integer File Descriptors to opaque Capability handles to prevent handle-guessing attacks.
- [ ] **Address Space Layout Randomization (ASLR)**: Implement entropy-based randomization for user stack, heap (brk), and executable base addresses.
- [ ] **Syscall Sandboxing**: Implement a per-process syscall filter (e.g., Seccomp-like) to restrict the attack surface for unprivileged processes.
- [ ] **Audit Logs & Integrity**: Implement cryptographic measurement of userspace binaries and kernel-level audit logs for failed syscall validations.

---

## Team Registry Summary

| Phase | Teams | Description |
|-------|-------|-------------|
| 1 | 001-009 | Foundation, Workspace Refactor |
| 2 | 010-017 | Timer, UART, GIC, HAL Hardening |
| 3 | 018-028 | MMU, Higher-Half Kernel, Audit |
| 4 | 029-039 | VirtIO Block, FAT32, Initramfs |
| 5 | 041-055 | Buddy/Slab Allocators, GIC Hardening, FDT Discovery |
| 6 | 056-066 | VirtIO Ecosystem (Net, GPU, Input), Hybrid Boot Spec |
| 7 | 067-071 | Multitasking, Scheduler, Context Switching |
| 8a | 072-079 | Userspace Foundation (EL0, Syscalls, ELF) |
| 8b | 080+ | Interactive Shell & Coreutils |
| 8c | 118+ | Userspace Architecture Refactor |
| 8d | 120+ | Process Management (Init, Spawn) |
| Maintenance | 121-163 | Bug fixes, refactors, architecture improvements |
| 10 | 164+ | `ulib` Userspace Library |
| 11 | 192-199 | Busybox Coreutils (Levbox) |
| 12 | 200+ | VFS Foundation (Prerequisites) |
| 13 | TBD | Core VFS Implementation |
| 14 | TBD | Filesystem Migration |
| 15 | TBD | Process & Signals |
| 16 | TBD | Text Editing & Interaction |
| 17 | TBD | Rust `std` Port & uutils |
| 18-19 | TBD | Multi-User Security |
| 20 | 214 | Advanced Kernel Hardening (Hostile Userspace) |

---

## 📚 Appendix A: External Kernel Reference & Gap Analysis

> **Updated:** 2026-01-06 (TEAM_207)
> 
> **Source:** `.external-kernels/` containing Redox, Theseus, and Tock kernels

### Reference Kernels Overview

| Kernel | Focus | Size | Key Strengths |
|--------|-------|------|---------------|
| [Redox](file:///home/vince/Projects/LevitateOS/.external-kernels/redox-kernel) | Full OS | ~104KB memory.rs | Futex, CoW fork, signals, pipe |
| [Theseus](file:///home/vince/Projects/LevitateOS/.external-kernels/theseus) | Research | 159 modules | Pluggable scheduler, IPC channels |
| [Tock](file:///home/vince/Projects/LevitateOS/.external-kernels/tock) | Embedded | 27 core files | Scheduler trait, capability grants |

### Gap Analysis Summary

#### 🔴 Critical Gaps (Required for `std`/uutils)

| Feature | Redox | Theseus | Tock | LevitateOS | Phase |
|---------|-------|---------|------|------------|-------|
| Futex (WAIT/WAKE) | ✅ | ❌ channels | ❌ | ❌ Missing | 17a |
| mmap/munmap | ✅ Full | ✅ | ❌ | ❌ Missing | 17b |
| clone/fork with CoW | ✅ | ✅ spawn | ❌ | ❌ Missing | 15/17a |
| Signals (SIGCHLD, etc.) | ✅ Full | ✅ 4 types | ❌ upcalls | ❌ Missing | 15 |
| Pipe (for `|`) | ✅ | ✅ | ❌ | ❌ Missing | 15 |
| TLS (TPIDR_EL0) | ✅ | ✅ | ✅ | ❌ Missing | 17a |

#### 🟡 Important Gaps

| Feature | Status | Phase | Reference File |
|---------|--------|-------|----------------|
| dup/dup2/dup3 | ❌ Missing | 15 | `redox/src/syscall/fs.rs` |
| poll/select | ❌ Missing | 16 | `redox/src/event.rs` |
| ioctl | ❌ Missing | 16 | Theseus `tty/` |
| Wait queues | ❌ Missing | 15 | `redox/src/sync/wait_queue.rs` |
| Scheduler policies | 🟡 Simple RR | 17+ | `tock/kernel/src/scheduler.rs` |

#### 🟢 Features LevitateOS Already Has

| Feature | Status | Phase Completed |
|---------|--------|-----------------|
| VFS layer | ✅ | 14 |
| tmpfs (full CRUD) | ✅ | 14 |
| initramfs | ✅ | 4 |
| Mount/Umount | ✅ | 14 |
| waitpid | ✅ | 8d |
| spawn_args | ✅ | 8d |
| symlinkat/linkat | ✅ | 11 |
| clock_gettime/nanosleep | ✅ | 10 |

### Key Reference Files for Implementation

| Task | Reference | File |
|------|-----------|------|
| Futex | Redox | [futex.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/redox-kernel/src/syscall/futex.rs) |
| mmap/CoW | Redox | [memory.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/redox-kernel/src/context/memory.rs) |
| Signals | Redox | [signal.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/redox-kernel/src/context/signal.rs) |
| Pipe | Redox | [pipe.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/redox-kernel/src/scheme/pipe.rs) |
| Scheduler | Tock | [scheduler.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/tock/kernel/src/scheduler.rs) |
| Task/TLS | Theseus | [task/lib.rs](file:///home/vince/Projects/LevitateOS/.external-kernels/theseus/kernel/task/src/lib.rs) |

### Recommended Implementation Order

1. **Phase 15 (Lower effort, high impact)**
   - `pipe2` → Enables shell pipelines
   - `dup`/`dup2` → FD redirection
   - Basic signals (SIGCHLD, SIGKILL, SIGTERM)

2. **Phase 17a (Threading)**
   - `clone` with CLONE_VM
   - TLS via TPIDR_EL0
   - `futex` (WAIT/WAKE)

3. **Phase 17b (Memory)**
   - `mmap`/`munmap` (anonymous first)
   - `mprotect` for guard pages

> [!TIP]
> **Quick win**: Pipe + dup only requires ~500 lines of kernel code but unlocks major shell functionality.

