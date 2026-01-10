# Phase 1: Discovery and Safeguards

**TEAM_332** | VirtIO Driver Reorganization  
**TEAM_333** | Reviewed: 2026-01-09 — Open questions answered in plan.md

## Refactor Summary

Reorganize scattered VirtIO driver code into a clean, modular crate structure with proper abstractions for transport (MMIO/PCI) and device drivers.

### Pain Points

1. **Scattered location** - Drivers in `kernel/src/{block,input,net,gpu}.rs` + `crates/gpu/`
2. **No transport abstraction** - Each driver handles MMIO/PCI differently
3. **Duplicated discovery logic** - Device enumeration repeated per driver
4. **Dead code** - `crates/virtio/` is unused
5. **No common interface** - No trait for VirtIO driver lifecycle

### Motivation

TEAM_331 exposed this when adding PCI input support - the pattern had to be duplicated again (MMIO vs PCI enum, separate init functions, etc.).

## Success Criteria

### Before

```
kernel/src/
├── block.rs      # VirtIO Block (MMIO only, aarch64)
├── input.rs      # VirtIO Input (MMIO + PCI, mixed)
├── net.rs        # VirtIO Net (MMIO only, aarch64)
├── gpu.rs        # GPU (PCI + Limine FB, complex)
└── virtio.rs     # HAL + device discovery

crates/
├── gpu/          # VirtIO GPU wrapper
├── pci/          # PCI enumeration
├── virtio/       # UNUSED reference code
└── hal/src/virtio.rs  # HAL implementation
```

### After

```
crates/
├── drivers/
│   ├── virtio-blk/     # Block driver crate
│   ├── virtio-input/   # Input driver crate
│   ├── virtio-net/     # Network driver crate
│   └── virtio-gpu/     # GPU driver crate (renamed from gpu/)
├── virtio-transport/   # Unified MMIO/PCI transport
├── pci/                # PCI enumeration (keep)
└── hal/                # HAL (keep, cleanup virtio.rs)

kernel/src/
├── drivers/            # Thin kernel integration layer
│   ├── mod.rs
│   ├── block.rs        # Kernel block API using virtio-blk
│   ├── input.rs        # Kernel input API using virtio-input
│   ├── net.rs          # Kernel net API using virtio-net
│   └── gpu.rs          # Kernel GPU API using virtio-gpu
└── virtio.rs           # Device discovery only
```

## Behavioral Contracts

### Public APIs That Must Remain Stable

| Module | Function | Callers |
|--------|----------|---------|
| `kernel/src/input.rs` | `read_char()` | Shell, syscall |
| `kernel/src/input.rs` | `poll()` | Timer interrupt |
| `kernel/src/block.rs` | `read_block()`, `write_block()` | FS layer |
| `kernel/src/gpu.rs` | `GPU` static | Terminal, display |
| `kernel/src/gpu.rs` | `get_resolution()` | Terminal |
| `kernel/src/net.rs` | `send()`, `receive()` | Network stack |

### Transport Support Matrix

| Driver | aarch64 | x86_64 |
|--------|---------|--------|
| Block | MMIO | PCI (TODO) |
| Input | MMIO | PCI |
| Net | MMIO | PCI (TODO) |
| GPU | MMIO | PCI + Limine FB |

## Golden/Regression Tests

### Must Pass Before Refactor

1. `cargo xtask test levitate` - Screenshot tests (both arches)
2. `cargo xtask test behavior` - Boot behavior golden logs
3. `cargo test` - Unit tests
4. Manual: x86_64 keyboard input works
5. Manual: aarch64 boots and shows terminal

### Baselines to Lock

- `tests/golden_boot.txt` - aarch64 boot log
- `tests/golden_boot_x86_64.txt` - x86_64 boot log
- Screenshot brightness thresholds

## Current Architecture Notes

### Dependency Graph

```
kernel/src/virtio.rs
    └── Uses: virtio-drivers crate (external)
    └── Provides: VirtioHal, device discovery

kernel/src/input.rs
    └── Uses: virtio-drivers::VirtIOInput
    └── Uses: los_hal::VirtioHal
    └── Uses: los_pci (x86_64 only)

kernel/src/block.rs
    └── Uses: virtio-drivers::VirtIOBlk
    └── Uses: los_hal::VirtioHal

kernel/src/gpu.rs
    └── Uses: los_gpu crate
    └── Uses: los_pci
    └── Uses: boot::Framebuffer (Limine fallback)

crates/gpu/
    └── Uses: virtio-drivers::VirtIOGpu
    └── Provides: Gpu struct, Display DrawTarget
```

### Known Couplings

1. `los_hal::VirtioHal` - Used by all drivers, must stay in HAL
2. `los_pci::PciTransport` - Used by GPU + Input on x86_64
3. `virtio-drivers` crate - External dependency, used everywhere

## Constraints

1. **No behavior change** - All drivers must work identically after refactor
2. **Both architectures** - aarch64 and x86_64 must both work
3. **Incremental** - Can migrate one driver at a time
4. **No new dependencies** - Use existing `virtio-drivers` crate

## Design Decisions (Answered)

See `plan.md` for full rationale:

1. **Transport approach:** WRAP `virtio-drivers` transports (not replace)
2. **Testing support:** YES, driver crates support `std` via feature flag
3. **PCI for block/net:** DEFER to avoid scope creep

---

## Phase 1 Steps

### Step 1: Map Current Behavior and Boundaries

**File:** `phase-1-step-1.md`

Tasks:
- Document all public APIs in current driver files
- Map call sites for each driver
- Identify arch-specific code paths

### Step 2: Lock in Golden Tests

**File:** `phase-1-step-2.md`

Tasks:
- Run all tests, ensure passing
- Capture current golden log baselines
- Document manual verification steps

### Step 3: Increase Test Coverage Where Missing

**File:** `phase-1-step-3.md`

Tasks:
- Identify untested code paths (x86_64 block/net)
- Add integration tests for driver APIs
- Consider adding driver unit tests in new crate structure
