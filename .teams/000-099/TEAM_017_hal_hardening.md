# TEAM_017 — HAL Hardening Implementation

**Created:** 2026-01-03
**Objective:** Bring LevitateOS HAL up to par with Redox kernel reference

## Status ✅ Complete
- [x] Planning complete
- [x] Implementation complete
- [x] Verification complete

## Scope
Based on TEAM_016 gap analysis, implementing all remaining improvements:

### Already Fixed (TEAM_015, TEAM_016)
- [x] GIC: Group0+Group1 enable (0x3)
- [x] GIC: `is_spurious()` helper

### Implemented (TEAM_017)
- [x] GIC: Use `is_spurious()` in exception handler (skip EOI for spurious)
- [x] UART: drain_fifo before enabling interrupts
- [x] UART: Configure FIFO interrupt levels (RX4_8, TX4_8)
- [x] Timer: Add `clear_irq()` method for explicit IRQ clearing
- [x] VHE detection: Documented as not needed (virtual timer works at EL1)

### Intentional Design Choices (No Change)
- Timer: 1 Hz tick rate (sufficient for current stage)
- UART: Single-byte read (simple and sufficient)
- GIC: Conservative memory barriers (more correct than Redox)
