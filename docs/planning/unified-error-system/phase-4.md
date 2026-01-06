# Phase 4: Implementation

**Bug:** Inconsistent error handling across LevitateOS  
**Author:** TEAM_151  
**Status:** Ready for Execution

---

## Overview

This phase contains all implementation work, broken into independent Units of Work (UoWs).

Each UoW can be executed by a single team in one session.

---

## UoW Summary

| UoW | File(s) | Task | Est. Lines | Dependencies |
|-----|---------|------|------------|--------------|
| 1 | `kernel/src/loader/elf.rs` | Add codes to ElfError | ~30 | None |
| 2 | `levitate-hal/src/fdt.rs` | Add codes to FdtError | ~15 | None |
| 3 | `levitate-hal/src/mmu.rs` | Create MmuError, replace strings | ~80 | None |
| 4 | `kernel/src/task/user_mm.rs` | Use MmuError | ~40 | UoW 3 |
| 5 | `kernel/src/task/process.rs` | Preserve inner errors in SpawnError | ~30 | UoW 1, 3 |
| 6 | `kernel/src/fs/*.rs` | Create FsError, replace strings | ~50 | None |
| 7 | `kernel/src/net.rs` | Add codes to NetError | ~20 | None |
| 8 | ~~`kernel/src/block.rs`~~ | ~~Remove panics~~ | ~~30~~ | ✅ DONE |

**Total remaining:** ~265 lines across 8 files

---

## Detailed UoWs

See individual UoW files for implementation details:
- `phase-4-uow-1.md` - ElfError codes
- `phase-4-uow-2.md` - FdtError codes
- `phase-4-uow-3.md` - MmuError (new type)
- `phase-4-uow-4.md` - user_mm.rs migration
- `phase-4-uow-5.md` - SpawnError preservation
- `phase-4-uow-6.md` - FsError (new type)
- `phase-4-uow-7.md` - NetError codes

---

## Execution Order

```
Independent (can run in parallel):
  UoW 1: ElfError
  UoW 2: FdtError  
  UoW 3: MmuError
  UoW 6: FsError
  UoW 7: NetError

Sequential (has dependencies):
  UoW 4: user_mm.rs (after UoW 3)
  UoW 5: SpawnError (after UoW 1, 3)
```

---

## Verification After Each UoW

1. `cargo build --release` passes
2. `cargo test -p levitate-hal -p levitate-utils -p xtask` passes
3. No new panics introduced (grep check)
4. Error codes are unique within subsystem

---

## Exit Criteria for Phase 4

- [ ] All UoWs complete
- [ ] All errors have numeric codes
- [ ] All `&'static str` errors replaced
- [ ] Build passes
- [ ] Tests pass
