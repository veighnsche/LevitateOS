# Phase 2 — Design

**Feature**: Reference Kernel Improvements  
**Team**: TEAM_043  
**Status**: ✅ COMPLETE (Questions answered 2026-01-04)  
**Parent**: phase-1.md

---

## Proposed Solution Overview

Implement 5 improvements in priority order:

1. **FDT Parsing** (foundation for everything else)
2. **GICv3 Detection** (uses FDT)
3. **InterruptHandler Trait** (cleaner IRQ registration)
4. **bitflags! Crate** (cleaner registers)
5. **VHE Detection** (timer optimization)

---

## Improvement 1: FDT Parsing

### API Design

```rust
// levitate-hal/src/fdt.rs

/// Parse FDT from memory address
pub fn parse(fdt_addr: usize) -> Result<Fdt, FdtError>;

/// Find node by compatible string
pub fn find_compatible(fdt: &Fdt, compatible: &[&str]) -> Option<FdtNode>;

/// Get reg property (base address + size)
pub fn get_reg(node: &FdtNode) -> Option<(usize, usize)>;

/// Get interrupt property
pub fn get_interrupt(node: &FdtNode, index: usize) -> Option<u32>;
```

### Data Model

```rust
pub struct Fdt<'a> {
    data: &'a [u8],
    root: FdtNode<'a>,
}

pub struct FdtNode<'a> {
    name: &'a str,
    properties: &'a [FdtProperty<'a>],
    children: &'a [FdtNode<'a>],
}

pub enum FdtError {
    InvalidMagic,
    InvalidVersion,
    NodeNotFound,
}
```

### Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Use existing `fdt` crate? | Yes, use `fdt` crate | Battle-tested, no_std compatible |
| Fallback if no FDT? | Use hardcoded defaults | QEMU may not always provide FDT |
| When to parse? | Early in kmain, before device init | Need info for device init |

---

## Improvement 2: GICv3 Detection via FDT

### Detection Flow

```
1. Parse FDT at boot
2. Search for compatible = ["arm,gic-v3"]
3. If found → use GICv3 API
4. Else search for compatible = ["arm,gic-400", "arm,cortex-a15-gic"]
5. If found → use GICv2 API
6. Else → panic (no GIC found)
```

### API Changes

```rust
// levitate-hal/src/gic.rs

/// Initialize GIC based on FDT
pub fn init_from_fdt(fdt: &Fdt) -> &'static Gic;

/// Get GIC addresses from FDT
fn parse_gic_fdt(fdt: &Fdt) -> GicConfig;

pub struct GicConfig {
    pub version: GicVersion,
    pub dist_base: usize,
    pub cpu_base: Option<usize>,   // GICv2 only
    pub redist_base: Option<usize>, // GICv3 only
}
```

### Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| What if FDT has no GIC? | Panic with error | GIC is required for operation |
| What if both v2 and v3 compatible? | Prefer v3 | More capable |
| Store GIC reference how? | Static with OnceCell | Thread-safe, init once |

---

## Improvement 3: InterruptHandler Trait

### Trait Design

```rust
// levitate-hal/src/irq.rs

pub trait InterruptHandler: Send + Sync {
    /// Called when interrupt fires
    fn handle(&self, irq: u32);
    
    /// Optional: called during registration
    fn on_register(&self, _irq: u32) {}
}

/// Register a handler for an IRQ
pub fn register_handler(irq: u32, handler: &'static dyn InterruptHandler);

/// Dispatch to registered handler
pub fn dispatch(irq: u32) -> bool;
```

### Migration Path

| Current | New |
|---------|-----|
| `fn timer_irq_handler()` | `impl InterruptHandler for TimerHandler` |
| `fn uart_irq_handler()` | `impl InterruptHandler for UartHandler` |
| `gic::register_handler(IrqId, fn())` | `irq::register_handler(irq, &TIMER_HANDLER)` |

### Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Handler as fn ptr or trait? | Trait object | More flexible, can hold state |
| Static or Box<dyn>? | Static reference | No heap allocation in IRQ path |
| Multiple handlers per IRQ? | No, one handler | Simpler, matches hardware |

