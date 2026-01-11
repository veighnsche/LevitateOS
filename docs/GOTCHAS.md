# LevitateOS Gotchas & Known Issues

This document captures non-obvious issues that future teams should know about.

---

## Critical Issues

### 1. Boot Hijack Prevents Interactive Mode (TEAM_081) **[RESOLVED]**

**Status:** Fixed by TEAM_120 (Userspace Exec & Init)
**Resolution:** The kernel now boots to a proper `init` process (PID 1) from initramfs, which spawns the shell. The hardcoded hijack has been replaced with a formal process management lifecycle.

---

### Gotcha #2: Userspace Linker Script Conflict (TEAM_082) **[RESOLVED]**

**Status:** Fixed by TEAM_118 (Userspace Refactor)
**Resolution:** Userspace crates now use per-crate `build.rs` to add linker arguments, avoiding global config conflicts. `userspace/` is now a separate workspace.

> **Old Description (for reference):**
> You cannot just add `-Tlink.ld` to `.cargo/config.toml` in the root workspace...
atisfy the root config without adding conflicting sections.

---

### 3. Dual Console Only Works After Stage 3 (TEAM_081)

**Location:** `levitate-hal/src/console.rs`

**Problem:** The GPU terminal callback is registered AFTER GPU initialization. Any `println!` calls BEFORE Stage 3 only go to UART, not GPU.

**Note:** This is by design, but can be confusing when debugging early boot.

---

### 4. GPU Display API Pattern (FIXED by TEAM_099)

**Location:** `kernel/src/gpu.rs`

**Status:** ✅ FIXED

**What TEAM_099 Fixed:**
The legacy `virtio-drivers` and its associated `GpuState` wrapper have been replaced by a custom, integrated driver stack (`levitate-virtio-gpu`). 

- **No more `Display` wrapper**: `VirtioGpu` now implements `DrawTarget` directly.
- **Locking**: Access the global `GPU` static via `IrqSafeLock`.
- **Flushing**: Use `gpu.flush()` after drawing to update the host display.

```rust
// NEW PATTERN:
let mut gpu_guard = gpu::GPU.lock();
if let Some(gpu) = gpu_guard.as_mut() {
    Text::new("Hello", Point::new(10, 30), style).draw(gpu).ok();
    gpu.flush().ok();
}
```

---

### 8. Diverging Functions and `asm!` (TEAM_099)

**Location:** `kernel/src/task/user.rs` (`enter_user_mode`)

**Problem:** Functions marked with `-> !` (diverging) that use `asm!(..., options(noreturn))` can sometimes fail to compile if the Rust compiler doesn't "see" the divergence clearly.

**Fix:** Ensure the `asm!` block is the last expression in the function or append an explicit `loop { core::hint::spin_loop(); }` (marked `#[allow(unreachable_code)]`) to satisfy the type system.

---

### 9. Console Mirroring Setup (TEAM_099)

**Location:** `kernel/src/terminal.rs`

**Note:** Dual console output (UART + GPU) is enabled by `levitate_hal::console::set_secondary_output(terminal::write_str)`. This happens late in boot (Stage 3). If you don't see GPU output, check if the GPU initialization succeeded in the serial logs.

---

### 10. Consolidate Layout Constants (TEAM_099)

**Location:** `kernel/src/task/user_mm.rs`

**Rule:** Always keep address space layout constants (e.g., `STACK_TOP`, `USER_SPACE_END`) in `user_mm.rs`. Other modules like `user.rs` or `process.rs` should import them from there to avoid "unused constant" warnings and maintain a single source of truth.

### 5. Kernel Does Not Recompile When Initramfs Changes (TEAM_090)

**Problem:** Changing the `initramfs.cpio` file does not trigger a kernel rebuild, even though the kernel embeds it (or uses it).
**Symptom:** You update `userspace/shell`, rebuild initramfs, run `./run.sh`, but the kernel runs the *old* shell binary.
**Fix:** Force a rebuild of the kernel package:
```bash
cargo clean -p levitate-kernel
cargo build --release
```

