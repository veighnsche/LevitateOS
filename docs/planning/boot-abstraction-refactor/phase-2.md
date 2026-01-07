# Phase 2: Structural Extraction — Boot Abstraction Layer

## Purpose
Extract a clean `BootInfo` abstraction that decouples the kernel from specific bootloaders, following UNIX philosophy principles.

---

## Target Design

### UNIX Philosophy Application

| Principle | Application |
|-----------|-------------|
| **Modularity (Rule 1)** | One crate/module per boot protocol parser |
| **Composition (Rule 2)** | `BootInfo` is a simple struct consumable by any subsystem |
| **Expressive Types (Rule 3)** | Enums for memory types, structs for regions |
| **Silence (Rule 4)** | Boot parsing produces no output on success |
| **Safety (Rule 5)** | All unsafe bootloader access wrapped in safe APIs |

### New Module Structure

```
kernel/src/boot/           # Boot abstraction (arch-agnostic)
├── mod.rs                 # BootInfo definition + unified entry
├── limine.rs              # Limine protocol → BootInfo
├── multiboot.rs           # Multiboot1/2 → BootInfo (legacy, for QEMU)
├── dtb.rs                 # Device Tree → BootInfo (AArch64)
└── stub.rs                # Minimal entry for direct QEMU -kernel

kernel/src/arch/x86_64/    # Arch-specific entry stubs
├── boot.S                 # Minimal stub (if keeping multiboot) OR deleted
└── boot.rs                # Calls boot::parse_*() → BootInfo

kernel/src/arch/aarch64/   # Arch-specific entry stubs  
├── asm/boot.S             # Entry point
└── boot.rs                # Calls boot::parse_dtb() → BootInfo
```

**Key Insight**: The `kernel/src/boot/` module is arch-agnostic and contains only
parsers and types. The arch-specific `boot.rs` files call into it and handle
the actual entry point mechanics.

### The BootInfo Contract

```rust
/// Unified boot information - the ONE interface between bootloader and kernel.
/// 
/// UNIX Rule 2: Designed for composition - any subsystem can consume this.
/// UNIX Rule 3: Type-safe, self-describing, no magic numbers.
#[derive(Debug)]
pub struct BootInfo {
    /// Physical memory map - regions available for kernel use
    pub memory_map: MemoryMap,
    
    /// Framebuffer for early console (optional)
    pub framebuffer: Option<Framebuffer>,
    
    /// Platform-specific firmware tables
    pub firmware: FirmwareInfo,
    
    /// Kernel command line
    pub cmdline: Option<&'static str>,
    
    /// Initial ramdisk location
    pub initramfs: Option<MemoryRegion>,
    
    /// Boot protocol that was used
    pub protocol: BootProtocol,
}

/// Memory map - array of typed regions
pub struct MemoryMap {
    pub regions: &'static [MemoryRegion],
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub kind: MemoryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Usable,           // Free RAM for kernel use
    Reserved,         // Firmware reserved
    AcpiReclaimable,  // Can be reclaimed after ACPI init
    Kernel,           // Where kernel is loaded
    Bootloader,       // Bootloader data (can reclaim)
    Framebuffer,      // Video memory
    Unknown,
}

/// Platform firmware info - architecture-specific
pub enum FirmwareInfo {
    /// x86_64: ACPI tables via RSDP
    Acpi { rsdp: *const u8 },
    
    /// AArch64: Device Tree Blob
    DeviceTree { dtb: *const u8 },
    
    /// No firmware info available
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum BootProtocol {
    Limine,
    Multiboot1,
    Multiboot2,
    DeviceTree,
    Direct,  // QEMU -kernel with no protocol
}

/// Framebuffer info for early console
#[derive(Debug)]
pub struct Framebuffer {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
}
```

### Unified Kernel Entry

```rust
// kernel/src/main.rs (both architectures)

/// THE kernel entry point - same signature for all architectures.
/// 
/// UNIX Rule 1: kernel_main does ONE thing - orchestrate kernel init.
/// The boot/ module handles all bootloader protocol translation.
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // All architectures, all bootloaders enter here with same interface
    init::run(boot_info)
}
```

