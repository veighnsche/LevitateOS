# Phase 1: Discovery — Hybrid Boot Specification

## Feature Summary
LevitateOS currently uses a monolithic `kmain` that has been partially refactored into five stages. This feature formalizes these stages into a rigorous "Hybrid Boot Specification" aligned with UEFI (SEC, PEI, DXE, BDS) and Linux (`start_kernel`) standards, with a specific focus on ensuring compatibility with the **Pixel 6 (Google Tensor GS101)** hardware boot flow (pBL -> sBL -> ABL -> Linux).

## Problem Statement
The current boot process is "functional but ad-hoc." Without a formal specification:
1. Transitioning from QEMU to physical hardware (Pixel 6) is risky.
2. Developers lack clear boundaries for where hardware initialization ends and system policy begins.
3. Interaction behaviors (like backspace/terminal wrap) are inconsistent.

## Success Criteria
- [ ] **SC1**: Formalized `BOOT_SPECIFICATION.md` exists with industry citations.
- [ ] **SC2**: `kmain` is architecturally separated into the five specified stages.
- [ ] **SC3**: Boot console (Stage 3) implements ANSI-compliant interactive behaviors.
- [ ] **SC4**: GS101-specific hardware constraints (SimpleFB, UART) are documented and mapped to stages.

## Current State Analysis
- **Stages**: 5 stages roughly defined in `main.rs`, but logic is still intertwined in a single loop.
- **Terminal**: Basic character output, recently updated with destructive backspace and blinking cursor.
- **Hardware**: QEMU-only today; GS101 DTB analysis just started.

## Codebase Reconnaissance
- `kernel/src/main.rs`: The orchestrator of all stages.
- `kernel/src/terminal.rs`: The Boot Console implementation.
- `levitate-hal/`: Contains drivers (UART, GIC, Timer) that must be allocated to specific stages.
- `docs/BOOT_SPECIFICATION.md`: Initial research document created by TEAM_060.

## Steps and Units of Work

### Step 1: Industry Alignment Analysis
- **UoW 1**: Map UEFI PI (SEC/PEI/DXE) phases to LevitateOS stages in `phase-1-step-1-uow-1.md`.
- **UoW 2**: Map Linux `start_kernel` milestones to LevitateOS stages.

### Step 2: Hardware-Target Discovery (Pixel 6)
- **UoW 1**: Analyze Pixel 6 (GS101) Device Tree for `simple-framebuffer` and UART nodes.
- **UoW 2**: Document ABL-to-Kernel handover protocols for Tensor chips.
