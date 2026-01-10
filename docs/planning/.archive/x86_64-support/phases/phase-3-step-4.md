# Phase 3 — Step 4: HAL Implementation (x86_64 Backends)

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Implement x86_64-specific hardware abstraction backends for console output, interrupts, and timers.

## Prerequisites
- Step 2 (Early Boot) must be complete — kernel reaches `kernel_main`
- Step 3 (Arch Stubs) provides the interface contracts

---

## UoW 4.1: Implement Serial Console (COM1)

**Goal**: Enable serial output on COM1 for early debugging.

**File**: `crates/hal/src/x86_64/serial.rs` (new)

**Tasks**:
1. Create `crates/hal/src/x86_64/` directory
2. Create `serial.rs` with `SerialPort` struct
3. Define COM1 base address: `0x3F8`
4. Implement port I/O using `inb`/`outb` (inline assembly or `x86_64` crate)
5. Implement initialization: set baud rate, line control, FIFO
6. Implement `write_byte(u8)` and `write_str(&str)`
7. Implement `core::fmt::Write` trait

**Exit Criteria**:
- `println!` macro works over serial
- QEMU `-serial stdio` shows kernel output

**Verification**:
```bash
cargo xtask run --arch x86_64
# Serial output should appear in terminal
```

---

## UoW 4.2: Implement VGA Text Mode Console

**Goal**: Enable text output to VGA buffer for visual feedback.

**File**: `crates/hal/src/x86_64/vga.rs` (new)

**Tasks**:
1. Create `vga.rs` with `VgaBuffer` struct
2. Define VGA buffer address: `0xB8000`
3. Define color codes (foreground/background)
4. Implement `write_byte(byte: u8, col: usize, row: usize, color: u8)`
5. Implement scrolling when buffer is full
6. Implement `core::fmt::Write` trait
7. Create static `WRITER` with spin lock

**Exit Criteria**:
- Text appears in QEMU VGA window
- Colors work correctly

**Verification**:
- Visual inspection in QEMU window

---

## UoW 4.3: Implement IDT Structure

**Goal**: Define the Interrupt Descriptor Table structure for x86_64.

**File**: `crates/hal/src/x86_64/idt.rs` (new)

**Tasks**:
1. Create `idt.rs`
2. Define `IdtEntry` struct (16 bytes per entry):
   - offset_low: u16
   - selector: u16
   - ist: u8
   - type_attr: u8
   - offset_mid: u16
   - offset_high: u32
   - zero: u32
3. Define `Idt` as array of 256 entries
4. Implement `IdtEntry::new(handler: u64, selector: u16, ist: u8, gate_type: u8)`
5. Define gate types: Interrupt (0x8E), Trap (0x8F)

**Exit Criteria**:
- IDT structure compiles
- Size is exactly 256 × 16 = 4096 bytes

**Verification**:
```rust
assert_eq!(core::mem::size_of::<Idt>(), 4096);
```

---

## UoW 4.4: Implement CPU Exception Handlers

**Goal**: Create handler stubs for CPU exceptions (0-31).

**File**: `crates/hal/src/x86_64/exceptions.rs` (new)

**Tasks**:
1. Create `exceptions.rs`
2. Define exception handler macro that:
   - Saves all registers
   - Calls a Rust handler
   - Restores registers
   - Returns with `iretq`
3. Implement handlers for:
   - #DE (0): Divide Error
   - #DB (1): Debug
   - #NMI (2): Non-Maskable Interrupt
   - #BP (3): Breakpoint
   - #OF (4): Overflow
   - #BR (5): Bound Range
   - #UD (6): Invalid Opcode
   - #NM (7): No Math Coprocessor
   - #DF (8): Double Fault (with error code)
   - #GP (13): General Protection (with error code)
   - #PF (14): Page Fault (with error code)
4. Rust handler prints exception name and halts

**Exit Criteria**:
- All 32 exception vectors have handlers
- Page fault shows faulting address from CR2

**Verification**:
- Trigger divide-by-zero; see "Divide Error" message

---

## UoW 4.5: Implement IDT Loading and Initialization

**Goal**: Load the IDT and enable exception handling.

**File**: `crates/hal/src/x86_64/idt.rs` (continued)

**Tasks**:
1. Create static `IDT: Idt` with lazy initialization
2. Implement `init()` function:
   - Populate entries 0-31 with exception handlers from UoW 4.4
   - Create IDT pointer struct (limit + base)
   - Execute `lidt` instruction
3. Export `x86_64::idt::init()` from hal

**Exit Criteria**:
- `lidt` executes without fault
- Exceptions trigger handlers instead of triple fault

**Verification**:
```rust
// In kernel_main:
x86_64::idt::init();
unsafe { asm!("int3"); }  // Should trigger breakpoint handler
```

---

## UoW 4.6: Detect and Initialize Local APIC

**Goal**: Initialize the Local APIC for interrupt handling.

**File**: `crates/hal/src/x86_64/apic.rs` (new)