---

### 6. IrqSafeLock is NOT Re-entrant (TEAM_083)

---

 
### 5. IrqSafeLock is NOT Re-entrant (TEAM_083)

**Location:** `levitate-hal/src/lib.rs`

**Problem:** `IrqSafeLock` wraps a `Spinlock` and disables interrupts. It does NOT support re-entrant locking. If you hold the lock and try to lock again from the same context, you will deadlock.

**Affects:**
- GPU operations (Display + flush)
- Any nested lock scenarios
- Timer/interrupt handlers that try to print (if they use the same locks)

**Pattern to avoid:**
```rust
let guard1 = SOME_LOCK.lock();
let guard2 = SOME_LOCK.lock();  // DEADLOCK!
```

---

### 7. Recursive Deadlocks in IRQ Serial Output (TEAM_092)

**Location:** `kernel/src/gpu.rs:heartbeat()` and `kernel/src/main.rs:TimerHandler`

**Problem:** Using `serial_println!` (which uses `WRITER.lock()`) inside a timer interrupt can cause a recursive deadlock if the shell or kernel main loop already holds the `WRITER` lock during a print operation.

**Fix:** Use `WRITER.try_lock()` in IRQ handlers and telemetry hooks. If the lock is held, skip the output or use a non-blocking queue.

---

## Build Gotchas

### Userspace Binaries Need Separate Build

Userspace binaries are NOT part of the main workspace. Build them separately:

```bash
cd userspace/hello
cargo build --release
```

Then copy to initramfs:
```bash
cp target/aarch64-unknown-none/release/hello ../../initrd_root/
cd ../..
./scripts/make_initramfs.sh
```

---

## Testing Gotchas

### Golden Tests Are Stale

The behavior tests in `tests/golden_boot.txt` reflect the OLD boot sequence before TEAM_073's userspace changes. They will fail until updated.

---

## Runtime Gotchas

### No Visual Echo in Userspace

The shell code reads from stdin but doesn't echo characters back. Users type "blind" - they won't see what they're typing until the command executes.

### Keyboard Input Sources

There are TWO keyboard input sources:
1. **UART** - Serial console input via `console::read_byte()`
2. **VirtIO Keyboard** - GPU keyboard via `input::read_char()`

Both need to be checked for full input coverage.

---

### 6. Serial Console is the Working Interface (TEAM_087)

**Status:** This is the current state, not a bug

**Working Interface:**
```bash
cargo xtask run
# Type directly in THIS terminal - that's the serial console
```

**NOT Working:**
- QEMU graphical window (shows "Display output is not active")
- Mouse/keyboard input in QEMU window

**Why:** VirtIO GPU display scanout is not configured. See issue #4 above.

**What Works via Serial:**
- Full boot messages
- Interactive prompt (`# `)
- Keyboard input (VirtIO keyboard echoes to serial)
- All kernel functionality

---

---

## VirtIO Gotchas

### 11. VirtQueue Used Ring Must Be 4-Byte Aligned (TEAM_109)

**Location:** `levitate-virtio/src/queue.rs`

**Problem:** VirtIO 1.1 spec requires the used ring to be 4-byte aligned. If the available ring ends at a 2-byte boundary (which it often does), padding is required.

**Symptom:** Device is notified but never responds. No error, just timeout.

**Fix:**
```rust
// After avail_ring, add padding before used_flags:
avail_ring: [u16; SIZE],
used_event: u16,
_padding: u16,  // TEAM_109: Ensure 4-byte alignment for used ring
used_flags: u16,
```

---

### 12. Command/Response Buffers Must Be DMA-Allocated (TEAM_109)

**Location:** `levitate-drivers-gpu/src/device.rs`

**Problem:** VirtIO devices access command/response buffers via DMA. Regular heap memory (`Vec`, `Box`) may not be DMA-accessible.

