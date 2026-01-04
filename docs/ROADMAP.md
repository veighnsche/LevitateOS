# LevitateOS Roadmap

**Last Updated:** 2026-01-04 (TEAM_031)

This document outlines the planned development phases for LevitateOS. Each completed item includes the responsible team for traceability.

---

## ✅ Phase 1: Foundation & Refactoring (Completed)

- **Objective**: Establish a modular, idiomatic Rust codebase.
- **Achievements**:
  - [x] Migrated to Cargo Workspace (`levitate-kernel`, `levitate-hal`, `levitate-utils`). (TEAM_009)
  - [x] Integrated `linked_list_allocator` for heap management.
  - [x] Basic UART (Console) and GIC (Interrupt) drivers.
  - [x] Basic VirtIO GPU and Input support.

---

## ✅ Phase 2: Idiomatic HAL & Basic Drivers (Completed)

- **Objective**: Harden the Hardware Abstraction Layer (HAL) and implement robust drivers.
- **Tasks**:
  - [x] **Timer**: AArch64 Generic Timer driver. (TEAM_010, TEAM_011)
  - [x] **PL011 UART**: Full PL011 driver with interrupt handling (RX/TX buffers). (TEAM_012, TEAM_014)
  - [x] **GICv2/v3**: Expanded GIC support with typed IRQ routing. (TEAM_015)
  - [x] **Safety**: All MMIO uses `volatile`, wrapper structs prevent unsafe state. (TEAM_016, TEAM_017)

---

## ✅ Phase 3: Memory Management (MMU) (Completed)

- **Objective**: Enable virtual memory and implement higher-half kernel architecture.
- **Tasks**:
  - [x] **Page Tables**: AArch64 page table walking, modification, and optimized 2MB block mappings. (TEAM_018, TEAM_019, TEAM_020)
  - [x] **Identity Mapping**: Initial boot mapping for transition.
  - [x] **Higher-Half Kernel**: Kernel runs at `0xFFFF800000000000` using TTBR1. (TEAM_025, TEAM_026, TEAM_027)
  - [x] **HAL Integration**: `mmu.rs` with `virt_to_phys`/`phys_to_virt` helpers. (TEAM_028)
  - [x] **Kernel Audit**: Documented all behaviors for Phase 2-3 freeze. (TEAM_021, TEAM_022)

---

## ✅ Phase 4: Storage & Filesystem (Completed)

- **Objective**: Persistent storage and basic filesystem access.
- **Tasks**:
  - [x] **VirtIO Block**: Disk driver for QEMU `virtio-blk`. (TEAM_029, TEAM_030)
  - [x] **Filesystem**: FAT32 filesystem using `embedded-sdmmc`. (TEAM_032)
  - [x] **Initramfs**: Load an initial ramdisk for early userspace. (TEAM_035, TEAM_036, TEAM_038, TEAM_039)

> [!NOTE]
> **Current Focus:** Phase 5 (Memory Management II). Phase 4 is complete.

---

## � Phase 5: Memory Management II — Dynamic Allocator (Current Priority)

- **Objective**: Replace the static heap with scalable kernel allocators.
- **Tasks**:
  - [ ] **Buddy Allocator**: Physical page allocator for large allocations.
  - [ ] **Slab Allocator**: Fast allocation for fixed-size kernel objects (tasks, file handles).
  - [ ] **Page Frame Allocator**: Integration with MMU for on-demand mapping.

---

## 🔮 Phase 6: VirtIO Ecosystem Expansion

- **Objective**: Expand hardware support using VirtIO.
- **Tasks**:
  - [ ] **VirtIO Net**: Basic network packet transmission/reception (`virtio-net`).
  - [ ] **GPU Refinement**: Text rendering or terminal emulation on GPU framebuffer.
  - [ ] **9P Filesystem**: Mount host directories via `virtio-9p`.

---

## 🔮 Phase 7: Multitasking & Scheduler

- **Objective**: Run multiple tasks concurrently.
- **Tasks**:
  - [ ] **Context Switching**: Save/Restore CPU state (registers) in assembly.
  - [ ] **Scheduler**: Cooperative (yield) or Preemptive (timer-based) Round-Robin scheduler.
  - [ ] **Task Primitives**: Define `Task` struct and `TaskControlBlock`.

---

## 🔮 Phase 8: Userspace & Syscalls

- **Objective**: Run unprivileged user programs.
- **Tasks**:
  - [ ] **EL0 Transition**: Switch CPU from EL1 (Kernel) to EL0 (User).
  - [ ] **Syscall Interface**: Define `svc` (Supervisor Call) handler and ABI.
  - [ ] **ELF Loader**: Parse and load userspace binaries from the disk/initramfs.
  - [ ] **Coreutils Strategy**:
    - *Initial*: Custom minimalist Rust binaries (bare-metal style, no libc) for `ls`, `echo`, `cat`.
    - *Long-term*: Port `uutils` (Rust Coreutils) once a standard library interface (libc-like) is established.

---

## 📱 Phase 9: Hardware Targets

- **Current**: QEMU (`virt` machine, AArch64).
- **Next Step**: **Raspberry Pi 4/5** (Standard AArch64, widely documented, accessible UART).
- **Moonshot**: **Pixel 6 (Tensor)**.
  - *Challenges*: Proprietary boot chain, complex driver porting (Linux drivers are GPL and intertwined with Linux subsystems), lack of hardware debug ports (SuzyQable required).
  - *Strategy*: Only attempt after robust MMU/Driver model is working on Pi 4.

---

## Team Registry Summary

| Phase | Teams | Description |
|-------|-------|-------------|
| 1 | 001-009 | Foundation, Workspace Refactor |
| 2 | 010-017 | Timer, UART, GIC, HAL Hardening |
| 3 | 018-028 | MMU, Higher-Half Kernel, Audit |
| 4 | 029-039 | VirtIO Block, FAT32, Initramfs |
