# Design: x86_64 Support

## Hardware Target: Intel NUC
- **CPU**: Intel Core i3 (7th Gen)
- **RAM**: 32GB
- **Storage**: 1TB NVMe (Requires NVMe driver support)
- **Interrupts**: Local & I/O APIC

## Overview
This document outlines the design for porting LevitateOS to the x86_64 architecture. The design mirrors the existing AArch64 abstraction but adheres to x86_64 specific hardware structures and the System V AMD64 ABI.

## 1. Memory Management (Paging)
x86_64 uses a 4-level paging hierarchy (PML4).

### Virtual Address Space
- **Higher Half Kernel**: The kernel will be linked at `-2GB` (0xFFFFFFFF80000000) or similar canonical high address.
- **User Space**: Lower canonical addresses (0x0000000000000000 - 0x00007FFFFFFFFFFF).
- **Physical Map**: A region (e.g., 0xFFFF800000000000) will linearly map all physical memory for easy kernel access.

### Structures
- **PML4, PDPT, PD, PT**: Each table contains 512 64-bit entries.
- **CR3**: Holds the physical address of the active PML4.

## 2. Interrupt Handling (IDT)
Interrupts are handled via the Interrupt Descriptor Table (IDT).

- **Mode**: Long Mode IDT.
- **Entry Size**: 16 bytes.
- **Stack Switching**: The Interrupt Stack Table (IST) mechanism in the TSS (Task State Segment) will be used to switch stacks for critical exceptions (Double Fault, NMI) to prevent stack overflows from crashing the handler.
- **GDT**: A Global Descriptor Table is required to define Kernel Code/Data and User Code/Data segments (even though segmentation is mostly disabled in Long Mode, the selectors are still used for permission checks).

## 3. System Calls
The `syscall` instruction (opcode `0F 05`) will be used.

### ABI (System V AMD64)
- **Instruction**: `syscall`
- **Kernel Entry**: Controlled by `MSR_LSTAR` (Target RIP), `MSR_STAR` (CS/SS selectors), `MSR_SFMASK` (RFLAGS mask).
- **Registers**:
  - `RAX`: Syscall Number / Return Value
  - `RDI`: Arg 1
  - `RSI`: Arg 2
  - `RDX`: Arg 3
  - `R10`: Arg 4 (Note: RCX is used for RIP save by `syscall`, so R10 is used instead)
  - `R8`:  Arg 5
  - `R9`:  Arg 6
- **Clobbered**: `RCX` (saved RIP), `R11` (saved RFLAGS).

## 4. Task Switching
Context switching will be software-based, saving callee-saved registers.

### Context Struct
```rust
#[repr(C)]
pub struct Context {
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64, // Saved instruction pointer
    pub rsp: u64, // Saved stack pointer
}
```

## 5. Boot Protocol
We will target **Multiboot2**. This allows booting via GRUB or QEMU's `-kernel` option.
- **Header**: A Multiboot2 header must be present in the first 32KiB of the kernel image.
- **State on Entry**:
  - CPU in Protected Mode (32-bit).
  - Paging disabled.
  - We must transition to Long Mode (64-bit) immediately in `asm/boot.S`.

## 6. Implementation Strategy
We will implement the modules in `kernel/src/arch/x86_64` one by one, starting with `boot` and `cpu` to get a minimal "hang", then `memory` to get a mapped kernel, then `logger` for output.
