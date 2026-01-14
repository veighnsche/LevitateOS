# CLAUDE.md

## CRITICAL FAILURES TO AVOID

These are recurring mistakes. **DO NOT REPEAT THEM:**

### 1. Creating team files too late
**Wrong:** Start coding, then create team file when done
**Right:** Create team file BEFORE any code changes

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
# Build
cargo run --bin builder -- initramfs

# VM (use xtask, NOT builder)
cargo xtask vm start
cargo xtask vm stop
cargo xtask vm send "command"
cargo xtask vm log

# Quick run
./run.sh              # GUI
./run-term.sh         # Serial
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
