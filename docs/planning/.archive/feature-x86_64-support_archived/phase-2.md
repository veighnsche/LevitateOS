# Phase 2: Design — x86_64 Support

## Proposed Solution
We will implement the x86_64 support by populating `kernel/src/arch/x86_64/` with modules mirroring the `aarch64` structure (`boot`, `cpu`, `memory`, `exceptions`, `task`).

### 1. Boot Protocol
**Decision**: Use **Multiboot2**.
- **Reasoning**: Standard for x86_64 kernels. Supported by QEMU `-kernel` (implies multiboot) and GRUB.
- **Mechanism**: `asm/boot.S` will contain the Multiboot2 header (magic numbers). The entry point `_start` will be 32-bit (protected mode), which sets up 64-bit Long Mode page tables and jumps to 64-bit code.

### 2. Memory Management
**Structure**: 4-Level Paging (PML4).
- **Mapping**: Higher-half kernel (`0xFFFFFFFF80000000`).
- **Physical Map**: Linear mapping of all physical RAM at `0xFFFF800000000000`.

### 3. Interrupts (IDT)
**Structure**: 64-bit IDT.
- **Exceptions**: Standard x86 exceptions (0-31).
- **Double Fault**: Uses a separate IST (Interrupt Stack Table) stack to avoid stack overflows crashing the system (equivalent to "panic stack").

### 4. System Calls
**Instruction**: `syscall` (AMD64).
**Registers**: System V AMD64 ABI.
- **Kernel Entry**: `MSR_LSTAR` points to `syscall_entry` in `asm/syscall.S`.
- **GS Base**: Used to store per-CPU data (Kernel Stack Pointer). On entry, `swapgs` switches to kernel GS.

### 5. Context Switching
- **Mechanism**: Save/Restore callee-saved registers (`rbx`, `rbp`, `r12-r15`, `rsp`, `rip`).

## Open Questions

> [!QUESTION]
> **Boot Protocol Confirmation**: Is Multiboot2 definitely the preferred choice over Limine native protocol? Multiboot2 is generally more "standard" for QEMU direct boot.

> [!QUESTION]
> **Toolchain**: Does the user have `x86_64-unknown-none` installed? We should add a check in `xtask` or documentation.

## Design Alternatives
- **UEFI**: Could boot purely via UEFI. Complex to implement from scratch without a bootloader helper like Limine. Multiboot2 is simpler for "kernel dev on QEMU".
- **Limine**: Could use Limine protocol. Very nice for Rust, but requires the `limine` crate and a specific config. Multiboot2 is built into QEMU.

## Verification
- We will verify by running a "headless" boot test using QEMU's debugcon or serial port.
