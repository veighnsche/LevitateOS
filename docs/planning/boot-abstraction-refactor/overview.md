# Boot Abstraction Refactor — Overview

## TEAM_280 Planning Document

### The Problem
The current x86_64 boot implementation violates UNIX philosophy:
- **330 lines of assembly** doing work bootloaders should handle
- **No abstraction** between bootloader protocols and kernel
- **Patch on patch** fixes for APIC mapping, register preservation
- **No real hardware path** - can't boot on Intel NUC (UEFI)

### The Solution
A clean `BootInfo` abstraction inspired by UNIX philosophy:

| UNIX Principle | Application |
|----------------|-------------|
| **Modularity** | One module per boot protocol (Limine, Multiboot, DTB) |
| **Composition** | BootInfo struct consumable by any subsystem |
| **Types** | Enums for memory types, structs for regions |
| **Silence** | Boot success = no output |
| **Safety** | Unsafe access wrapped in safe APIs |

### Architecture Change

```
BEFORE:
┌─────────────┐     ┌─────────────┐
│ Multiboot   │     │    DTB      │
│ (x86_64)    │     │ (AArch64)   │
└──────┬──────┘     └──────┬──────┘
       │ different          │ different
       │ signatures         │ signatures
       ▼                    ▼
┌─────────────┐     ┌─────────────┐
│kernel_main  │     │ rust_main   │
│(magic,info) │     │   (dtb)     │
└─────────────┘     └─────────────┘

AFTER:
┌─────────────┬─────────────┬─────────────┐
│   Limine    │  Multiboot  │     DTB     │
└──────┬──────┴──────┬──────┴──────┬──────┘
       │             │             │
       └─────────────┼─────────────┘
                     │
                     ▼ parse_*() → BootInfo
              ┌─────────────┐
              │  BootInfo   │
              │   struct    │
              └──────┬──────┘
                     │
                     ▼ unified signature
              ┌─────────────┐
              │ kernel_main │
              │ (&BootInfo) │
              └─────────────┘
```

### Phases

| Phase | Purpose | Key Deliverable |
|-------|---------|-----------------|
| **Phase 1** | Discovery & Safeguards | Lock in tests, map current code |
| **Phase 2** | Structural Extraction | BootInfo types, protocol parsers |
| **Phase 3** | Migration | Move callers to BootInfo |
| **Phase 4** | Cleanup | Delete boot.S, dead code |
| **Phase 5** | Hardening | NUC boot, documentation |

### Expected Outcomes

| Metric | Before | After |
|--------|--------|-------|
| x86_64 boot assembly | 330 lines | 0-50 lines |
| Entry point signatures | 2 (different) | 1 (unified) |
| Real hardware support | None | Intel NUC via UEFI |
| Boot protocol coupling | Hardcoded | Pluggable |

### Files

- `phase-1.md` - Discovery and Safeguards
- `phase-2.md` - Structural Extraction (BootInfo design)
- `phase-3.md` - Migration strategy
- `phase-4.md` - Cleanup (dead code removal)
- `phase-5.md` - Hardening and Handoff

### Target Hardware
- Intel NUC i3 7th Gen
- 32GB RAM
- 1TB NVMe
- UEFI firmware (no legacy BIOS)
