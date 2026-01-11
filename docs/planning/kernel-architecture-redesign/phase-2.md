# Phase 2: Structural Extraction

## Target Design

### New Workspace Layout

```
levitate-kernel/
├── Cargo.toml                    # Workspace root (existing, modified)
├── rust-toolchain.toml           # (existing)
├── .cargo/config.toml            # (existing)
│
├── kernel/                       # NEW: Thin kernel binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Entry point, panic handler
│       └── init.rs               # Boot orchestration (< 200 lines)
│
├── arch/                         # NEW: Architecture crates
│   ├── aarch64/
│   │   ├── Cargo.toml
│   │   ├── linker.ld             # (moved from src/arch/)
│   │   └── src/
│   │       ├── lib.rs            # Platform trait impl
│   │       ├── boot.rs           # Assembly entry
│   │       ├── exceptions.rs     # Exception vectors
│   │       ├── syscall.rs        # ABI (registers, numbers)
│   │       ├── context.rs        # Task context switch
│   │       ├── mmu.rs            # Page table manipulation
│   │       └── time.rs           # Timer handling
│   └── x86_64/
│       └── (same structure)
│
├── mm/                           # NEW: Memory management crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── phys.rs               # Physical frame allocator
│       ├── heap.rs               # Kernel heap
│       ├── vma.rs                # Virtual memory areas
│       ├── user.rs               # User address space (split from 709-line file)
│       ├── cow.rs                # Copy-on-write (extracted from user.rs)
│       └── mapping.rs            # Page mapping helpers
│
├── sched/                        # NEW: Scheduler crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── process.rs            # Process struct
│       ├── thread.rs             # Thread struct
│       ├── task.rs               # Task (unified view)
│       ├── scheduler.rs          # Run queue, pick_next
│       ├── wait.rs               # Wait queues
│       └── fd_table.rs           # File descriptor table
│
├── vfs/                          # NEW: Virtual filesystem crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── inode.rs              # (from fs/vfs/)
│       ├── dentry.rs
│       ├── file.rs
│       ├── mount.rs
│       ├── path.rs               # (split from 528-line file)
│       ├── resolve.rs            # Path resolution (extracted)
│       ├── ops.rs
│       └── error.rs
│
├── fs/                           # RESTRUCTURED: Filesystem implementations
│   ├── tmpfs/
│   │   ├── Cargo.toml
│   │   └── src/...
│   ├── devfs/                    # NEW
│   ├── procfs/                   # NEW (extract from scattered code)
│   └── initramfs/
│
├── syscall/                      # NEW: Syscall dispatch crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # dispatch(), SyscallResult
│       ├── io.rs                 # read, write, ioctl
│       ├── file.rs               # open, close, stat
│       ├── dir.rs                # getdents, mkdir
│       ├── process.rs            # fork, exec, exit
│       ├── thread.rs             # clone, futex
│       ├── mm.rs                 # mmap, brk
│       └── misc.rs               # uname, getpid
│
├── drivers/                      # RESTRUCTURED
│   ├── console/                  # Extract from scattered code
│   ├── tty/                      # (from fs/tty/)
│   ├── virtio/
│   │   ├── transport/
│   │   ├── blk/
│   │   ├── gpu/
│   │   ├── input/
│   │   └── net/
│   └── pci/                      # (existing los_pci)
│
├── hal/                          # EXISTING (los_hal) - SLIMMED
│   └── src/
│       ├── lib.rs                # Platform trait only
│       ├── allocator/            # (keep buddy/slab)
│       └── (remove arch-specific code - moved to arch/)
│
├── utils/                        # EXISTING (los_utils)
├── error/                        # EXISTING (los_error)
├── term/                         # EXISTING (los_term)
└── gpu/                          # EXISTING (los_gpu)
```

### Dependency Graph (Target)

