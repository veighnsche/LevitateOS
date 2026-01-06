# TEAM_164: Feature - Userspace Standard Library (ulib)

## Objective
Create a feature plan for Phase 10: The Userspace Standard Library (`ulib`).

## Context
- **Phase**: 10 (First major item in Part II: Userspace Expansion & Apps)
- **Specification**: `docs/specs/userspace-abi.md`
- **Prior Work**: Phase 8a-8d completed userspace foundation (EL0, syscalls, ELF loader, shell, spawn)

## Scope
Planning only - creating the feature plan for `ulib` to enable complex userspace applications.

## Key Components to Plan
1. **Global Allocator**: Userspace heap via `brk`/`sbrk`
2. **File Abstractions**: `File`, `OpenOptions`, `Metadata`
3. **Directory Iteration**: `ReadDir` via `sys_getdents`
4. **Buffered I/O**: `BufReader`, `BufWriter`
5. **Environment**: `args()` and `vars()` parsing
6. **Time**: `sys_time`, `sys_sleep`, `Instant`, `Duration`
7. **Error Handling**: Standard `io::Error` and `Result`

## Status
- [x] Planning documents created
- [x] Phase 1 (Discovery) complete
- [x] Phase 2 (Design) complete
- [x] Phase 3 (Implementation plan) complete

## Artifacts Created
- `docs/planning/ulib-phase10/phase-1.md` - Discovery
- `docs/planning/ulib-phase10/phase-2.md` - Design with 7 behavioral questions
- `docs/planning/ulib-phase10/phase-3.md` - Implementation plan with 9 steps, 15 UoWs
- `.questions/TEAM_164_ulib_design.md` - Questions awaiting user answers

## Next Steps
1. User answers Q1-Q7 in `.questions/TEAM_164_ulib_design.md`
2. Next team reads phase-3.md and begins Step 1 (Kernel sbrk)

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [x] Planning docs ready for implementation teams
