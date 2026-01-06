# TEAM_152: Implement Unified Error System Plan

**Created:** 2026-01-06  
**Task:** Review and implement the unified-error-system plan  
**Status:** Complete

---

## Plan Location

`docs/planning/unified-error-system/`

## Files Reviewed

- `plan.md` - Overview (outdated by detailed phases)
- `phase-1.md` - Understanding and Scoping ✅
- `phase-2.md` - Root Cause Analysis ✅
- `phase-3.md` - Fix Design (canonical subsystem allocation)
- `phase-4.md` - Implementation overview
- `phase-4-uow-1.md` through `phase-4-uow-7.md` - Individual UoWs
- `phase-5.md` - Cleanup and Handoff

---

## Review Findings

### Phase 1: Questions and Answers Audit

No `.questions/` files for this plan. Open questions in phase-1.md are answered inline:

| # | Question | Answer | Reflected in Plan? |
|---|----------|--------|-------------------|
| 1 | Syscall errno stay negative? | Yes - ABI | ✅ phase-1 line 68 |
| 2 | Error context depth? | Single level | ✅ phase-1 line 69 |
| 3 | New crate or inline? | Inline first | ✅ phase-1 line 70 |

**Result:** All answered questions reflected correctly.

---

### Phase 2: Scope and Complexity Check

#### Metrics
- **5 phases** (appropriate for this scope)
- **7 UoWs** in Phase 4 (appropriate granularity)
- **~265 lines** estimated remaining

#### Overengineering Signals
- ❌ None found - plan is appropriately sized

#### Oversimplification Signals
- ⚠️ **MINOR**: `levitate-virtio/src/hal_impl.rs` panics mentioned in phase-2 Priority 1 but no UoW created
- ⚠️ **MINOR**: plan.md subsystem allocation is outdated vs phase-3.md canonical list

---

### Phase 3: Architecture Alignment

#### Canonical Subsystem Allocation
User confirmed `phase-3.md:34-108` is the canonical list. Verified alignment:

| Subsystem | plan.md | phase-3.md (canonical) | Status |
|-----------|---------|------------------------|--------|
| Core | 0x00xx | 0x00xx | ✅ Match |
| MMU | 0x01xx | 0x01xx | ✅ Match |
| ELF | 0x02xx | 0x02xx | ✅ Match |
| Process | 0x03xx | 0x03xx | ✅ Match |
| Syscall | 0x04xx | 0x04xx | ✅ Match |
| FS | 0x05xx | 0x05xx | ✅ Match |
| Block | 0x06xx | 0x06xx | ✅ Match |
| Net | 0x07xx | 0x07xx | ✅ Match |
| GPU | 0x08xx | 0x08xx | ✅ Match |
| FDT | 0x09xx | 0x09xx | ✅ Match |
| PCI | 0x0Axx | 0x0Axx | ✅ Match |
| VirtIO | 0x0Bxx | 0x0Bxx | ✅ Match |

**Result:** Allocations consistent. phase-3.md is more comprehensive.

#### Pattern Consistency
`BlockError` (TEAM_150) established the pattern. All UoWs follow it:
- `code() -> u16`
- `name() -> &'static str`
- `Display` impl: `E{code:04X}: {name}`
- `Error` trait impl

**Result:** Pattern is consistent across all UoWs.

---

### Phase 4: Global Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 6 (Robust Error Handling) | ✅ | Core goal of this plan |
| Rule 14 (Fail Loud) | ✅ | Keeps invariant panics, removes recoverable panics |
| Rule 4 (Regression Protection) | ✅ | Phase 5 includes golden test verification |
| Rule 5 (Breaking Changes) | ✅ | Plan accepts breaking API changes, callers updated |

---

### Phase 5: Verification and References

#### Code Verification
- `block.rs` BlockError pattern verified - matches plan exactly
- Dependency order in phase-4.md is correct (leaf modules first)

#### Documentation Gaps
- ⚠️ **MINOR**: plan.md should reference phase-3.md as canonical for subsystem allocation

---

## Recommendations

### Critical (Must Fix)
None.

### Suggested Improvements

1. **Add VirtIO hal_impl.rs UoW** (or explicitly defer)
   - Phase-2 Priority 1 lists it but no UoW exists
   - Decision: Create UoW 8 or document deferral

2. **Update plan.md subsystem table**
   - Current table is subset of phase-3.md canonical list
   - Either update or add note: "See phase-3.md for complete allocation"

3. **Clarify UoW 4 variant mapping**
   - 12 string errors mapped to only 3-4 MmuError variants
   - Some mappings lose specificity (e.g., "Cannot allocate zero bytes" → InvalidVirtualAddress)
   - Consider: Add `ZeroSize` variant to MmuError?

---

## Verdict

**APPROVED with minor suggestions.**

The plan is:
- ✅ Well-structured (5 phases, 7 UoWs)
- ✅ Not overengineered (inline approach vs new crate)
- ✅ Not oversimplified (covers all identified issues)
- ✅ Follows established pattern (BlockError)
- ✅ Has clear reversal strategy
- ✅ Has verification steps

Ready for implementation.

---

## Implementation Progress

### Completed UoWs

| UoW | File(s) | Task | Status |
|-----|---------|------|--------|
| 1 | `kernel/src/loader/elf.rs` | Add codes to ElfError | ✅ Done |
| 2 | `levitate-hal/src/fdt.rs` | Add codes to FdtError | ✅ Done |
| 3 | `levitate-hal/src/mmu.rs` | Create MmuError | ✅ Done |
| 4 | `kernel/src/task/user_mm.rs` | Migrate to MmuError | ✅ Done |
| 5 | `kernel/src/task/process.rs` | Preserve inner errors in SpawnError | ✅ Done |
| 6 | `kernel/src/fs/*.rs` | Create FsError | ✅ Done |
| 7 | `kernel/src/net.rs` | Add codes to NetError | ✅ Done |

### Error Code Allocation Verified

| Subsystem | Range | Codes |
|-----------|-------|-------|
| MMU | 0x01xx | 5 |
| ELF | 0x02xx | 9 |
| Process | 0x03xx | 3 |
| FS | 0x05xx | 7 |
| Block | 0x06xx | 4 |
| Net | 0x07xx | 3 |
| FDT | 0x09xx | 2 |

---

## Handoff

- [x] Build passes
- [x] All tests pass (37 tests)
- [x] No duplicate error codes
- [x] Review complete
- [x] Implementation complete
- [x] Findings documented
