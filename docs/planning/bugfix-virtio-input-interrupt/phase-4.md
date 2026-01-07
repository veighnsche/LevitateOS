# Phase 4: Implementation

**Status**: 🔲 Ready for Execution

## Implementation Overview

| Step | File | Changes | Complexity |
|------|------|---------|------------|
| 1 | `kernel/src/virtio.rs` | Track MMIO slot index, pass to `input::init()` | Low |
| 2 | `crates/hal/src/gic.rs` | Add `VirtioInput(u32)` variant to `IrqId` | Low |
| 3 | `kernel/src/input.rs` | Add slot tracking, IRQ computation, interrupt handler | Medium |
| 4 | `kernel/src/input.rs` | Add Ctrl+C detection with immediate signaling | Low |

---

## Step 1: Track MMIO Slot Index in VirtIO Discovery

### File: `kernel/src/virtio.rs`

**Current code** (lines 54-60):
```rust
virtio_drivers::transport::DeviceType::Input => {
    crate::input::init(transport);
}
```

**Modified code**:
```rust
virtio_drivers::transport::DeviceType::Input => {
    // TEAM_241: Pass MMIO slot index for IRQ computation
    crate::input::init(transport, i);
}
```

**Rationale**: The slot index `i` is already available in the loop. We pass it to `input::init()` so it can compute the correct IRQ number.

---

## Step 2: Add VirtioInput IRQ Variant to GIC

### File: `crates/hal/src/gic.rs`

**Location**: `IrqId` enum (line ~143)

**Current code**:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IrqId {
    /// Virtual Timer (PPI, IRQ 27)
    VirtualTimer = 0,
    /// PL011 UART (SPI, IRQ 33)
    Uart = 1,
    // Future: VirtioGpu, VirtioInput, VirtioBlk, VirtioNet
}
```

**Modified code**:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqId {
    /// Virtual Timer (PPI, IRQ 27)
    VirtualTimer,
    /// PL011 UART (SPI, IRQ 33)
    Uart,
    /// VirtIO Input device (SPI, IRQ = 48 + slot_index)
    /// TEAM_241: Added for async Ctrl+C detection
    VirtioInput(u32),
}
```

> **Note**: Remove `#[repr(u8)]` since `VirtioInput` now carries data.

**Location**: `IrqId::irq_number()` method (line ~157)

**Current code**:
```rust
pub const fn irq_number(self) -> u32 {
    match self {
        IrqId::VirtualTimer => 27,
        IrqId::Uart => 33,
    }
}
```

**Modified code**:
```rust
pub fn irq_number(self) -> u32 {
    match self {
        IrqId::VirtualTimer => 27,
        IrqId::Uart => 33,
        // TEAM_241: QEMU virt assigns VirtIO MMIO IRQs sequentially from base 48
        IrqId::VirtioInput(slot) => 48 + slot,
    }
}
```

> **Note**: Remove `const` from function signature since we now have runtime computation.

**Location**: `IrqId::from_irq_number()` method (line ~166)

**Current code**:
```rust
pub fn from_irq_number(irq: u32) -> Option<Self> {
    match irq {
        27 => Some(IrqId::VirtualTimer),
        33 => Some(IrqId::Uart),
        _ => None,
    }
}
```

**Modified code**:
```rust
pub fn from_irq_number(irq: u32) -> Option<Self> {
    match irq {
        27 => Some(IrqId::VirtualTimer),
        33 => Some(IrqId::Uart),
        // TEAM_241: VirtIO MMIO IRQs range from 48 to 48+31
        48..=79 => Some(IrqId::VirtioInput(irq - 48)),
        _ => None,
    }
}
```

**Location**: Handler registration (line ~192)

The current `register_handler` uses `irq as usize` as index. We need a different approach for `VirtioInput` since it has variable slots.

**Current code**:
```rust
pub fn register_handler(irq: IrqId, handler: &'static dyn InterruptHandler) {
    let idx = irq as usize;
    handler.on_register(irq.irq_number());
    unsafe {
        HANDLERS[idx] = Some(handler);
    }
}
```