**Tasks**:
1. Create `apic.rs`
2. Check APIC availability via CPUID (leaf 1, EDX bit 9)
3. Read Local APIC base from MSR 0x1B (IA32_APIC_BASE)
4. Map APIC MMIO region (typically 0xFEE00000)
5. Define APIC register offsets:
   - ID (0x20), Version (0x30), TPR (0x80), EOI (0xB0)
   - Spurious (0xF0), LVT Timer (0x320), etc.
6. Implement `init()`:
   - Enable APIC (set bit 8 in Spurious register)
   - Set spurious vector to 0xFF
7. Implement `end_of_interrupt()`

**Exit Criteria**:
- APIC is enabled
- Local APIC ID can be read

**Verification**:
- Print Local APIC ID at boot

---

## UoW 4.7: Implement I/O APIC for External IRQs

**Goal**: Initialize I/O APIC to route external interrupts.

**File**: `crates/hal/src/x86_64/ioapic.rs` (new)

**Tasks**:
1. Create `ioapic.rs`
2. Define I/O APIC base address (typically 0xFEC00000, from ACPI MADT)
3. Implement register access:
   - IOREGSEL (0x00): select register
   - IOWIN (0x10): read/write data
4. Implement `read_reg(reg: u32) -> u32`
5. Implement `write_reg(reg: u32, value: u32)`
6. Implement `init()`:
   - Read APIC version and max redirection entries
7. Implement `route_irq(irq: u8, vector: u8, dest_apic: u8)`:
   - Write to redirection table entry

**Exit Criteria**:
- I/O APIC initialized
- Can read max IRQ count

**Verification**:
- Print I/O APIC version and max IRQs

---

## UoW 4.8: Implement PIT Timer

**Goal**: Set up Programmable Interval Timer for basic timing.

**File**: `crates/hal/src/x86_64/pit.rs` (new)

**Tasks**:
1. Create `pit.rs`
2. Define PIT ports: Channel 0 (0x40), Command (0x43)
3. Define PIT frequency: 1.193182 MHz
4. Implement `init(frequency_hz: u32)`:
   - Calculate divisor
   - Write command byte (channel 0, lobyte/hibyte, rate generator)
   - Write divisor low then high byte
5. Route PIT (IRQ 0) to vector 32 via I/O APIC
6. Implement interrupt handler that increments tick counter

**Exit Criteria**:
- Timer interrupts fire at configured rate
- Global tick counter increments

**Verification**:
- Print tick count periodically

---

## UoW 4.9: Implement InterruptController Trait for APIC

**Goal**: Make APIC conform to the `InterruptController` trait.

**File**: `crates/hal/src/x86_64/apic.rs` (continued)

**Tasks**:
1. Create `ApicController` struct
2. Implement `InterruptController` trait:
   - `init()`: call APIC + I/O APIC init
   - `enable_irq(irq)`: route IRQ via I/O APIC
   - `disable_irq(irq)`: mask in I/O APIC
   - `acknowledge()`: read ISR to get current IRQ
   - `end_of_interrupt(irq)`: write EOI register
   - `is_spurious(irq)`: check for vector 0xFF
   - `register_handler()`: store in handler table
   - `map_irq()`: IRQ to vector translation

**Exit Criteria**:
- Kernel can use generic `InterruptController` interface
- AArch64 GIC and x86 APIC share same trait

**Verification**:
- Use trait methods in generic kernel code

---

## UoW 4.10: Create x86_64 HAL Module Structure

**Goal**: Wire all x86_64 HAL components together.

**Files**: 
- `crates/hal/src/x86_64/mod.rs` (new)
- `crates/hal/src/lib.rs` (modify)

**Tasks**:
1. Create `crates/hal/src/x86_64/mod.rs`:
   - `pub mod serial;`
   - `pub mod vga;`
   - `pub mod idt;`
   - `pub mod exceptions;`
   - `pub mod apic;`
   - `pub mod ioapic;`
   - `pub mod pit;`
2. Export `ApicController` and console types
3. In `crates/hal/src/lib.rs`:
   - Add `#[cfg(target_arch = "x86_64")] pub mod x86_64;`
4. Ensure conditional compilation doesn't break AArch64

**Exit Criteria**:
- `cargo xtask build --arch x86_64` compiles HAL
- `cargo xtask build --arch aarch64` still works

**Verification**:
```bash
cargo xtask build --arch aarch64  # Must still work
cargo xtask build --arch x86_64   # New code compiles
```

---

## Progress Tracking
- [x] UoW 4.1: Serial Console
- [x] UoW 4.2: VGA Console
- [x] UoW 4.3: IDT Structure
- [x] UoW 4.4: Exception Handlers
- [x] UoW 4.5: IDT Init
- [x] UoW 4.6: Local APIC
- [x] UoW 4.7: I/O APIC
- [x] UoW 4.8: PIT Timer
- [x] UoW 4.9: Trait Implementation
- [x] UoW 4.10: Module Structure

## Dependencies Graph
```
UoW 4.1 ──┐
          ├──→ UoW 4.10 (all must complete)
UoW 4.2 ──┤
          │
UoW 4.3 ──→ UoW 4.4 ──→ UoW 4.5
          │
UoW 4.6 ──┼──→ UoW 4.9
          │
UoW 4.7 ──┤
          │
UoW 4.8 ──┘
```
