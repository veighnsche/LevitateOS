# TEAM_359 — Eyra Syscalls (ppoll, tkill, pkey_alloc)

**Created:** 2026-01-09  
**Plan:** `docs/planning/eyra-syscalls/`  
**Status:** ✅ Phase 2 Complete — Awaiting Q1 Confirmation

## Objective

Implement the three syscalls required for Eyra/std to run on LevitateOS:
- `ppoll` (271 x86_64) — Poll with timeout
- `tkill` (200 x86_64) — Thread-directed signal
- `pkey_alloc` (302 x86_64) — Memory protection keys

## Problem Statement

The Eyra test (`cargo xtask test eyra`) revealed these missing syscalls. Without them, Eyra panics during initialization.

## Priority

| Syscall | Priority | Reason |
|---------|----------|--------|
| `ppoll` | **P0** | Used by std I/O, blocks execution |
| `tkill` | **P1** | Thread signals, needed for threading |
| `pkey_alloc` | **P2** | Memory protection keys, can stub |

## Progress Log

### 2026-01-09
- Team registered
- Beginning discovery phase
