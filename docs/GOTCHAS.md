# LevitateOS Gotchas & Known Issues

This document captures non-obvious issues that future teams should know about.

---

## Critical Issues

### 1. Boot Hijack Prevents Interactive Mode (TEAM_081)

**Location:** `kernel/src/main.rs:612`

**Problem:** TEAM_073 added code to demo userspace execution that runs immediately and never returns. This prevents the kernel from reaching its main loop where keyboard input is processed.

**Symptom:** System boots, shows messages, then appears frozen. No typing works.

**Fix:**
```rust
// Comment out this line:
// task::process::run_from_initramfs("hello", &archive);
```

---

### 2. Userspace Linker Script Conflict (TEAM_082)

**Location:** `userspace/hello/linker.ld`

**Problem:** Root `.cargo/config.toml` adds `-Tlinker.ld` for all aarch64 builds. This conflicts with userspace's `link.ld`. Cargo merges configs, so both scripts are used, causing "Cannot allocate memory" errors.

**Fix:** Userspace directories need an empty `linker.ld` stub file to satisfy the root config without adding conflicting sections.

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

### 16. Two GPU Crates Exist — BOTH ARE BROKEN (TEAM_109)

**Location:** `levitate-gpu/` and `levitate-drivers-gpu/`

**⚠️ CRITICAL WARNING:**
- `levitate-gpu`: Uses external `virtio-drivers`. **GIVES FALSE POSITIVE TESTS.** Tests pass but GPU display doesn't actually work.
- `levitate-drivers-gpu`: Custom implementation. Times out on commands.

**Why This Matters:**
The reason `levitate-drivers-gpu` was created is because `levitate-gpu` (virtio-drivers) **never actually worked**. It passes tests because the test harness doesn't verify actual display output — it only checks that the driver initializes without crashing.

**DO NOT:**
- ❌ Trust "passing tests" with `levitate-gpu` — they are false positives
- ❌ Fall back to `levitate-gpu` thinking it "works" — it doesn't
- ❌ Consider the GPU "fixed" just because tests pass

**DO:**
- ✅ Verify actual display output in QEMU window
- ✅ Fix `levitate-drivers-gpu` properly
- ✅ Use `cargo xtask gpu-dump` to verify pixel data

**History:** See `.teams/TEAM_108_investigate_gpu_crate_alignment.md` and `.teams/TEAM_109_fix_gpu_driver_no_fallback.md` for full context.

---

## Adding New Gotchas

When you discover a non-obvious issue:
1. Add it to this file with your TEAM_XXX
2. Include: Location, Problem, Symptom, Fix
3. Leave breadcrumbs in the code too
