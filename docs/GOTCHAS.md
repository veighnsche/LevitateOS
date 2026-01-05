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

### 4. GPU Display API Pattern (PARTIAL FIX by TEAM_086)

**Location:** `kernel/src/gpu.rs` - `Display` struct

**Status:** ⚠️ API FIXED, but GPU display still not working

**What TEAM_086 Fixed:**
The `Display` struct was refactored to accept `&mut GpuState` instead of locking internally. This eliminates the internal deadlock:

```rust
// CORRECT PATTERN:
let mut gpu_guard = gpu::GPU.lock();
if let Some(gpu_state) = gpu_guard.as_mut() {
    let mut display = Display::new(gpu_state);
    Text::new("Hello", Point::new(10, 30), style).draw(&mut display).ok();
    gpu_state.flush();  // Safe - still holding the same lock
}
```

**TEAM_089 Update:**
- Added 10Hz timer-based GPU flush in `kernel/src/main.rs`.
- **Status:** Behavior tests pass (golden log match), proving init/flush works.
- **Visuals:** Display may still be blank on some QEMU versions/hosts despite correct driver behavior.
- **Verification:** Always use `cargo xtask gpu-dump` to verify if pixels are being modified in RAM before assuming the driver is failing.
- **Recommendation:** Rely on serial console for active development.

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

## Adding New Gotchas

When you discover a non-obvious issue:
1. Add it to this file with your TEAM_XXX
2. Include: Location, Problem, Symptom, Fix
3. Leave breadcrumbs in the code too