---

## Improvement 4: bitflags! Crate

### Usage Pattern

```rust
// levitate-hal/src/timer.rs

bitflags! {
    pub struct TimerCtrl: u32 {
        const ENABLE = 1 << 0;
        const IMASK = 1 << 1;
        const ISTATUS = 1 << 2;
    }
}

// Usage
let ctrl = TimerCtrl::ENABLE | TimerCtrl::IMASK;
write_timer_ctrl(ctrl.bits());
```

### Modules to Update

| Module | Registers |
|--------|-----------|
| `gic.rs` | GICD_CTLR flags |
| `timer.rs` | Timer control flags |
| `mmu.rs` | Page flags (already has custom impl) |

### Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Add bitflags dependency? | Yes | Widely used, no_std compatible |
| Replace existing PageFlags? | No | Current impl works, low priority |
| Apply to all registers? | Start with timer/gic | Incremental adoption |

---

## Improvement 5: VHE Detection for Timer

### Detection Logic

```rust
// levitate-hal/src/timer.rs

/// Check if Virtualization Host Extensions are present
fn vhe_present() -> bool {
    // Read ID_AA64MMFR1_EL1.VH field
    let mmfr1: u64;
    unsafe { asm!("mrs {}, ID_AA64MMFR1_EL1", out(reg) mmfr1) };
    ((mmfr1 >> 8) & 0xF) != 0
}

/// Choose timer based on VHE
pub fn init() {
    if vhe_present() {
        // Use physical timer (CNTP_*)
        init_physical_timer();
    } else {
        // Use virtual timer (CNTV_*)
        init_virtual_timer();
    }
}
```

### Behavioral Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Default timer type? | Virtual | Works everywhere |
| When to check VHE? | Once at init | VHE doesn't change at runtime |
| Impact on existing code? | Minimal | Abstracted behind init() |

---

## Design Alternatives Considered

### FDT Parsing
- **Alternative**: Parse ACPI tables instead
- **Rejected**: ACPI less common on ARM, FDT is standard

### GIC Detection
- **Alternative**: Continue PIDR2 probing
- **Rejected**: Proven unreliable in testing

### InterruptHandler
- **Alternative**: Keep function pointers
- **Rejected**: No state, less type-safe

---

## Answered Questions (2026-01-04)

### Q1: FDT Crate Version
**Answer**: Use `fdt = "0.1"` (repnop/fdt crate)
- Standard Rust no_std FDT parser
- Ergonomic API: `fdt.find_compatible(&["arm,gic-v3"])`

### Q2: GIC Fallback Strategy  
**Answer**: Fall back to hardcoded QEMU virt addresses
- GICD=0x08000000, GICC=0x08010000, GICR=0x080A0000
- Maintains backward compatibility

### Q3: Handler Registration Timing
**Answer**: After GIC init
- ARM best practice: init distributor → init CPU interface → register handlers

### Q4: bitflags Feature Flags
**Answer**: Use `default-features = false` (no_std)
- `bitflags = { version = "2.4", default-features = false }`

### Q5: Implementation Priority
**Answer**: FDT + GICv3 critical, others optional
- Pixel 6 Tensor has GICv3 - must work

### Q6: FDT Source Location
**Answer**: Try x0 first, then scan memory
- ARM convention: x0 contains FDT pointer at boot

---

## Phase 2 Steps

### Step 1 — Draft Initial Design
**Status**: Complete (this document)

### Step 2 — Define Behavioral Contracts
**Status**: Complete (tables above)

### Step 3 — Review Design Against Architecture
**Status**: Ready
- FDT parsing fits as new HAL module
- Traits follow existing patterns
- No breaking changes to public API

### Step 4 — Finalize Design After Questions Answered
**Status**: Complete (Questions answered 2026-01-04)

---

## Next Phase

Once questions are answered, proceed to **Phase 3 — Implementation** with steps broken into UoWs for each improvement.