```
                    kernel (binary)
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    arch/aarch64    arch/x86_64     drivers/*
         │               │               │
         └───────┬───────┘               │
                 │                       │
         ┌───────┴───────┬───────┬───────┘
         │               │       │
         ▼               ▼       ▼
        mm            sched    vfs
         │               │       │
         └───────┬───────┴───────┘
                 │
                 ▼
             syscall
                 │
         ┌───────┴───────┐
         ▼               ▼
       hal            utils
         │
         ▼
       error
```

### Extraction Strategy

**Order of Extraction** (each step must compile):

1. **Extract `error/`** (already done - los_error exists)
2. **Extract `utils/`** (already done - los_utils exists)
3. **Extract `mm/`** - Memory management first (foundation)
4. **Extract `sched/`** - Scheduler depends on mm
5. **Extract `vfs/`** - VFS depends on sched (for fd_table)
6. **Extract `syscall/`** - Syscalls depend on all above
7. **Extract `arch/`** - Architecture as separate crates
8. **Slim `kernel/`** - Thin binary that wires everything together

### Coexistence Strategy

During migration, both old and new code paths exist:
```rust
// Temporary: re-export from new location
pub use mm::user::validate_user_buffer;  // new
// pub use crate::memory::user::validate_user_buffer;  // old (removed)
```

**Rule: No compatibility shims** - When moving code:
1. Move the function/struct to new location
2. Update ALL call sites immediately
3. Delete the old location
4. Compile and test

### Module Ownership Rules (Rule 7)

Each new crate must follow:

1. **Private by default**: All fields private, expose via methods
2. **No deep imports**: `use mm::PhysFrame`, not `use mm::phys::frame::PhysFrame`
3. **Own state**: Each module owns its static data, no global god objects
4. **File limits**: < 500 lines ideal, < 1000 max
5. **Single responsibility**: One concern per module

### Platform Trait Design

```rust
// hal/src/lib.rs
pub trait Platform: Send + Sync {
    type PageTable: PageTableOps;
    type Context: ContextOps;
    type Timer: TimerOps;

    fn page_size() -> usize;
    fn kernel_base() -> usize;
    fn init_interrupts();
    fn enable_interrupts();
    fn disable_interrupts() -> bool;  // returns previous state
}

// arch/aarch64/src/lib.rs
pub struct Aarch64;
impl Platform for Aarch64 { ... }

// arch/x86_64/src/lib.rs
pub struct X86_64;
impl Platform for X86_64 { ... }
```

### File Splitting Plan

**`memory/user.rs` (709 lines) → mm/src/**:
- `user.rs` (< 300): Core user memory operations
- `cow.rs` (< 200): Copy-on-write handling
- `mapping.rs` (< 200): Page mapping utilities

**`syscall/fs/fd.rs` (638 lines) → syscall/src/**:
- `file.rs` (< 300): open, close, stat
- `io.rs` (< 200): read, write, lseek
- `dup.rs` (< 150): dup, dup2, fcntl

**`init.rs` (570 lines) → kernel/src/**:
- `init.rs` (< 200): Boot orchestration
- Driver init moved to `drivers/*/init.rs`
- Memory init moved to `mm/src/init.rs`

**`arch/*/mod.rs` (~580 lines) → arch/*/src/**:
- `lib.rs` (< 100): Platform trait impl
- `boot.rs` (< 200): Boot sequence
- `mmu.rs` (< 200): Page table ops
- `exceptions.rs` (< 200): Exception handling

### New Crate Templates

**mm/Cargo.toml**:
```toml
[package]
name = "los_mm"
version = "0.1.0"
edition = "2024"

[dependencies]
los_hal = { path = "../hal" }
los_error = { path = "../error" }
log = "0.4"
bitflags = { workspace = true }
```

**sched/Cargo.toml**:
```toml
[package]
name = "los_sched"
version = "0.1.0"
edition = "2024"

[dependencies]
los_mm = { path = "../mm" }
los_hal = { path = "../hal" }
los_error = { path = "../error" }
log = "0.4"
```