**Symptom:** Commands time out. Device sees garbage or wrong addresses.

**Fix:** Allocate persistent DMA buffers at device init:
```rust
let (cmd_buf_paddr, cmd_buf_ptr) = H::dma_alloc(1, BufferDirection::DriverToDevice);
let (resp_buf_paddr, resp_buf_ptr) = H::dma_alloc(1, BufferDirection::DeviceToDriver);
```

---

### 13. VirtQueue Writes Need Volatile Operations (TEAM_109)

**Location:** `levitate-virtio/src/queue.rs`

**Problem:** The compiler may optimize away writes to struct fields that are read by the device via DMA. Regular Rust assignments are not guaranteed to generate stores.

**Symptom:** Device doesn't see descriptor updates, avail_idx changes, etc.

**Fix:** Use `write_volatile` for all device-visible memory:
```rust
unsafe {
    core::ptr::write_volatile(&mut (*desc_ptr).addr, phys_addr);
    core::ptr::write_volatile(&mut (*desc_ptr).len, length);
    core::ptr::write_volatile(&mut self.avail_idx, new_idx);
}
```

---

### 14. ARM DSB Required for Device Visibility (TEAM_109)

**Location:** `levitate-virtio/src/queue.rs`

**Problem:** On ARM, `fence(Ordering::SeqCst)` orders CPU memory accesses but may not ensure writes are visible to devices accessing via DMA. DSB (Data Synchronization Barrier) ensures completion.

**Symptom:** Device misses updates even with volatile writes and fences.

**Fix:** Add DSB after critical writes:
```rust
#[cfg(target_arch = "aarch64")]
unsafe {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}
```

---

### 15. VirtQueue Architecture: Embedded vs Pointer-Based (TEAM_109)

**Location:** `levitate-virtio/src/queue.rs` vs `virtio-drivers/src/queue.rs`

**Problem:** Our VirtQueue embeds all data in one struct. virtio-drivers allocates separate DMA regions and stores pointers. This architectural difference may cause subtle incompatibilities.

**Our approach:**
```rust
struct VirtQueue<SIZE> {
    descriptors: [Descriptor; SIZE],  // Embedded
    avail_flags: u16,                 // Embedded
    // ...
}
```

**virtio-drivers approach:**
```rust
struct VirtQueue<H, SIZE> {
    desc: NonNull<[Descriptor]>,      // Pointer to DMA
    avail: NonNull<AvailRing>,        // Pointer to DMA
    used: NonNull<UsedRing>,          // Pointer to DMA
    desc_shadow: [Descriptor; SIZE],  // Local shadow copy
}
```

**Note:** If incremental fixes don't work, architectural refactoring may be required.

---

### 16. GPU Display Still Broken - Use VNC to Verify (TEAM_111)

**Location:** `levitate-drivers-gpu/`

**⚠️ CRITICAL: GPU GIVES FALSE POSITIVE TESTS**

Serial output says "GPU initialized successfully" but QEMU window shows **"Display output is not active"**.

**Current Status (2026-01-05):**
- ❌ QEMU display shows nothing
- ✅ Serial console works (boots to shell prompt)
- ⚠️ Tests pass but are misleading

**How to Verify GPU State:**
```bash
# Start QEMU with VNC
cargo xtask run-vnc

# Open browser to http://localhost:6080/vnc.html
# Click "Connect"
# 
# "Display output is not active" = BROKEN
# Terminal text visible = WORKING
```

**Root Cause:** VirtIO GPU SET_SCANOUT command not working properly.

**Fix Required:** Make `levitate-drivers-gpu` correctly send:
1. SET_SCANOUT - link framebuffer to display
2. TRANSFER_TO_HOST_2D - copy pixels
3. RESOURCE_FLUSH - refresh display

