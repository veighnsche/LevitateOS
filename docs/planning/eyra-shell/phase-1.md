# Phase 1: Discovery

## Objective

Understand brush shell's architecture and determine what's needed to run it on Eyra/LevitateOS.

## Current State Analysis

### Current Shell (lsh)
- Location: `crates/userspace/shell/`
- Type: no_std binary using libsyscall
- Features: Basic REPL, builtins (echo, help, clear, exit), external command execution
- Limitations: No scripting, no variables, no control flow

### brush Shell (Target)
- Source: https://github.com/reubeno/brush
- Type: std Rust binary (POSIX/Bash compatible)
- OS Support: Linux, macOS, WSL (Windows experimental)
- Key Dependencies:
  - **reedline** — Line editor (from Nushell) ✅
  - **clap** — Command parsing
  - **tokio** — Async runtime
  - **nix** — POSIX APIs
  - **fancy-regex** — Regex support
- Test Coverage: 900+ test cases

## Tasks

### Step 1: Clone and Analyze brush
- [ ] Clone brush repository
- [ ] Identify core crates and their responsibilities
- [ ] Map dependencies to Eyra equivalents
- [ ] Check tokio async requirements

### Step 2: Identify Syscall Requirements
- [ ] List syscalls brush uses (via nix crate)
- [ ] Compare to LevitateOS implemented syscalls
- [ ] Identify gaps that need implementation

### Step 3: Identify Eyra Compatibility
- [ ] Test if brush compiles with Eyra
- [ ] Identify std features brush requires
- [ ] Check if Eyra provides those features
- [ ] Assess tokio/async compatibility

## Critical: Tokio Async Runtime

brush uses **tokio** for async operations. This is the biggest porting challenge.

### Tokio Syscall Requirements
| Syscall | Purpose | Kernel Status |
|---------|---------|---------------|
| `epoll_create1` | Event polling | ❌ **NOT IMPLEMENTED** |
| `epoll_ctl` | Register events | ❌ **NOT IMPLEMENTED** |
| `epoll_wait` | Wait for events (timeout param for timers) | ❌ **NOT IMPLEMENTED** |
| `eventfd2` | Inter-thread signaling | ❌ **NOT IMPLEMENTED** |
| `pipe2` | Async pipes | ✅ Available |

**Note:** `timerfd_*` syscalls are NOT required. Tokio implements timers in userland using a hierarchical timer wheel, not OS timerfd. (TEAM_393 investigation)

**Resolution**: ✅ Implement epoll syscalls in kernel (Phase 0)

This unblocks tokio AND benefits future async applications.

## Expected Outputs

1. **brush Architecture Document** — Crate breakdown
2. **Syscall Gap Analysis** — Missing syscalls list (especially epoll/tokio)
3. **Eyra Compatibility Report** — What works, what doesn't
4. **Tokio Compatibility Report** — Can async runtime work?
5. **Effort Estimate** — How much work to port

## Success Criteria

- [ ] brush source code analyzed
- [ ] Syscall requirements documented (including tokio/epoll)
- [ ] Eyra compatibility assessed
- [ ] Tokio async runtime compatibility verified or workaround identified
- [ ] Clear path forward identified
