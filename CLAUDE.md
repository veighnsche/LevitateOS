# CLAUDE.md — AcornOS + IuppiterOS Consolidated Build

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

## How to Test

```bash
# Build
cd AcornOS && cargo run -- build
cd IuppiterOS && cargo run -- build

# Boot (manual verification — more reliable than install-tests)
cd AcornOS && cargo run -- run
cd IuppiterOS && cargo run -- run --serial

# Install-tests (may fail on boot detection — see Known Issues above)
cd testing/install-tests && cargo run --bin install-tests -- run --distro acorn
cd testing/install-tests && cargo run --bin install-tests -- run --distro iuppiter
```

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
