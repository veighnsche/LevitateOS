# Phase 2: Design — x86_64 Support (Intel NUC)

## Proposed Solution
Introduce a hardware abstraction layer (HAL) trait system for core kernel services (Interrupts, Timers, MMU) to support both AArch64 and x86_64. Implement x86_64 backends using standard PC hardware (APIC, PIT, PML4).

### User-Facing Behavior
- Users can choose the target architecture during build (e.g., `cargo xtask run --arch x86_64`).
- Boot output appears on both VGA text mode and COM1 serial port.
- System responds to keyboard input via PS/2 or USB (VirtIO).

### System Behavior
1. **Boot**: Multiboot2 entry point in `boot.S` transitions from 32-bit protected mode to 64-bit long mode.
2. **MMU**: Initialize PML4 tables with identity mapping for early boot and higher-half mapping for kernel space.
3. **Interrupts**: Initialize Local APIC and I/O APIC; set up IDT for exception and IRQ handling.
4. **Console**: Implement `Console` trait for VGA and Serial.

## API Design

### 1. Interrupt Controller Trait
```rust
pub trait InterruptController {
    fn init(&self);
    fn enable_irq(&self, irq: u32);
    fn disable_irq(&self, irq: u32);
    fn end_of_interrupt(&self, irq: u32);
}
```

### 2. MMU Interface
```rust
pub trait MmuInterface {
    fn map_page(&mut self, va: usize, pa: usize, flags: PageFlags) -> Result<(), MmuError>;
    fn unmap_page(&mut self, va: usize) -> Result<(), MmuError>;
    fn switch_to(&self);
}
```

## Behavioral Decisions
- **Bootloader**: Use Multiboot2. Why? Broad support in GRUB and QEMU.
- **Paging**: Use 4KB pages initially for simplicity, matching AArch64 granule.
- **Interrupts**: Ignore legacy PIC; use APIC exclusively for modern x86_64 compatibility.

## Open Questions
- **Q2.1**: Should we support UEFI directly, or rely on Multiboot2-compatible loaders like GRUB?
  - *Recommendation*: Start with Multiboot2 for QEMU ease-of-use.
- **Q2.2**: How should we handle the differences in `SyscallFrame` between architectures in `kernel/src/syscall/mod.rs`?
  - *Recommendation*: Use a per-arch `SyscallFrame` struct that implements a common trait.
- **Q2.3**: The NUC uses NVMe. Should we prioritize an NVMe driver or stick to VirtIO-Block for initial x86_64 support?
  - *Recommendation*: VirtIO-Block first (via QEMU) to stabilize the architecture, then NVMe for bare-metal NUC.

## Steps
### Step 1: Define HAL Traits
- **Goal**: Formalize the interfaces in `crates/hal/src/lib.rs`.
- **Tasks**: Create traits for `InterruptController`, `Timer`, and `Console`.

### Step 2: Implement x86_64 Boot Protocol
- **Goal**: Reach `kernel_main` on x86_64.
- **Tasks**: Implement Multiboot2 header and transition to Long Mode.

### Step 3: Implement x86_64 HAL Backends
- **Goal**: Functional MMU and Interrupts.
- **Tasks**: Implement `Pml4Mmu`, `ApicController`, and `PitTimer`.