---

## Extraction Strategy

### Principle: Parallel Paths, Not Big Bang

1. **Keep old boot path working** - QEMU testing continues
2. **Add new boot path alongside** - Limine support in parallel
3. **Migrate incrementally** - Switch default only when stable
4. **Delete old only at end** - Phase 4/5

### Coexistence Period

```
                  ┌─────────────────────────────────────┐
                  │          Bootloader Layer           │
                  ├─────────────────────────────────────┤
                  │  Limine  │ Multiboot │    DTB       │
                  │  (new)   │  (legacy) │  (existing)  │
                  └────┬─────┴─────┬─────┴──────┬───────┘
                       │           │            │
                       ▼           ▼            ▼
                  ┌─────────────────────────────────────┐
                  │         boot/mod.rs                 │
                  │    parse_*() → BootInfo             │
                  └────────────────┬────────────────────┘
                                   │
                                   ▼
                  ┌─────────────────────────────────────┐
                  │     kernel_main(&BootInfo)          │
                  │        (unified entry)              │
                  └─────────────────────────────────────┘
```

---

## Steps

### Step 1: Define BootInfo Types
**File**: `phase-2-step-1.md`

Create the type definitions in `kernel/src/boot/mod.rs`:
- `BootInfo` struct
- `MemoryMap` and `MemoryRegion`
- `FirmwareInfo` enum
- `BootProtocol` enum
- `Framebuffer` struct

**Exit Criteria**: Types compile, no consumers yet.

### Step 2: Implement Multiboot → BootInfo Parser
**File**: `phase-2-step-2.md`

Create `kernel/src/boot/multiboot.rs`:
- Parse Multiboot1 info structure
- Parse Multiboot2 tags (if needed)
- Convert to `BootInfo`

**Exit Criteria**: 
- Existing x86_64 boot can be converted to BootInfo
- No behavior change yet (just new code path)

### Step 3: Implement DTB → BootInfo Parser
**File**: `phase-2-step-3.md`

Create `kernel/src/boot/dtb.rs`:
- Extract memory info from DTB
- Convert to `BootInfo`

**Exit Criteria**:
- AArch64 DTB parsing produces BootInfo
- Existing AArch64 boot unaffected

### Step 4: Add Limine Support (New)
**File**: `phase-2-step-4.md`

Create `kernel/src/boot/limine.rs`:
- Add `limine` crate dependency (pin version for API stability)
- Define Limine requests (memory map, framebuffer, RSDP)
- Implement Limine → BootInfo conversion
- Create Limine entry point

**Cargo.toml Addition**:
```toml
[dependencies]
limine = "0.2"  # Pin to specific version - API changed significantly between versions
```

**Example limine.cfg**:
```
timeout: 3

/LevitateOS
    protocol: limine
    kernel_path: boot():/boot/levitate-kernel
    # Optional: initramfs
    # module_path: boot():/boot/initramfs.cpio
```

**Exit Criteria**:
- Kernel can boot via Limine in QEMU
- Produces same `BootInfo` as multiboot path

### Step 5: Create Unified kernel_main
**File**: `phase-2-step-5.md`

Modify entry points to use BootInfo:
- x86_64: `kernel_main(&BootInfo)` 
- AArch64: `rust_main(&BootInfo)`
- Keep old entry points as wrappers initially

**Exit Criteria**:
- Both architectures call same `kernel_main(&BootInfo)`
- All tests pass
- No behavior change

---

## Modular Refactoring Rules Checklist

Per `kernel-development.md` and Rule 7:

- [ ] Each boot parser is a separate module
- [ ] `BootInfo` has no arch-specific fields in core struct
- [ ] FirmwareInfo enum handles arch differences
- [ ] No deep relative imports (use `crate::boot::*`)
- [ ] Files < 500 lines (ideal), < 1000 (max)
- [ ] Private fields with intentional public API
