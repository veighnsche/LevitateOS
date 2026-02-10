# CLAUDE.md — AcornOS + IuppiterOS Consolidated Build

## ⛔ NEVER USE DNF (OR ANY SYSTEM PACKAGE MANAGER)

**Recipe IS the package manager.** NEVER use `dnf`, `apt`, `yum`, `pacman`, or any
system package manager to install dependencies. NEVER suggest the user install
packages with a system package manager. NEVER use `dnf download` to fetch RPMs.

If a tool is missing, recipe must handle it by:
1. **Building from source** via a recipe
2. **Adding a Rust helper** to the recipe binary itself
3. **Eliminating the need** — use a simpler approach that doesn't require the tool

dnf has been replaced with a guard script that blocks execution. This is intentional.

## Recipe Collection Philosophy

**Every dependency should be a recipe.** Even optional tools, even verification-only tools,
even "nice to have" utilities. The goal is a complete recipe collection that makes the
entire build reproducible from a bare Rocky Linux install with only gcc, make, and python3.

**When you need a new tool:**
1. Create a recipe in `distro-builder/recipes/` (if shared) or `leviso/deps/` (if leviso-specific)
2. Follow the pattern in `rocky-deps.rhai` or `leviso-deps.rhai` — download source, build, install to TOOLS_PREFIX
3. Wire it into the build via `let deps = ["recipe-name"]` in the consumer recipe
4. NEVER skip creating a recipe because "it's just one tool" — that's how dnf creep starts

**Existing recipe collection:**
- `rocky-deps.rhai` — aria2c, 7zz, unsquashfs (for Rocky ISO extraction)
- `linux-deps.rhai` — flex, bison, gcc, etc. (for kernel compilation)
- `leviso-deps.rhai` — mkfs.erofs, xorriso, mkfs.fat, mtools, ukify, isoinfo (for ISO building)

You are building **two Alpine-based OS variants** in a single Ralph loop.
Read the PRD and progress file every iteration.

## What You're Building

| | AcornOS (desktop) | IuppiterOS (appliance) |
|---|---|---|
| Purpose | Desktop-ready base system | Headless HDD refurbishment server |
| Base | Alpine Linux | Alpine Linux |
| Init | OpenRC | OpenRC |
| libc | musl | musl |
| Display | Serial console (desktop is post-install) | Serial console only (no display ever) |
| Network | WiFi + wired | Wired only |
| Packages | Full daily driver | Minimal (smartmontools, hdparm, sg3_utils) |
| Boot | UKI, interactive | UKI, serial console auto |
| Data | User home dirs | /var/data partition |

IuppiterOS is AcornOS with fewer packages and headless config. They share most code.

## Repository Structure

```
LevitateOS/
├── AcornOS/                   # AcornOS builder — WORKSPACE
├── IuppiterOS/                # IuppiterOS builder — WORKSPACE
├── distro-spec/
│   ├── src/acorn/             # AcornOS specs — MAY MODIFY
│   ├── src/iuppiter/          # IuppiterOS specs — MAY MODIFY
│   ├── src/shared/            # Shared — CAREFUL
│   └── src/levitate/          # LevitateOS — DO NOT TOUCH
├── distro-builder/            # Shared abstractions — MAY MODIFY
├── leviso/                    # LevitateOS builder — REFERENCE ONLY
├── testing/install-tests/     # E2E tests — USE TO GRADE
└── tools/                     # Shared tools
```

## Layer Boundaries (CRITICAL)

### You MAY modify:
- `AcornOS/` and `IuppiterOS/` — builder implementations
- `distro-spec/src/acorn/` and `distro-spec/src/iuppiter/` — variant specs
- `distro-builder/` — shared abstractions IF both variants benefit

### You MAY fix bugs in:
- `leviso/` — fix real bugs if you find them, but do NOT restructure

### You MUST NOT modify:
- `distro-spec/src/levitate/` — do NOT change LevitateOS specs
- `testing/install-tests/src/steps/` — do NOT change test assertions
- Anything that would break LevitateOS

### The rule: removing AcornOS and IuppiterOS must leave LevitateOS unbroken.

## Shared Infrastructure (USE IT — DO NOT REIMPLEMENT)

### distro-spec
Single source of truth for both variants. Already defines packages, services,
boot modules, UKI entries, paths, and signing keys for both acorn and iuppiter.
**Always pull constants from here. Never hardcode.**

### distro-builder
Shared build abstractions:
- `Installable` trait + `Op` enum — component installation
- `DistroConfig` trait — distro identification
- `artifact::erofs` — EROFS rootfs builder
- `artifact::cpio` — CPIO/initramfs builder
- `executor/` — directory, file, user operations

**Look at how leviso uses distro-builder. Mirror that pattern.**

### tools/ (Shared libraries — import via Cargo.toml)

| Tool | What it does | Used for |
|------|-------------|----------|
| **recstrap** | Extracts rootfs to disk (like pacstrap) | Installing OS to target partition |
| **recinit** | Builds initramfs (busybox for live) | Creating initramfs for ISO boot |
| **reciso** | Creates bootable UEFI ISO (xorriso) | Final ISO creation step |
| **recipe** | Rhai-based package orchestrator | Alpine APK dependency resolution |
| **recqemu** | QEMU command builder | Powers `cargo run -- run` |
| **recfstab** | Generates /etc/fstab | Post-install fstab |
| **recchroot** | Enters chroot with bind mounts | Post-install configuration |
| **recuki** | Builds Unified Kernel Images | UKI for systemd-boot |
| **leviso-elf** | ELF analysis + library copying | Initramfs library bundling |