**References:**
- `docs/handoffs/TEAM_111_gpu_display_fix_handoff.md` - Full fix instructions
- `.agent/workflows/fix-gpu-display.md` - Debug workflow
- `.teams/TEAM_111_investigate_desired_behaviors_and_qemu_vnc.md` - Investigation

---

### 17. PCI GPU Works But Terminal Rendering Is Broken (TEAM_114)

**Location:** `levitate-terminal/src/lib.rs`, `kernel/src/terminal.rs`

**Status:** 🔴 OPEN

**What Works:**
- ✅ VirtIO GPU via PCI transport initializes successfully
- ✅ Framebuffer is mapped and writable
- ✅ Purple test pattern is visible
- ✅ Text can be drawn to framebuffer

**What's Broken:**
- ❌ Terminal renders as raw text with black per-line backgrounds
- ❌ No proper terminal grid/buffer
- ❌ Not a real terminal UI - just println output with black boxes

**Screenshot Evidence:** Each boot message appears as a black rectangle on purple background. This is NOT a terminal - it's just text with per-character background fills.

**Root Cause (suspected):**
The terminal implementation (`levitate-terminal`) may be:
1. Just printing text character-by-character with background fills
2. Not maintaining a proper terminal buffer/grid
3. Not clearing/managing a terminal viewport area

**What Future Teams Should Do:**
1. Review `levitate-terminal/src/lib.rs` for how text is rendered
2. Implement a proper terminal with:
   - Fixed terminal area (black background rectangle)
   - Character grid (rows × columns)
   - Scrolling support
   - Cursor position tracking
3. The GPU/PCI layer is working - this is a terminal layer issue

**References:**
- `.teams/TEAM_114_review_plan_virtio_pci.md`
- `levitate-gpu/` - GPU wrapper (working)
- `levitate-pci/` - PCI subsystem (working)

---

### 18. Recursive Deadlock: Holding Kernel Locks during Userspace Jump (TEAM_120)

**Location:** `kernel/src/main.rs:run_from_initramfs`

**Problem:** If you hold a `Spinlock` (e.g., `INITRAMFS.lock()`) while starting the first user process or performing a context switch that doesn't return, you create a permanent lock-out. 
If that process later triggers a syscall that needs the same lock, it will deadlock. Even worse, if you context switch *away* without releasing, no other task can ever acquire it.

**Symptom:** The first process starts fine, but its first syscall (e.g., `spawn`) hangs the system.

**Fix:** Always release the lock or copy/clone the data out of the lock before transitioning to userspace or yielding.
```rust
// TEAM_120: Correct pattern
let archive = {
    let lock = crate::fs::INITRAMFS.lock();
    *lock.as_ref().unwrap() // Copy/Clone here
}; // Lock released here
task::process::run_from_initramfs("init", &archive);
```

---

### 19. Interactive Input Starvation via Syscall Masking (TEAM_149)

**Location:** `kernel/src/syscall.rs` (`sys_read`)

**Problem:** Syscalls (`svc`) automatically mask interrupts (PSTATE.I=1). If a syscall handler loops tightly (e.g., polling input) and yields to another task that is *also* in a syscall (e.g., `init` calling `sys_yield`), the CPU effectively spins with interrupts permanently disabled.

**Symptom:** Interactive shell is unresponsive. Typing produces no output. `run-term.sh` appears hung.

**Fix:** You must explicitly unmask interrupts briefly within any long-running syscall loop to allow device ISRs to fire:
```rust
loop {
    poll();
    // Allow ISRs (UART, VirtIO) to run
    unsafe { levitate_hal::interrupts::enable(); }
    let _ = levitate_hal::interrupts::disable();
    yield_now();
}
```

---

### 20. Unsafe Preemption in IRQ Handlers (TEAM_149)

**Location:** `kernel/src/init.rs` (`TimerHandler`)