**Modified code**:
```rust
/// Register a handler for an IRQ.
///
/// # Safety
/// Must be called before interrupts are enabled. Not thread-safe.
pub fn register_handler(irq: IrqId, handler: &'static dyn InterruptHandler) {
    // TEAM_241: Map IrqId to handler table index
    let idx = match irq {
        IrqId::VirtualTimer => 0,
        IrqId::Uart => 1,
        IrqId::VirtioInput(slot) => 2 + slot as usize, // slots 0-31 map to indices 2-33
    };
    
    if idx >= MAX_HANDLERS {
        return; // Silently fail if out of bounds (shouldn't happen)
    }
    
    handler.on_register(irq.irq_number());
    unsafe {
        HANDLERS[idx] = Some(handler);
    }
}
```

**Location**: Dispatch function (line ~203)

**Modified code**:
```rust
pub fn dispatch(irq_num: u32) -> bool {
    if let Some(irq_id) = IrqId::from_irq_number(irq_num) {
        // TEAM_241: Map IrqId to handler table index (must match register_handler)
        let idx = match irq_id {
            IrqId::VirtualTimer => 0,
            IrqId::Uart => 1,
            IrqId::VirtioInput(slot) => 2 + slot as usize,
        };
        
        if idx >= MAX_HANDLERS {
            return false;
        }
        
        unsafe {
            if let Some(handler) = HANDLERS[idx] {
                handler.handle(irq_num);
                return true;
            }
        }
    }
    false
}
```

---

## Step 3: Create Input Interrupt Handler

### File: `kernel/src/input.rs`

**Add new imports** (at top of file):
```rust
use los_hal::gic::{self, InterruptHandler, IrqId};
```

**Add slot tracking** (after `CTRL_PRESSED`):
```rust
/// TEAM_241: Track which MMIO slot the input device is on (for IRQ computation)
static INPUT_SLOT: Mutex<Option<usize>> = Mutex::new(None);
```

**Add interrupt handler struct** (before `pub fn init`):
```rust
/// TEAM_241: VirtIO Input interrupt handler for async Ctrl+C detection
pub struct InputInterruptHandler;

impl InterruptHandler for InputInterruptHandler {
    fn handle(&self, _irq: u32) {
        // Poll VirtIO input device for pending events
        poll();
        
        // Check if Ctrl+C was just pressed and signal immediately
        // Note: poll() already pushed '\x03' to buffer, but we also
        // need to signal the foreground process immediately
        check_and_signal_ctrl_c();
    }
}

/// Static handler instance for GIC registration
static INPUT_HANDLER: InputInterruptHandler = InputInterruptHandler;
```

**Add Ctrl+C check helper** (after `InputInterruptHandler`):
```rust
/// TEAM_241: Check for Ctrl+C in keyboard buffer and signal foreground process
fn check_and_signal_ctrl_c() {
    // Check if the most recent character is Ctrl+C
    // This is called from ISR context, so we need to be quick
    let buf = KEYBOARD_BUFFER.lock();
    
    // If buffer contains Ctrl+C, signal immediately
    // Note: We iterate through buffer to find any pending Ctrl+C
    // This handles the case where multiple chars were buffered
    let has_ctrl_c = buf.iter().any(|&c| c == '\x03');
    drop(buf); // Release lock before calling signal
    
    if has_ctrl_c {
        crate::syscall::signal::signal_foreground_process(
            crate::syscall::signal::SIGINT
        );
    }
}
```

> **Note**: We need to check if `RingBuffer` has an `iter()` method. If not, we need a different approach.

**Alternative if `RingBuffer::iter()` doesn't exist**:

Modify `poll()` to signal immediately when Ctrl+C is detected:

**Current code** (line ~84-87):
```rust
if *CTRL_PRESSED.lock() && code == KEY_C {
    if !KEYBOARD_BUFFER.lock().push('\x03') {
        crate::verbose!("KEYBOARD_BUFFER overflow, Ctrl+C dropped");
    }
}
```

