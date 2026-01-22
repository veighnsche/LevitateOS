# CLAUDE.md

## What is LevitateOS?

**LevitateOS is a daily driver Linux distribution competing with Arch Linux.**

| | Arch | LevitateOS |
|---|------|------------|
| Target user | Power users who want control | Same |
| Philosophy | Minimal base, user builds up | Same |
| Package manager | pacman + AUR | recipe (Rhai-scripted) |
| Binary source | Compiled from source | Extracted from Fedora/Rocky RPMs |
| Build time | Hours (compiling) | Minutes (extracting) |

We are NOT:
- An embedded OS (too small = missing things)
- A container base image
- A server-only distro
- A resource-constrained system

We ARE:
- A daily driver desktop/workstation OS
- Competing directly with Arch Linux
- Minimal by default, complete via `recipe install`
- **For modern hardware with a local LLM**

---

## ⛔ STOP. READ. THEN ACT.

**This is the most important rule. Read it before every action.**

Every time you think you know where something goes - **stop. Read first.**

Every time you think something is worthless and should be deleted - **stop. Read it first.**

Every time you're about to write code - **stop. Read what already exists first.**

The five minutes you spend reading will save hours of cleanup, and save the person reviewing your work from wanting to throw their laptop out the window.

You're not paid to type fast. You're paid to do it right.

### Why this exists

On 2026-01-21, a Claude instance:
1. Was told to fix tests in the `install-tests/` crate
2. Never read that crate
3. Created 500+ lines of code in the WRONG location (`leviso/tests/`)
4. Then deleted that code without checking if it had useful improvements
5. Lost all the work

**The cost:** Money (API tokens aren't free), time (hours wasted), trust (eroded), and real emotional harm to the developer.

**The fix:** STOP. READ. THEN ACT. Every single time.

---

## System Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 8 GB | 16+ GB |
| Storage | 64 GB SSD | 256+ GB NVMe |
| CPU | x86-64-v3 (Haswell 2013+) | Recent AMD/Intel |
| GPU | Any | For LLM acceleration |

**QEMU testing uses 4GB RAM minimum. Never use toy values like 512MB.**

---

## ⚠️ FIRST: Create Team File

```bash
ls .teams/TEAM_*.md | tail -1  # Find next number
# Create: .teams/TEAM_XXX_short-description.md
```

**Do this BEFORE any code changes. No exceptions.**

---

## Critical Rules

### 1. Required features = ON by default
Wrong: `--uefi` flag for required UEFI boot
Right: UEFI default, `--bios` flag to opt out

### 2. Ask "What does archiso do?"
LevitateOS competes with Arch Linux. The live ISO experience should match archiso behavior:
- **Autologin**: archiso has autologin → LevitateOS live ISO has autologin
- **Root shell**: archiso boots to root shell → LevitateOS boots to root shell
- **Installer**: archiso requires manual install → LevitateOS uses `recstrap` (our archinstall equivalent)

When making UX decisions about the live ISO, CHECK archiso first. Don't invent different behavior.

### 3. Ask before architecture decisions
Don't silently add workarounds. Ask first.

### 4. Question "is this necessary?" BEFORE building
Don't build solutions without questioning assumptions. Ask "why do we need this?"
**Costly example (TEAM_075):** Built entire bootstrap system (busybox, 34 recipes, static binaries) before realizing the live ISO already provides everything needed. Recipe can install directly to /mnt. No bootstrap tarball necessary. Wasted tokens = wasted money.

### 5. Check vendor/ before inventing solutions
```bash
grep -rn "your_problem" vendor/systemd/
```

### 6. No false positives in tests
Never move missing items from CRITICAL to OPTIONAL just to pass tests.
If users need it → test fails when missing. No "optional" trash bin.

### 7. Before deleting directories
```bash
git status --ignored  # Check for valuable gitignored files
```
ASK before deleting. Gitignored ≠ worthless.

---

## Commands

```bash
cd leviso
cargo run -- build      # Full build
cargo run -- initramfs  # Rebuild initramfs only
cargo run -- iso        # Rebuild ISO only
cargo run -- test       # Claude: quick debug (terminal)
cargo run -- run        # User: full test (QEMU GUI, UEFI)
```

## Architecture

```
builder/  → Builds artifacts (kernel, initramfs)
xtask/    → Dev tasks (VM control, tests)
vendor/   → Reference implementations (systemd, util-linux, brush, uutils)
.teams/   → Work history
```

`builder` = build things | `xtask` = run things. Never mix.

---

## Website (submodule)

```bash
cd website
bun install && bun run dev    # http://localhost:3000
bun run build                 # Production build
```

Astro 5.7 static site, Tailwind 4, Shiki highlighting, 20 pages.
