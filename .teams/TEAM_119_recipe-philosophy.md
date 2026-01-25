# TEAM_119: Recipe Core Design Philosophy + Rocky Recipe Fix

## Status: COMPLETE

## Problem

The recipe package manager's design philosophy is "paper thin" - too loosely communicated. This enables reward hacking: agents find shortcuts that appear to complete tasks while violating core design intent.

**Example**: `resolve()` function cramming acquire + build + install logic into a single function. Task appeared complete, but violated the fundamental principle of phase separation for caching.

Reference: https://www.anthropic.com/research/emergent-misalignment-reward-hacking

## Goals

1. Embed design philosophy EVERYWHERE - make constraints thick, not thin
2. Remove `resolve()` lifecycle - it's a reward-hacking invitation
3. Rewrite `rocky.rhai` with proper phase separation
4. Update leviso to use `execute()` instead of `resolve()`

## Changes Made

### Part 1: Documentation (Design Philosophy)
- [x] `tools/recipe/CLAUDE.md` - Add design philosophy section
- [x] `tools/recipe/src/lib.rs` - Add philosophy to module docs
- [x] `tools/recipe/src/core/lifecycle/mod.rs` - Add philosophy comments

### Part 2: Remove resolve() Lifecycle
- [x] Delete `resolve()` from `lifecycle/mod.rs`
- [x] Delete `Commands::Resolve` from `bin/recipe.rs`
- [x] Remove `resolve()` method from `RecipeEngine` in `lib.rs`
- [x] Remove resolve() tests from lifecycle/mod.rs
- [x] Remove unused ValueEnum import

### Part 3: Rewrite rocky.rhai
- [x] Split monolithic `resolve()` into `acquire()` and `build()` phases
- [x] Each phase checks "already done?" first
- [x] Added `install()` phase for completeness

### Part 4: Update Leviso Integration
- [x] `leviso/src/resolve.rs` - Use `execute()` (via `recipe install`) and read output by convention
- [x] Added `get_output_path_by_convention()` for determining output paths

## The Core Invariant

```
Each phase is SEPARATE and IDEMPOTENT:
- acquire() → get sources, skip if already acquired
- build()   → transform sources, skip if already built
- install() → install to PREFIX, skip if already installed

WHY: Separation enables caching. Re-build without re-acquiring.
     Re-install without rebuilding. Each phase checks "already done?"
```

## Success Criteria

1. No monolithic functions - acquire/build/install are separate
2. Each phase idempotent - checks "already done?" first
3. Philosophy documented - in CLAUDE.md AND code comments
4. resolve() gone - removed from recipe system
5. leviso uses execute() - not resolve()
