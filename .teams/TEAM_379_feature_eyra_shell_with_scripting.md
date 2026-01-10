# TEAM_379 — Feature Plan: Eyra Shell with Scripting

**Created:** 2026-01-10  
**Plan:** `docs/planning/eyra-shell/`  
**Status:** Planning

---

## Objective

Replace the current no_std `lsh` shell with an Eyra-based shell that supports:
- Full scripting (variables, loops, conditionals)
- Script file execution
- Modern shell features (tab completion, history, etc.)

## Research Findings

### Option 1: Ion Shell (Redox OS) ⭐ RECOMMENDED
- **Source:** https://github.com/redox-os/ion
- **Pros:**
  - Written in Rust for a Rust OS (Redox)
  - Already has `redox_syscall` support
  - Faster than Bash/Dash
  - Full scripting: variables, loops, conditionals, functions
  - Memory safe (no shellshock vulnerabilities)
- **Cons:**
  - Not POSIX/Bash compatible (different syntax)
  - May need adaptation for Eyra syscalls

### Option 2: Custom Shell with conch-parser
- Use conch-parser for POSIX parsing
- Build execution layer on Eyra
- More work, but Bash-compatible

### Option 3: Nushell
- Modern structured data shell
- Heavy dependencies, may be overkill

## Recommendation

**Use Ion Shell** - it's designed for exactly this use case (Rust OS shell). We can fork/adapt it to use Eyra instead of Redox syscalls.

---

## Plan Created

All phase files written to `docs/planning/eyra-shell/`:
- README.md — Overview and success criteria
- phase-1.md — Discovery (analyze Ion Shell)
- phase-2.md — Design (5 key questions to answer)
- phase-3.md — Implementation steps
- phase-4.md — Integration & Testing
- phase-5.md — Polish & Documentation

**Status:** Plan complete, ready for review and implementation.
