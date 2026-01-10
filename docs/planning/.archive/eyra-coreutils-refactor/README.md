# Eyra Coreutils Refactor Plan

**TEAM_376** | Created: 2026-01-10  
**Reviewed by:** TEAM_377 | 2026-01-10 | ✅ **APPROVED**

## Problem Summary

The Eyra coreutils workspace has severe structural issues:

1. **5.1GB of stale target folders** - Each utility has its own `target/` from standalone builds
2. **Duplicate Cargo.lock files** - 15 per-utility locks vs 1 workspace lock
3. **Duplicate .cargo folders** - Per-utility configs vs workspace config
4. **Tests don't test coreutils** - Only internal std library is tested

## Phases

| Phase | Description | Status |
|-------|-------------|--------|
| [Phase 1](phase-1.md) | Discovery and Cleanup | ✅ Complete (TEAM_378) |
| [Phase 2](phase-2.md) | Consolidate Build Config | ✅ Complete (TEAM_378) |
| [Phase 3](phase-3.md) | Real Coreutils Testing | ✅ Complete (TEAM_378) |
| [Phase 4](phase-4.md) | Cleanup and Hardening | ✅ Complete (TEAM_378) |

## Quick Start (Phase 1 - Immediate Cleanup)

```bash
# Remove 5.1GB of stale target folders
find crates/userspace/eyra -mindepth 2 -name "target" -type d -exec rm -rf {} +

# Remove per-utility Cargo.lock (workspace lock is authoritative)
find crates/userspace/eyra -mindepth 2 -name "Cargo.lock" -delete

# Remove per-utility .cargo folders
find crates/userspace/eyra -mindepth 2 -name ".cargo" -type d -exec rm -rf {} +

# Verify build still works
./run-test.sh
```

## Target State

```
crates/userspace/eyra/
├── .cargo/config.toml      ← Single workspace config
├── Cargo.toml              ← Workspace manifest
├── Cargo.lock              ← Single lock file
├── target/                 ← Single shared target
├── rust-toolchain.toml     ← Pinned toolchain
├── eyra-test-runner/       ← Tests actual coreutils
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
├── cat/
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
└── ... (other utilities, no target/ or Cargo.lock)
```

## Units of Work Estimate
- Phase 1: 1 UoW (30 min)
- Phase 2: 1 UoW (30 min)
- Phase 3: 3 UoW (2-3 hours)
- Phase 4: 1 UoW (30 min)

**Total: ~4-5 hours**
