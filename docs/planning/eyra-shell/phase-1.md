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

## Expected Outputs

1. **brush Architecture Document** — Crate breakdown
2. **Syscall Gap Analysis** — Missing syscalls list
3. **Eyra Compatibility Report** — What works, what doesn't
4. **Effort Estimate** — How much work to port

## Success Criteria

- [ ] brush source code analyzed
- [ ] Syscall requirements documented
- [ ] Eyra compatibility assessed
- [ ] Clear path forward identified
