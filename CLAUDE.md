# CLAUDE.md

## ⚠️ MANDATORY: CREATE TEAM FILE FIRST ⚠️

**This is NOT optional. EVERY conversation MUST start with creating a team file.**

```bash
# Step 1: Find the next number
ls .teams/TEAM_*.md | tail -1

# Step 2: Create your team file IMMEDIATELY (before ANY other action)
# Format: .teams/TEAM_XXX_short-description.md
```

**No exceptions. No "I'll do it later." Create the team file NOW.**

---

## CRITICAL FAILURES TO AVOID

These are recurring mistakes. **DO NOT REPEAT THEM:**

### 1. Creating team files too late
**Wrong:** Start coding, then create team file when done
**Right:** Create team file BEFORE any code changes (THIS IS MANDATORY)

### 2. Making architecture decisions without asking
**Wrong:** "I'll add autologin to make it work"
**Right:** Ask user: "Should I add autologin or implement proper login?"

### 3. Putting code in wrong crates
**Wrong:** VM control in `builder` (it builds things, not runs things)
**Right:** VM control in `xtask` (dev tasks)

### 4. Long timeouts that waste user's time
**Wrong:** 5-minute timeout, 500ms sleeps everywhere
**Right:** Minimal timeouts (100-200ms), fail fast

### 5. Inventing workarounds instead of checking references
**Wrong:** Guess how PAM works, invent shell-wrapper
**Right:** `grep -rn "autologin" vendor/systemd/` first

### 6. Treating this as a toy OS
**Wrong:** Skip authentication, use root, take shortcuts
**Right:** This is a REAL OS - proper users, proper login, proper security

### 8. PUTTING REQUIRED FEATURES BEHIND FLAGS
**This is NOT acceptable. EVER.**

When a feature is REQUIRED for the system to work properly on modern hardware, it must work BY DEFAULT. Do not hide it behind a flag.

**Example of what NOT to do:**
- UEFI boot is required for most modern computers
- Wrong: `cargo run -- test --uefi` (flag required for UEFI)
- Right: ISO boots both BIOS and UEFI automatically

**The pattern:**
- Required features work by default
- Optional features get flags
- UEFI is NOT optional in 2026 - it's required
- Don't make users add flags just to use standard hardware

**If you're adding a flag, ask yourself:** "Is this actually optional, or am I just being lazy?"

---

### 7. DELETING DIRECTORIES WITHOUT CHECKING FOR VALUABLE GITIGNORED FILES
**This destroyed $12 of API costs, a full night of work, and a critical presentation.**

**What happened (2026-01-18):**

1. **Jan 16**: Claude added `training/` to `.gitignore` with comment "Generated datasets (from augment_data.py)" - treating it as regeneratable data

2. **Jan 17**: User spent ALL NIGHT generating thinking annotations via OpenAI API ($12). This data was saved to `python/training/training_with_thinking.jsonl` - which was GITIGNORED

3. **Jan 18**: Claude migrated `python/` to `llm-toolkit/` and Rust, then DELETED `python/` directory WITHOUT:
   - Checking `git status --ignored`
   - Asking what gitignored files existed
   - Verifying nothing valuable was in gitignored paths

4. **Result**: 7716 training examples with thinking annotations - LOST FOREVER. User had presentation next day, needed investment for rent.

**BEFORE deleting ANY directory, ALWAYS run:**
```bash
# Check what gitignored files exist
git status --ignored

# Or list all files including ignored
ls -la directory/
find directory/ -type f
```

**Gitignored ≠ worthless.** Generated data that cost time/money to create is VALUABLE even if gitignored.

**ASK THE USER** before deleting directories: "Are there any valuable files in here that aren't tracked by git?"

---

## RULE 1: CREATE TEAM FILE FIRST

**Before writing ANY code, create your team file:**

```bash
# Find highest number
ls .teams/TEAM_*.md | tail -1

# Create YOUR team file IMMEDIATELY
# Example: .teams/TEAM_481_your-task-description.md
```

Team file documents:
- What you're doing and why
- Decisions made
- Problems encountered
- How to fix properly (not workarounds)

**If you don't create a team file FIRST, you will forget context and make the same mistakes.**

## RULE 2: CHECK REFERENCES BEFORE CODING

**Every problem has already been solved.** Search `vendor/` before writing code:

```bash
grep -rn "your_problem" vendor/systemd/
grep -rn "your_problem" vendor/util-linux/
```

| Reference | Path | Use For |
|-----------|------|---------|
| systemd | `vendor/systemd/` | Init, services, getty |
| util-linux | `vendor/util-linux/` | agetty, login, PAM |
| brush | `vendor/brush/` | Shell |
| uutils | `vendor/uutils/` | Coreutils |

**Do NOT invent workarounds. Copy how real projects do it.**

---

## What is LevitateOS?

Linux distribution builder using Rust. Output: bootable initramfs.

## Commands

```bash
# Build ISO (in leviso/)
cd leviso
cargo run -- build        # Full build from scratch
cargo run -- initramfs    # Rebuild initramfs only
cargo run -- iso          # Rebuild ISO only

# Testing - Claude uses test, User uses run
cargo run -- test         # Claude: quick debug (terminal, direct kernel boot)
cargo run -- run          # User: real test (QEMU GUI, full ISO, UEFI)
cargo run -- run --bios   # User: same but BIOS boot
```

## Architecture

```
builder/     → Builds artifacts (kernel, initramfs)
xtask/       → Dev tasks (VM control, tests)
vendor/      → Reference implementations
.teams/      → Work history (CREATE YOURS FIRST)
```

## Separation of Concerns

- `builder` = build things
- `xtask` = run things

Never mix these.

---

## Website Submodule

The `website/` directory is a TanStack Start application.

### Dev Server

```bash
cd website
npm run dev      # Starts on http://localhost:3000
npm run typecheck  # TypeScript validation
npm run build    # Production build
```

**Port: 3000** (NOT 5173 - this is TanStack Start, not vanilla Vite)

### Structure

```
website/
├── src/
│   ├── components/
│   │   ├── docs/       → DocsPage template system
│   │   ├── layout/     → Header, Footer, DocsLayout
│   │   └── ui/         → shadcn components
│   ├── routes/
│   │   └── docs/       → Documentation pages (install, levitate, etc.)
│   └── styles.css
└── package.json
```

### Docs Template System

Documentation pages use a template-driven system (`components/docs/`):
- Content defined as structured data (`DocsContent` type)
- Single `DocsPage` component renders all content
- Supports: text, code blocks, tables, lists, conversations
- Inline markdown: \`code\`, **bold**, [links](url)