## Known Issues (READ BEFORE STARTING)

### Install-Tests Boot Detection is Broken (TEAM_154)

The automated install-tests runner fails during initial boot detection (before Phase 1).
**This is a test harness I/O buffering issue, NOT an ISO problem.**

- `wait_for_live_boot_with_context()` times out because Console I/O doesn't capture QEMU serial output
- Manual testing confirms the ISO boots perfectly
- See `.teams/TEAM_154_install-tests-broken-boot-detection.md` for details

**What this means for you:**
- Do NOT waste iterations trying to make install-tests pass if boot detection fails
- If you get "BOOT STALLED: No output received", mark the install-test task BLOCKED and move on
- You CAN fix the Console I/O buffering issue (in `testing/install-tests/src/qemu/serial/mod.rs`) — that's legitimate
- Use manual QEMU boot testing (`cargo run -- run` / `cargo run -- run --serial`) to verify your work

### Phase 6 Post-Reboot Tests

Phase 6 (post-reboot verification) in install-tests has been broken for a long time.
Do not expect Phase 6 to pass. Focus on Phases 1-5.

## Checkpoint Development Loop (PREFERRED)

Use checkpoints for incremental verification. Each gates the next.

```bash
# Run a single checkpoint
cd testing/install-tests && cargo run --bin checkpoints -- --distro acorn --checkpoint 1

# Run all checkpoints up to N
cd testing/install-tests && cargo run --bin checkpoints -- --distro acorn --up-to 3

# Check status
cd testing/install-tests && cargo run --bin checkpoints -- --distro acorn --status

# Reset after rebuild
cd testing/install-tests && cargo run --bin checkpoints -- --distro acorn --reset
```

| # | Name | Validates |
|---|------|-----------|
| 1 | Live Boot | ISO boots in QEMU |
| 2 | Live Tools | recstrap, recfstab, etc. present |
| 3 | Installation | Full scripted install to disk |
| 4 | Installed Boot | Boot from disk after install |
| 5 | Automated Login | Harness can login + run commands |
| 6 | Daily Driver Tools | sudo, ip, ssh, etc. present |

State is in `.checkpoints/{distro}.json` (gitignored). Auto-resets when ISO mtime changes.

## How to Test

```bash
# Build
cd AcornOS && cargo run -- build
cd IuppiterOS && cargo run -- build

# Checkpoints (PRIMARY — incremental, fast feedback)
cd testing/install-tests && cargo run -- --distro acorn --checkpoint 1
cd testing/install-tests && cargo run -- --distro acorn --up-to 3
cd testing/install-tests && cargo run -- --distro acorn --status
cd testing/install-tests && cargo run -- --distro acorn --reset

# Boot (manual verification)
cd AcornOS && cargo run -- run
cd IuppiterOS && cargo run -- run --serial

# Full 24-step flow (advanced — may fail on boot detection, see Known Issues)
cd testing/install-tests && cargo run --bin install-tests -- run --distro acorn
cd testing/install-tests && cargo run --bin install-tests -- run --distro iuppiter
# Add --experimental to include Phase 6 (post-reboot, known broken)
```

## Token Efficiency (CRITICAL)

**Every token costs money. Do not waste tokens on searches the compiler can do for free.**

- **Use the compiler to find call sites.** When renaming, moving, or deleting code, just make
  the change and run `cargo check`. The compiler errors ARE your search results — complete,
  accurate, and zero-cost. Do NOT grep/search for call sites first.
- **NO backwards compatibility. Ever.** No re-exports, no shims, no wrappers. BC is tech debt.
  Break the code, run `cargo check`, fix every error the compiler shows you. Done.
- **Do not read files you don't need.** If you already know what to write, write it. If you
  need to verify a function signature, read that one file — not five.
- **Do not speculatively search.** If you're about to run 3 greps "just in case", stop. Make
  the change, let the compiler tell you what broke.
- **Prefer `cargo check` over grep** for understanding code relationships. It's faster, it's
  complete, and it doesn't burn tokens on output you have to read and interpret.

## Timeout Awareness

- If a command produces no output for 2+ minutes, kill it and move on.
- Prefer `cargo check` over `cargo build` for verification.
- Commit BEFORE starting long-running operations.
- Match verification to the task: `.rs` change → `cargo check`. Boot config → full build + QEMU.

## Commit Rules

- Commit after EVERY meaningful change
- Format: `feat(acorn): ...` / `fix(iuppiter): ...` / `feat(shared): ...`
- Commit inside the relevant submodule first
- Run `cargo check` before committing

## Team Files (REQUIRED)

After each iteration, create or update a team file in `.teams/` to document your work:

**File naming:** `TEAM_NNN_short-description.md` where NNN is the next available number.
Check existing files: `ls .teams/TEAM_*.md | tail -5` to find the next number.

**Team file must contain:**
- Date and status
- What you implemented or fixed
- Key decisions and why
- Files modified
- Any blockers or known issues

**One team file per logical unit of work.** If you implement task 1.5 (IuppiterOS init),
create `TEAM_155_iuppiter-crate-init.md`. If you fix a bug found during review,
create `TEAM_156_fix-whatever.md`.

## Progress Tracking

After each iteration:
1. Update `.ralph/progress.txt` with what you did
2. Update `.ralph/learnings.txt` with patterns/gotchas
3. Create/update a team file in `.teams/` (see above)
4. If ALL PRD items are [x] and tests pass, output: <promise>COMPLETE</promise>