**Modified code**:
```rust
if *CTRL_PRESSED.lock() && code == KEY_C {
    if !KEYBOARD_BUFFER.lock().push('\x03') {
        crate::verbose!("KEYBOARD_BUFFER overflow, Ctrl+C dropped");
    }
    // TEAM_241: Signal foreground process immediately on Ctrl+C
    crate::syscall::signal::signal_foreground_process(
        crate::syscall::signal::SIGINT
    );
}
```

**This is the preferred approach** - it signals at the moment of detection, not after (simpler and avoids RingBuffer API issues).

**Modify `init()` function**:

**Current signature**:
```rust
pub fn init(transport: StaticMmioTransport) {
```

**New signature**:
```rust
/// Initialize VirtIO Input device.
/// 
/// # Arguments
/// * `transport` - MMIO transport for the device
/// * `slot` - MMIO slot index (used to compute IRQ number)
pub fn init(transport: StaticMmioTransport, slot: usize) {
```

**Add at end of `init()` function** (before closing brace):
```rust
    // TEAM_241: Register interrupt handler for async Ctrl+C detection
    *INPUT_SLOT.lock() = Some(slot);
    
    let irq_id = IrqId::VirtioInput(slot as u32);
    gic::register_handler(irq_id, &INPUT_HANDLER);
    gic::active_api().enable_irq(irq_id.irq_number());
    
    crate::verbose!("VirtIO Input IRQ {} enabled (slot {})", irq_id.irq_number(), slot);
```

---

## Step 4: Simplify Ctrl+C Detection (Alternative Approach)

If the RingBuffer iteration approach is complex, use this simpler inline approach:

### File: `kernel/src/input.rs`

Modify the Ctrl+C detection in `poll()` to signal immediately:

**Location**: Line ~84-87 in `poll()` function

**Change**:
```rust
if *CTRL_PRESSED.lock() && code == KEY_C {
    if !KEYBOARD_BUFFER.lock().push('\x03') {
        crate::verbose!("KEYBOARD_BUFFER overflow, Ctrl+C dropped");
    }
    // TEAM_241: Signal foreground process immediately on Ctrl+C
    // This ensures signal is delivered even if no one is reading stdin
    crate::syscall::signal::signal_foreground_process(
        crate::syscall::signal::SIGINT
    );
}
```

And the interrupt handler becomes simply:
```rust
impl InterruptHandler for InputInterruptHandler {
    fn handle(&self, _irq: u32) {
        // TEAM_241: Poll VirtIO input device
        // poll() already handles Ctrl+C detection and signaling
        poll();
    }
}
```

---

## Verification Plan

### 1. Build Verification
```bash
cd /home/vince/Projects/LevitateOS
cargo xtask build all
```
**Expected**: Build succeeds with no errors

### 2. Existing Test Verification
```bash
# In QEMU via ./run-term.sh:
cat /README.txt      # Verify normal input still works
pipe_test            # Verify pipes work
clone_test           # Verify threading works
mmap_test            # Verify memory mapping works
```
**Expected**: All tests pass

### 3. Signal Test (Manual - Primary Fix Verification)
```bash
# In QEMU via ./run-term.sh:
signal_test
# Wait for "Waiting for signal in pause()..." message
# Press Ctrl+C
```
**Expected**: 
- See "Signal handler called with signal 2"
- Process terminates normally
- Shell prompt returns

### 4. Long-Running Process Test (Manual)
```bash
# If we have a simple infinite loop program:
# The program should terminate on Ctrl+C
```

---

## Reversal Plan

If fix doesn't work:

1. Revert `kernel/src/virtio.rs` - remove slot parameter from `input::init()` call
2. Revert `kernel/src/input.rs` - remove handler, slot tracking, and immediate signaling
3. Revert `crates/hal/src/gic.rs` - remove `VirtioInput` variant and handler mapping changes
4. Rebuild and verify original behavior is restored

---

## Code Review Checklist

Before marking as complete:

- [ ] All TEAM_241 comments added
- [ ] No clippy warnings introduced
- [ ] Handler uses existing `InterruptHandler` trait pattern
- [ ] IRQ computation matches QEMU virt machine behavior
- [ ] Existing tests still pass
- [ ] signal_test responds to Ctrl+C
