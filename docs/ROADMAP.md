# LevitateOS Roadmap

**TEAM_009: Workspace Refactoring & Roadmap Definition**

This document outlines the planned development phases for LevitateOS, following the workspace refactoring.

## ✅ Phase 1: Foundation & Refactoring (Completed)
- **Objective**: Establish a modular, idiomatic Rust codebase.
- **Achievements**:
  - Migrated to Cargo Workspace (`levitate-kernel`, `levitate-hal`, `levitate-utils`).
  - Integrated `linked_list_allocator` for heap management.
  - Basic UART (Console) and GIC (Interrupt) drivers.
  - Basic VirtIO GPU and Input support.

## ✅ Phase 2: Idiomatic HAL & Basic Drivers (Completed)
- **Objective**: Harden the Hardware Abstraction Layer (HAL) and implement robust drivers.
- **Tasks**:
  - [x] **Timer**: Implement a proper AArch64 Generic Timer driver in `levitate-hal`.
  - [x] **PL011 UART**: Refactor `console.rs` into a full PL011 driver with interrupt handling (RX/TX buffers).
  - [x] **GICv2/v3**: Expand GIC support to handle specific IRQ routing cleanly. (TEAM_015)
  - [x] **Safety**: Ensure all MMIO operations use `volatile` correctly and wrapper structs prevent unsafe state. (TEAM_017 — HAL Hardening)

## ✅ Phase 3: Memory Management (MMU) (Completed)
- **Objective**: Enabled virtual memory and implemented a higher-half kernel architecture.
- **Tasks**:
  - [x] **Page Tables**: Implement AArch64 page table walking and modification.
  - [x] **Identity Mapping**: Initial boot mapping for transition.
  - [x] **Higher-Half Kernel**: Kernel moved to `0xFFFF800000000000` using TTBR1. (TEAM_027)
  - [x] **HAL Integration**: Refactored `mmu.rs` for physical address support and added conversion helpers.

## 🚧 Phase 4: VirtIO & Filesystem (Next Priority)
- **Objective**: Expand hardware support and persistency.
- **Tasks**:
  - [x] **VirtIO Block**: Implement driver for disk I/O (`virtio-blk`). (TEAM_029)
  - [ ] **Memory Management II**: Replace the static heap with a dynamic Buddy Allocator + Slab for kernel objects.
  - [ ] **Filesystem**: Basic FAT32 or ext2 reader to load initial programs.

## 🔮 Phase 4: VirtIO Ecosystem
- **Objective**: Expand hardware support using VirtIO.
- **Tasks**:
  - [x] **VirtIO Block**: Implement driver for disk I/O (`virtio-blk`). (TEAM_029)
  - [ ] **VirtIO Net**: Basic network packet transmission/reception (`virtio-net`).
  - [ ] **GPU Refinement**: Add text rendering or terminal emulation on the GPU framebuffer.

## 🔮 Phase 5: Multitasking & Scheduler
- **Objective**: Run multiple tasks concurrently.
- **Tasks**:
  - [ ] **Context Switching**: Save/Restore CPU state (registers) in assembly.
  - [ ] **Scheduler**: Cooperative (yield) or Preemptive (timer-based) Round-Robin scheduler.
  - [ ] **Threads/Tasks**: Define a `Task` struct and `TaskControlBlock`.

## 🔮 Phase 6: Userspace & Syscalls
- **Objective**: Run unprivileged user programs.
- **Tasks**:
  - [ ] **EL0 Transition**: Switch CPU from EL1 (Kernel) to EL0 (User).
  - [ ] **Syscall Interface**: Define `svc` (Supervisor Call) handler and ABI.
  - [ ] **ELF Loader**: Parse and load userspace binaries from the disk/initramfs.
  - [ ] **Coreutils Strategy**:
    - *Initial*: Custom minimalist Rust binaries (bare-metal style, no libc) for `ls`, `echo`, `cat`.
    - *Long-term*: Port `uutils` (Rust Coreutils) once a standard library interface (libc-like) is established.

## 📱 Phase 7: Hardware Targets
- **Current**: QEMU (`virt` machine, AArch64).
- **Next Step**: **Raspberry Pi 4/5** (Standard AArch64, widely documented, accessible UART).
- **Moonshot**: **Pixel 6 (Tensor)**.
  - *Challenges*: Proprietary boot chain, complex driver porting (Linux drivers are GPL and intertwined with Linux subsystems), lack of hardware debug ports (SuzyQable required).
  - *Strategy*: Only attempt after robust MMU/Driver model is working on Pi 4.
