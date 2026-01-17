# TEAM_018: Website & Code Corrections

## Status: Complete

## Context
1. User concerned Sway/Wayland recipes might be mistaken as default
2. Website features were incorrect/misleading
3. Binary name needed change from `levitate` to `recipe`

## Changes Made

### Part 1: Clarify Test Recipes
- Renamed `.vm/recipes/` -> `.vm/example-recipes/`
- Added README explaining these are TEST/EXAMPLE only
- Updated docs/sway-e2e-test.md with prominent warning

### Part 2: Website Feature Corrections

**Feature 1:** Your Own Packages - KEPT (fine)

**Feature 2:** Package Manager
- Changed `levitate` -> `recipe` in name and examples
- Removed `levitate desktop # Sway stack` reference

**Feature 3:** LLM Recipe Assistant
- Renamed to "SmolLM3 Recipe Assistant"
- Added "from Hugging Face" and "runs on your machine"

**Feature 4:** REPLACED "Pure Wayland" (WRONG!)
- New: "Natural Language Installer"
- Description: conversational install, no memorizing commands

**Feature 5:** FIXED "musl + GNU"
- Old: "Unusual combo" (wrong - Void does this)
- New: "Choose Your Stack" - Standard (systemd+glibc) vs Minimal (runit+musl)

**Feature 6:** REPLACED "Reference-Driven" (lame)
- New: "Terminal First"
- Description: boots to login prompt like Arch, no desktop preinstalled

**Quick Start Section:**
- Removed `levitate desktop` step
- Changed to "You're done" with `recipe install` example

### Part 3: Binary Rename (levitate -> recipe)

**recipe/Cargo.toml:**
- Binary name: `levitate` -> `recipe`
- Source file: `levitate.rs` -> `recipe.rs`

**recipe/src/bin/recipe.rs:**
- Command name: `levitate` -> `recipe`
- Paths: `/usr/share/levitate/` -> `/usr/share/recipe/`
- Env var: `LEVITATE_RECIPE_DIR` -> `RECIPE_DIR`
- Removed `Desktop` command entirely

**xtask/src/vm.rs:**
- Updated all build/copy references
- Removed Sway desktop install steps

**install-arch.sh (both copies):**
- Updated binary references
- Updated directory paths
- Removed Sway instructions

**recipe/src/executor/context.rs:**
- Build dir: `levitate-build` -> `recipe-build`

## Key Points
- LevitateOS = minimal, boots to terminal (like Arch)
- NO Wayland/Sway by default
- Package manager is now called `recipe`
- 4 variants: Standard/Minimal x with/without LLM
- SmolLM3 from Hugging Face for AI features
