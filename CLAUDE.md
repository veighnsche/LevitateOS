# CLAUDE.md

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

### 2. Ask before architecture decisions
Don't silently add autologin/workarounds. Ask first.

### 3. Question "is this necessary?" BEFORE building
Don't build solutions without questioning assumptions. Ask "why do we need this?"
**Costly example (TEAM_075):** Built entire bootstrap system (busybox, 34 recipes, static binaries) before realizing the live ISO already provides everything needed. Recipe can install directly to /mnt. No bootstrap tarball necessary. Wasted tokens = wasted money.

### 4. Check vendor/ before inventing solutions
```bash
grep -rn "your_problem" vendor/systemd/
```

### 5. No false positives in tests
Never move missing items from CRITICAL to OPTIONAL just to pass tests.
If users need it → test fails when missing. No "optional" trash bin.

### 6. Before deleting directories
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
