# TEAM_287: x86_64 Behavior Test Questions

## Q1: Limine Version Requirement

**Context**: The `limine` crate may have version-specific request formats.

**Question**: Should we require a specific minimum Limine version, or support multiple versions?

**Options**:
- A) Require Limine 7.x+ (current in ISO build)
- B) Support fallback for older Limine versions
- C) Document version requirement, fail fast if incompatible

**Recommendation**: Option C - Document and fail fast.

---

## Q2: ECAM Base Address Source

**Context**: PCI ECAM base address varies by platform.

**Question**: How should we determine the PCI ECAM base address on x86_64?

**Options**:
- A) Hardcode 0xB0000000 (standard, works for QEMU)
- B) Parse ACPI MCFG table for dynamic discovery
- C) Add boot parameter to configure

**Recommendation**: Option A for now, with TODO for ACPI MCFG in future.

---

## Q3: Golden File Content Scope

**Context**: The aarch64 golden file covers full boot through shell prompt.

**Question**: How much of the boot output should the x86_64 golden file cover?

**Options**:
- A) Full boot through shell prompt (like aarch64)
- B) Only until "System Ready" message
- C) Minimal - just confirm kernel boots

**Recommendation**: Option A - Full parity with aarch64 test.

---

## Q4: APIC Access Strategy

**Context**: APIC is at physical address 0xFEE00000, currently skipped for Limine boot.

**Question**: Should APIC/IOAPIC be accessed via HHDM or custom mapping?

**Options**:
- A) Always use HHDM for APIC (0xFEE00000)
- B) Set up dedicated APIC mapping after boot
- C) Skip APIC for now, use PIT timer

**Recommendation**: Option A - HHDM is simpler and works.

---

## Decision Log

| Question | Decision | Date | Rationale |
|----------|----------|------|-----------|
| Q1 | C - Fail fast | 2026-01-08 | Document version requirement, panic if incompatible |
| Q2 | A - Hardcode 0xB0000000 | 2026-01-08 | Standard QEMU ECAM, TODO for ACPI MCFG later |
| Q3 | A - Full boot | 2026-01-08 | Parity with aarch64 behavior test |
| Q4 | A - Use HHDM | 2026-01-08 | Simpler than custom mapping, works reliably |
