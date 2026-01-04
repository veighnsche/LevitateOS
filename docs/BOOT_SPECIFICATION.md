# LevitateOS Boot & Console Specification

This document aligns LevitateOS's boot process and console behavior with industry standards (UEFI, Linux Boot Protocol, and ANSI/VT100).

## 1. Multi-Stage Boot Sequence
LevitateOS follows a structured sequence inspired by the **UEFI PI (Platform Initialization)** and **Linux `start_kernel`** phases.

| LevitateOS Stage | UEFI Equivalent | Linux Equivalent | Description |
| :--- | :--- | :--- | :--- |
| **Stage 1: Core HAL** | SEC (Security) | `setup_arch()` | CPU initialization, GIC/Interrupt setup, and early UART. |
| **Stage 2: Memory & MMU** | PEI (Pre-EFI) | `mm_init()` | RAM detection, Page Table establishment, and Stack setup. |
| **Stage 3: Boot Console** | DXE (Console Init) | `console_init()` | GPU Terminal / Framebuffer initialization for visual feedback. |
| **Stage 4: Discovery** | DXE / BDS | `vfs_caches_init()` | VirtIO scanning, initrd extraction, and FS mounting. |
| **Stage 5: Ready State** | BDS (Boot Device) | `rest_init()` / `init` | Kernel steady state; transition to scheduler or userland. |

## 2. Terminal Interaction Standard
LevitateOS aims for **ANSI VT100 / XTerm** compatibility for its Boot Console, with the following rigorous definitions for non-userspace interaction.

### 2.1 Control Character Handling
Following the **ANSI X3.64 / ECMA-48** standards:

| Character | Name | Action (Standard Citation) |
| :--- | :--- | :--- |
| `0x08` | **BS (Backspace)** | **Non-destructive cursor move.** Moves cursor one cell left. (VT100 Spec). |
| `0x0A` | **LF (Line Feed)** | Move cursor to same column on the next line. (ECMA-48). |
| `0x0D` | **CR (Carriage Return)** | Move cursor to column 0 of current line. (ECMA-48). |
| `0x09` | **HT (Horizontal Tab)** | Advance to next tab stop (default: 8 columns). |

### 2.2 Interactive "Destructive" Backspace
While the VT100 standard defines `0x08` as non-destructive, the **POSIX Terminal Interface** and modern shells (Bash/Zsh) emulate a "Destructive Backspace" via the sequence: `BS -> SPACE -> BS`.
- **LevitateOS Implementation**: To provide a user-friendly Boot Console, LevitateOS implements **Stage-3 Interactive Backspace** which performs this erasure automatically when `0x08` is received, including line-wrap support (moving back to the parent line).

## 3. Early Kernel Logging (`earlycon`)
Inspired by the **Linux `earlycon`** and **UEFI `SerialIo`** protocols:
- **Serial Priority**: The UART console is the primary diagnostic source.
- **Console Redirection**: Output is duplicated to the GPU Terminal (Stage 3+) for visibility on physical displays without attached serial debuggers.

## 4. Hardware Mapping: Pixel 6 (GS101 / Tensor G1)

To ensure "Pixel-ready" behavior, the LevitateOS boot stages map to GS101 hardware as follows:

| Stage | GS101 Component | Implementation Notes |
| :--- | :--- | :--- |
| **Stage 1** | **GS101 UART** | Managed via USB-C SBU pins. 115200 8N1 @ 1.8V. |
| **Stage 2** | **Low-Power DDR** | RAM region detected via DTB (typically 0x80000000+). |
| **Stage 3** | **SimpleFB** | Early console uses `simple-framebuffer` node in DTB. |
| **Stage 4** | **UFS / Exynos-DW** | Block discovery shifts from VirtIO to UFS storage. |
| **Stage 5** | **TrustZone (EL1/EL0)** | Final state transitions to secure userland processes. |

> [!NOTE]
> **SimpleFB Compatibility**: Our current Stage 3 `terminal.rs` is compatible with the Pixel 6 simple-framebuffer protocol, as it only requires a linear memory buffer and resolution parameters provided by the bootloader (ABL).