**Problem:** Calling `yield_now()` from an IRQ handler is unsafe because the interrupted context's state (stack, registers, PSTATE) is saved on the stack, but switching tasks replaces the stack pointer. If the scheduler switches back, it must restore exact state. This is extremely fragile without dedicated IRQ stacks and careful PSTATE management.

**Symptom:** Random hangs or corruption during interactive sessions.

**Rule:** **IRQ Handlers Should NOT Yield.** Use `scheduler::SCHEDULER.schedule()` only at the *end* of the exception handler (in assembly return path), or rely on cooperative multitasking until full preemption is implemented safely.

---

## VFS Gotchas (TEAM_201, TEAM_202)

### 21. Dentry Cache vs Filesystem State (TEAM_202)

**Location:** `kernel/src/fs/vfs/dentry.rs`

**Problem:** The dentry cache is a **cache**, not the source of truth. If you modify filesystem state directly (e.g., create a file in tmpfs), you must also update the dentry cache.

**Symptom:** File exists in filesystem but `vfs_open` returns NotFound. Or file was deleted but still appears.

**Fix:** Always update both:
```rust
// After creating in filesystem:
parent_dentry.add_child(new_dentry);

// After deleting from filesystem:
parent_dentry.remove_child(name);
dcache().invalidate(path);
```

---

### 22. Kernel and Userspace Stat Must Match (TEAM_201)

**Location:** `kernel/src/syscall/mod.rs` and `userspace/libsyscall/src/lib.rs`

**Problem:** Kernel `Stat` and userspace `Stat` structs must be identical (`#[repr(C)]`). If they differ in size or field order, fstat will corrupt memory.

**Symptom:** Garbage values in stat fields. Random crashes in userspace.

**Fix:** When adding fields to Stat, add to BOTH files with identical layout:
- `kernel/src/syscall/mod.rs` — kernel Stat
- `userspace/libsyscall/src/lib.rs` — userspace Stat

---

### 23. InodeOps Must Be Static References (TEAM_202)

**Location:** `kernel/src/fs/vfs/inode.rs`

**Problem:** `Inode.ops` is `&'static dyn InodeOps` because Inodes have unbounded lifetime. You cannot pass owned or borrowed InodeOps.

**Symptom:** Compile error about lifetime requirements.

**Fix:** Create a static instance:
```rust
// Good: static instance
static TMPFS_INODE_OPS: TmpfsInodeOps = TmpfsInodeOps;
let inode = Inode::new(..., &TMPFS_INODE_OPS, ...);

// Bad: owned instance (won't compile)
let inode = Inode::new(..., &TmpfsInodeOps {}, ...);
```

---

### 24. Weak References for Parent Pointers (TEAM_202)

**Location:** `kernel/src/fs/vfs/dentry.rs`, `kernel/src/fs/vfs/inode.rs`

**Problem:** Parent pointers (dentry→parent, inode→superblock) must use `Weak<T>` to avoid reference cycles that prevent deallocation.

**Symptom:** Memory leaks. Objects never freed.

**Pattern:**
```rust
pub struct Dentry {
    parent: Option<Weak<Dentry>>,  // Weak!
}

pub struct Inode {
    sb: Weak<dyn Superblock>,  // Weak!
}
```

---

### 25. FdType Migration Order Matters (TEAM_202)

**Location:** `kernel/src/task/fd_table.rs`

**Problem:** `FdType` enum currently has per-filesystem variants. Changing to `FdType::File(Arc<vfs::File>)` requires ALL filesystems to implement VFS first.

**Fix Order:**
1. Implement InodeOps for tmpfs
2. Implement InodeOps for initramfs
3. THEN change FdType
4. THEN update syscalls

**Don't:** Change FdType before filesystems are migrated - syscalls will break.

---

### 26. VFS Boot Initialization Order (TEAM_202)

**Location:** `kernel/src/init.rs` (future)

**Problem:** VFS components must be initialized in correct order during boot.

