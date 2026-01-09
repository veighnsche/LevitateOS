# x86_64 Hardware Abstraction Layer

This directory contains the x86_64-specific HAL implementation, organized into logical compartments.

## Directory Structure

```
x86_64/
├── cpu/           # CPU structures (GDT, TSS, IDT, exceptions)
├── mem/           # Memory management (paging, MMU, frame allocator)
├── interrupts/    # Interrupt handling (APIC, IOAPIC, PIT)
├── io/            # I/O devices (serial, VGA, console)
├── boot/          # Boot info (Limine-only)
└── mod.rs         # Main module with init functions
```

## Boot Flow

The following diagram shows the x86_64 boot sequence from BIOS/bootloader to kernel:

```mermaid
flowchart TD
    subgraph Limine Bootloader
        A[UEFI/BIOS] --> B[Limine]
        B --> C[Load kernel ELF]
        C --> D[Setup HHDM mapping]
        D --> E[Jump to kernel entry]
    end

    subgraph HAL Init
        E --> F[init]
        F --> G[Serial init]
        G --> H[GDT/IDT init]
        H --> I[Exceptions init]
        I --> J[APIC/IOAPIC init]
        J --> K[PIT timer init]
        K --> L[HAL Ready]
    end

    subgraph Kernel Main
        L --> M[Console init]
        M --> N[Memory init]
        N --> O[Task init]
        O --> P[init::run]
    end
```

## Memory Layout

```mermaid
flowchart LR
    subgraph Physical Memory
        PA[0x0 - 0x100000<br/>Legacy BIOS]
        PB[0x200000<br/>Kernel Load]
        PC[0x800000 - 0x1000000<br/>Early Frame Pool]
        PD[0xB8000<br/>VGA Buffer]
        PE[0xFEC00000<br/>IOAPIC]
        PF[0xFEE00000<br/>Local APIC]
    end

    subgraph Virtual Memory
        VA[0xFFFF800000000000<br/>PMO - Physical Memory Offset]
        VB[0xFFFFFFFF80000000<br/>Kernel Higher-Half]
        VC[Identity Map<br/>First 1MB]
    end

    PA -.-> VC
    PB -.-> VB
    PA -.-> VA
```

## Compartment Details

### cpu/ - CPU Structures

```mermaid
classDiagram
    class GDT {
        +null: u64
        +kernel_code: u64
        +kernel_data: u64
        +user_data: u64
        +user_code: u64
        +tss: GdtTssEntry
        +init()
    }

    class TSS {
        +rsp0: u64
        +ist[7]: u64
        +set_kernel_stack()
    }

    class IDT {
        +entries[256]: IdtEntry
        +set_handler()
        +load()
    }

    class Exceptions {
        +page_fault_handler()
        +gp_fault_handler()
        +double_fault_handler()
        +irq_dispatch()
    }

    GDT --> TSS : contains
    IDT --> Exceptions : routes to
```

### mem/ - Memory Management

```mermaid
flowchart TB
    subgraph 4-Level Paging
        PML4[PML4 - Level 4<br/>512 entries]
        PDPT[PDPT - Level 3<br/>512 entries]
        PD[PD - Level 2<br/>512 entries]
        PT[PT - Level 1<br/>512 entries]
        PAGE[4KB Page Frame]

        PML4 --> PDPT
        PDPT --> PD
        PD --> PT
        PT --> PAGE
    end

    subgraph MMU Functions
        MAP[map_page]
        UNMAP[unmap_page]
        VIRT[virt_to_phys]
        PHYS[phys_to_virt]
    end

    subgraph Frame Allocator
        EARLY[EarlyFrameAllocator<br/>Bump allocator<br/>0x800000 - 0x1000000]
    end

    MAP --> PT
    EARLY --> MAP
```

### interrupts/ - Interrupt Handling

```mermaid
flowchart LR
    subgraph Hardware
        TIMER[PIT Timer<br/>IRQ 0]
        SERIAL[COM1<br/>IRQ 4]
        DEVICES[Other Devices]
    end

    subgraph IOAPIC
        ROUTE[Route IRQs<br/>to vectors]
    end

    subgraph LAPIC
        VECTOR[Deliver to CPU]
        EOI[Signal EOI]
    end

    subgraph CPU
        IDT2[IDT Lookup]
        HANDLER[Exception/IRQ Handler]
    end

    TIMER --> ROUTE
    SERIAL --> ROUTE
    DEVICES --> ROUTE
    ROUTE --> VECTOR
    VECTOR --> IDT2
    IDT2 --> HANDLER
    HANDLER --> EOI
```

### io/ - I/O Devices

```mermaid
flowchart TB
    subgraph Console Output
        PRINT[println! macro]
        WRITER[WRITER static]
        SERIAL[SerialPort<br/>COM1 0x3F8]
        VGA[VGA Buffer<br/>0xB8000]
    end

    PRINT --> WRITER
    WRITER --> SERIAL
    WRITER -.-> VGA

    subgraph Serial Port
        TX[Transmit byte]
        INIT[Initialize 115200 baud]
    end

    SERIAL --> TX
    SERIAL --> INIT
```

## Key Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `KERNEL_VIRT_BASE` | `0xFFFFFFFF80000000` | Kernel higher-half base |
| `PHYS_OFFSET` | `0xFFFF800000000000` | Physical memory offset (PMO) |
| `PAGE_SIZE` | `0x1000` (4KB) | Standard page size |
| `HUGE_PAGE_SIZE` | `0x200000` (2MB) | Huge page size |
| `APIC_BASE` | `0xFEE00000` | Local APIC address |
| `IOAPIC_BASE` | `0xFEC00000` | I/O APIC address |

## Known Issues

### TEAM_317: HHDM Doesn't Map MMIO

**Problem:** Limine's HHDM (Higher Half Direct Map) only maps RAM, not MMIO regions.

```
HHDM maps:     Physical RAM → 0xFFFF800000000000 + phys_addr
HHDM does NOT: APIC (0xFEE00000), IOAPIC (0xFEC00000), PCI MMIO
```

**Symptom:** Page fault at `0xFFFF8000FEE000xx` when accessing APIC via `phys_to_virt()`.

**Current Workaround:** Skip APIC/IOAPIC init, use legacy PIC mode with PIT timer.

**Proper Fix (TODO):** Map MMIO regions explicitly before enabling APIC mode.

## Files

| File | Description |
|------|-------------|
| `cpu/gdt.rs` | Global Descriptor Table and TSS |
| `cpu/idt.rs` | Interrupt Descriptor Table |
| `cpu/exceptions.rs` | Exception handlers and IRQ dispatch |
| `mem/paging.rs` | Page table structures and operations |
| `mem/mmu.rs` | Memory mapping, address translation |
| `mem/frame_alloc.rs` | Early bump allocator for page frames |
| `interrupts/apic.rs` | Local APIC controller |
| `interrupts/ioapic.rs` | I/O APIC for external interrupts |
| `interrupts/pit.rs` | Programmable Interval Timer |
| `interrupts/state.rs` | Interrupt enable/disable/restore |
| `io/serial.rs` | COM1 serial port driver |
| `io/vga.rs` | VGA text mode buffer |
| `io/console.rs` | Console writer abstraction |
| `boot/multiboot2.rs` | Multiboot2 boot info parsing |
