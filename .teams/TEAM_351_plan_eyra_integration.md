# TEAM_351 — Eyra Integration Plan

**Created:** 2026-01-09  
**Status:** Planning

## Objective

Create a comprehensive plan to integrate Eyra (pure Rust `std` runtime) with LevitateOS and run a test binary.

## Context

- **TEAM_349** created discovery and prerequisite planning docs
- **TEAM_350** implemented all prerequisite syscalls
- This team creates the integration plan (phases 3-5)

## Planning Documents

| Phase | File | Purpose |
|-------|------|---------|
| 1 | `phase-1.md` | Discovery (TEAM_349) |
| 2 | `phase-2.md` | Prerequisites & Design (TEAM_349) |
| 3 | `phase-3.md` | Implementation - Build Eyra binary |
| 4 | `phase-4.md` | Integration & Testing |
| 5 | `phase-5.md` | Polish & Documentation |

## Progress

- [x] Review existing planning docs
- [x] Create phase-3.md (Implementation)
- [x] Create phase-4.md (Integration)
- [x] Create phase-5.md (Polish)

## Summary

The complete Eyra integration plan is now ready. Execute phases in order:

1. **Phase 3**: Build `eyra-hello` test binary
2. **Phase 4**: Boot LevitateOS and run the binary, debug failures
3. **Phase 5**: Document, cleanup, handoff

## Build Status: ✅ SUCCESS

**Solution found:** Use `nightly-2025-04-28` as specified in Eyra's rust-toolchain.toml

### Working Build

```bash
cd userspace/eyra-hello
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort
./target/x86_64-unknown-linux-gnu/release/eyra-hello
```

**Output:**
```
=== Eyra Test on LevitateOS ===
[OK] println! works
[OK] argc = 1
[OK] Instant::now() works
[OK] elapsed = 2.197µs
[OK] HashMap works (getrandom ok), value = 42
=== Eyra Test Complete ===
```

### Next Steps

1. **Static linking** - Currently dynamic; need static for LevitateOS
2. **LevitateOS integration** - Add to initramfs and test on kernel
