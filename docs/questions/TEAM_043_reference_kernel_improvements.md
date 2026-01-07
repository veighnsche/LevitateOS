# Questions: Reference Kernel Improvements

**Team**: TEAM_043  
**Feature**: Reference Kernel Improvements  
**Phase**: Phase 2 (Design)  
**Status**: ✅ ANSWERED (2026-01-04)

---

## Q1: FDT Crate Version

**Context**: Need to add FDT parsing capability to levitate-hal.

**Options**:
- **A)** Use `fdt = "0.1"` - simpler API, fewer features
- **B)** Use `fdt = "0.2"` - more features, slightly more complex

**Answer**: **B - Use latest `fdt` crate (currently 0.1.5)**

**Rationale**: 
- The `repnop/fdt` crate is the standard Rust no_std FDT parser
- Used by major projects (Redox, Hermit, etc.)
- Has ergonomic API: `fdt.find_compatible(&["arm,gic-v3"])`
- Pixel 6 Tensor SoC has complex device tree - need full features

---

## Q2: GIC Fallback Strategy

**Context**: When FDT doesn't contain GIC information (rare but possible).

**Options**:
- **A)** Panic with error message - strict, ensures proper config
- **B)** Fall back to hardcoded addresses - maintains backward compatibility

**Answer**: **B - Fall back to hardcoded QEMU virt addresses**

**Rationale**:
- Pixel 6 emulation requires GICv3, but development/testing uses GICv2
- QEMU virt without explicit FDT should still work
- Hardcoded fallback: GICD=0x08000000, GICC=0x08010000, GICR=0x080A0000
- Real Pixel 6 will always have FDT from bootloader

---

## Q3: Handler Registration Timing

**Context**: When should IRQ handlers be registered relative to GIC init?

**Answer**: **B - After GIC init**

**Rationale**:
- GIC must be initialized to configure interrupt routing
- ARM best practice: init distributor → init CPU interface → register handlers
- Linux kernel does this: `gic_of_init()` before `request_irq()`
- Enables proper priority and target configuration

---

## Q4: bitflags Feature Flags

**Context**: The bitflags crate has optional features.

**Answer**: **B - Use `default-features = false`**

**Rationale**:
- Kernel is no_std environment
- `bitflags = { version = "2.4", default-features = false }`
- No heap allocation, no std dependency
- Standard practice for embedded Rust

---

## Q5: Implementation Priority

**Context**: Which improvements are most critical vs nice-to-have?

**Answer**: **B - FDT + GICv3 critical, others optional**

**Rationale for Pixel 6 emulation**:
1. **FDT Parsing** - CRITICAL: Enables hardware discovery like real device
2. **GICv3 Detection** - CRITICAL: Pixel 6 Tensor has GICv3
3. **InterruptHandler Trait** - NICE: Cleaner code, not blocking
4. **bitflags!** - NICE: Code quality, not blocking  
5. **VHE Detection** - NICE: Optimization, not blocking

**Implementation order**: FDT → GICv3 → others as time permits

---

## Q6: FDT Source Location

**Context**: Where does the kernel get the FDT address?

**Answer**: **C - Both: try x0 first, then scan**

**Rationale**:
- ARM boot convention: x0 contains FDT pointer (if provided)
- QEMU virt passes FDT via x0 when using `-dtb` option
- Fallback scan catches cases where x0 isn't set
- Pixel 6 bootloader will pass FDT via x0

**Implementation**:
```rust
fn find_fdt() -> Option<&'static Fdt> {
    // 1. Check x0 from boot (saved in BSS)
    if BOOT_X0 != 0 && is_valid_fdt(BOOT_X0) {
        return Some(parse_fdt(BOOT_X0));
    }
    // 2. Scan memory for FDT magic
    scan_for_fdt(0x4000_0000..0x5000_0000)
}
```

---

## Summary for Phase 3

| Decision | Choice | Impact |
|----------|--------|--------|
| FDT crate | `fdt = "0.1"` (repnop) | Standard, well-tested |
| GIC fallback | Hardcoded addresses | Backward compatible |
| Handler timing | After GIC init | Correct ordering |
| bitflags | no_std mode | Kernel-safe |
| Priority | FDT + GICv3 first | Pixel 6 focus |
| FDT source | x0 + scan | Flexible |

**Phase 2 is now complete. Phase 3 can begin.**