**Required Order:**
1. Mount table init (`fs::mount::init()`)
2. Initramfs superblock creation
3. Root dentry creation with initramfs root inode
4. Set dcache root
5. Mount tmpfs at /tmp

**Symptom:** "NotFound" errors for `/` or `/tmp` paths.

---

### 27. Multi-Architecture Build Defaults (TEAM_255)

**Location:** `xtask/`

**Problem:** `xtask` commands (build, run, test) default to `aarch64`. If you are working on `x86_64`, you MUST specify the architecture explicitly.

**Symptom:** You run `cargo xtask run` expecting x86 but see ARM boot logs or errors.

**Fix:** Use the `--arch` flag:
```bash
cargo xtask build --arch x86_64
cargo xtask run --arch x86_64
```

---

### 28. Platform-Specific Syscall and Stat Layouts (TEAM_255)

**Location:** `kernel/src/arch/`

**Gotcha:** `SyscallNumber`, `Stat`, and `Termios` are NOT generic. They are defined in architecture-specific modules because their numeric values and struct layouts must match the target platform's ABI (e.g., Linux AArch64 vs Linux x86_64).

**Pattern:** Never define these in generic modules like `kernel/src/syscall/mod.rs`. Always import them from `crate::arch`.

---

### 29. HAL Trait Implementation Requirements (TEAM_255)

**Location:** `crates/hal/src/traits.rs`

**Gotcha:** When adding support for a new architecture, you must implement the `InterruptController` and `MmuInterface` traits. The kernel depends on `los_hal::active_interrupt_controller()` which uses conditional compilation to return the correct implementation. Failure to provide an implementation for a new target will result in `unimplemented!` panics at runtime.

---

### 30. Log Formatting for Behavior Tests (TEAM_272)

**Location:** `kernel/src/logger.rs`, `kernel/src/main.rs`

**Problem:** Behavior tests (golden boot logs) are extremely sensitive to output format. 

**Gotchas:**
- **External Crates:** Some crates (e.g., `virtio_drivers`) may log noisy initialization messages that aren't in the golden file. Filter these in `logger.rs`.
- **Level Prefixes:** Golden logs usually don't include `[INFO]`, `[DEBUG]`, etc. The logger must strip these prefixes.
- **Verbose Feature:** Behavior tests often require `Trace` level logging. Ensure that `#[cfg(feature = "verbose")]` enables the correct level in `main.rs`.

**Fix Pattern (in `logger.rs`):**
```rust
fn log(&self, record: &Record) {
    let target = record.metadata().target();
    if target.starts_with("virtio_drivers") { return; } // Filter noisy crate
    println!("{}", record.args()); // No [LEVEL] prefix
}
```

---

### 31. Duplicate Boot Output (TEAM_272)

**Location:** `kernel/src/main.rs`, `kernel/src/init.rs`

**Problem:** Printing the same information (e.g., boot registers) in multiple boot stages will cause behavior test failures if the golden file only expects it once.

**Rule:** Ensure diagnostic information is printed exactly once in the boot sequence. If a function like `print_boot_regs()` is moved or called in multiple stages, audit the call sites to prevent duplicates.

---

### 32. AArch64 Boot Code Cannot Use Higher-Half Symbols Directly (TEAM_422)

**Location:** `crates/kernel/arch/aarch64/src/asm/boot.S`, `crates/kernel/arch/aarch64/src/boot.rs`

**Problem:** AArch64 boot code runs at physical address 0x40080000, but the kernel is linked at higher-half virtual addresses (0xFFFF_8000_0000_0000+). The `adrp` instruction has a ±4GB range limit, so boot code CANNOT directly access higher-half symbols.

**Symptom:** Linker error: `relocation truncated to fit: R_AARCH64_ADR_PREL_PG_HI21`

**Root Cause:**
- Boot assembly uses `adrp` which is PC-relative with ±4GB range
- Distance from physical 0x40080000 to virtual 0xFFFF_8000... is >4GB
- Rust code in higher-half also can't reach physical-address symbols

**Solution - Two-Copy Pattern:**
```rust
// 1. Physical-address symbol (written by boot.S)
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.boot")]
pub static mut BOOT_DATA_PHYS: u64 = 0;

// 2. Higher-half symbol (read by Rust code)
static mut BOOT_DATA: u64 = 0;

// 3. Copy function (called after identity mapping is active)
pub unsafe fn copy_boot_data() {
    BOOT_DATA = core::ptr::read_volatile(core::ptr::addr_of!(BOOT_DATA_PHYS));
}
```

**Linker Script Requirements:**
```ld
/* Before higher-half jump */
.bss.boot (NOLOAD) : {
    *(.bss.boot)
}

/* Page tables for early MMU */
.boot_page_tables (NOLOAD) : {
    __boot_l0_ttbr0 = .; . = . + 4096;
    __boot_l0_ttbr1 = .; . = . + 4096;
    /* ... */
}

/* THEN jump to higher-half */
. += 0xFFFF800000000000;
```

**Boot Assembly Must:**
1. Save boot registers to `.bss.boot` symbols (reachable by `adrp`)
2. Set up early page tables with identity + higher-half mapping
3. Enable MMU
4. Jump to Rust entry point at higher-half address

**References:**
- `crates/kernel/arch/aarch64/src/asm/boot.S` - Complete boot sequence
- `crates/kernel/levitate/src/arch/aarch64/linker.ld` - Memory layout

---

### 33. Kernel Modular Crate Dependencies Must Avoid Cycles (TEAM_422)

**Location:** `crates/kernel/` workspace

**Problem:** When refactoring the kernel into separate crates, circular dependencies are easy to create. For example: `los_sched` needs types from `los_syscall`, but `los_syscall` needs the scheduler.

**Symptom:** Cargo error: `cyclic package dependency`

**Solution - Shared Types Crate:**
```
los_types/      ← Shared types (SyscallFrame, Pid, etc.)
   ↑
   ├── los_sched    (imports los_types)
   ├── los_syscall  (imports los_types)
   └── los_mm       (imports los_types)
```

**Pattern:**
1. Extract shared types/traits to a separate crate (`los_types`)
2. Make it dependency-free (only depends on `core`)
3. All crates that need shared types import from `los_types`

**Common Shared Types:**
- `SyscallFrame` - Register state for syscall dispatch
- `Pid`, `Tid` - Process/thread identifiers
- `SyscallResult` - Standard syscall return type
- Architecture-specific constants

---

### 34. Architecture Crates Use Extern Callbacks for Integration (TEAM_422)

**Location:** `crates/kernel/arch/*/src/lib.rs`

**Problem:** Architecture crates (`los_arch_aarch64`, `los_arch_x86_64`) need to call into kernel code (e.g., syscall dispatch), but can't depend on the kernel binary.

**Solution - Extern Function Pattern:**
```rust
// In arch crate (los_arch_aarch64/src/exceptions.rs)
unsafe extern "Rust" {
    fn syscall_dispatch(frame: &mut SyscallFrame);
}

// Called from exception handler
pub fn handle_svc(frame: &mut SyscallFrame) {
    unsafe { syscall_dispatch(frame); }
}
```

```rust
// In kernel binary (levitate/src/main.rs)
#[unsafe(no_mangle)]
fn syscall_dispatch(frame: &mut crate::arch::SyscallFrame) {
    los_syscall::syscall_dispatch(frame);
}
```

**Why This Works:**
- Arch crate declares extern function (no implementation)
- Kernel binary provides implementation with `#[no_mangle]`
- Linker resolves at link time
- No circular dependency

**Required Callbacks:**
- `syscall_dispatch` - Called from syscall exception
- `handle_user_exception` - Called for user-mode faults (AArch64)
- `handle_irq_dispatch` - Called for IRQ handling (AArch64)
- `check_and_deliver_signals` - Called before return to userspace
