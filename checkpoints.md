# Checkpoints

Status matrix for `testing/install-tests` (`cargo run --bin checkpoints -- ...`).

## Checkpoint Semantics

Each stage (01–08) represents a deterministic, bootable system state.

A stage is not only a validation gate — it is a *spawn point*.
From any passed stage, the system must be rebuildable and bootable
so a human can enter that exact state for audit, debugging, or inspection.

Stage 00 is the only exception: it validates build capability only and does not
represent a bootable runtime state.

## Distro Behavior (Authoritative)

| Area | LevitateOS | RalphOS | AcornOS | iuppiterOS |
|---|---|---|---|---|
| Visibility | Public | Public | Public | Private/internal |
| ISO live env | Full | Minimal | Full | Minimal |
| Purpose | Stable Daily | R4D Sandbox | Snappy Daily | iuppter-dar |
| Steal apps from | Rocky DVD ISO | Rocky DVD ISO | Alpine Extended | Alpine Extended |
| Toolchain | glibc/systemd/GNU | glibc/systemd/GNU | musl/OpenRC/busybox | musl/OpenRC/busybox |
| Pre-installed |  |  |  |  |
| Package Manager | recipe | recipe | recipe | recipe |
| 08 release target | `ISO` + `qcow2` + `.img` | `qcow2` | `ISO` + `qcow2` + `.img` | Private `.img` |

## Stages

| Stage | Ladder Semantics (Proven Authority) | Game Savepoint Semantics (Spawnable State) |
|---|---|---|
| 00-Build | Kernel + ISO build succeeds. | Not spawnable (build only). |
| 01-Boot | Live ISO boots to ready state. | Spawn into minimal live environment. |
| 02-LiveTools | Live ISO tools verified. | Spawn into live env with functional installer + toolchain. |
| 03-Install | Disk installation completes. | Spawn into freshly installed system (pre-login verified). |
| 04-LoginGate | Installed system reaches deterministic login boundary. | Spawn at login surface (TTY/DM/console ready). |
| 05-Harness | Harness can reliably authenticate and execute commands. | Spawn into installed system with trusted automation access. |
| 06-Runtime | Core programs pass integration tests under harness control. | Spawn into validated runtime baseline (canonical state). |
| 07-Update | A/B slot edit + reboot into alternate slot verified. | Spawn into update-capable system with confirmed slot identity. |
| 08-Package | 06 baseline convertible to release artifacts (`qcow2`, `.img`, ISO where applicable). | Spawn into distributable image derived from 06 baseline. |

### Caveat

At 00 the ISO must build successfully, but it is not yet feature-complete.
Each subsequent stage validates additional functionality and requires rebuilding the ISO with the newly verified components included.
The ISO at 06 represents the fully verified runtime baseline.
08 converts that verified baseline into distributable images.

## Legend

- `OK`: verified for that exact output target
- `X`: not verified yet
- `-`: not applicable

## Progress Table

| Stage | Lev x86_64 A/B | Lev x86_64 mutable | Lev aarch64 A/B | Lev aarch64 mutable | Ralph x86_64 A/B | Ralph aarch64 A/B | Acorn x86_64 A/B | Acorn x86_64 mutable | Acorn aarch64 A/B | Acorn aarch64 mutable | Iuppiter x86_64 A/B |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 00 | X | OK | X | X | OK | X | X | X | X | X | X |
| 01 | X | OK | X | X | X | X | X | OK | X | X | X |
| 02 | X | OK | X | X | X | X | X | OK | X | X | X |
| 03 | X | X | X | X | X | X | X | X | X | X | X |
| 04 | X | X | X | X | X | X | X | X | X | X | X |
| 05 | X | X | X | X | X | X | X | X | X | X | X |
| 06 | X | X | X | X | X | X | X | X | X | X | X |
| 07 | X | - | X | - | X | X | X | - | X | - | X |
| 08 | X | X | X | X | X | X | X | X | X | X | X |

## Notes

- Levitate/Acorn A/B columns are expected to remain `X` until A/B install flow is implemented.
- Ralph live install env is internal even though Ralph is public; 08 release target is public `qcow2`.
- Iuppiter remains private/internal; 08 release target is non-public `.img`.
- DO NOT UNDERESTIMATE 05: it is the stage where the harness becomes a trusted instrument (reliable login + readiness detection + command execution on an installed OS); without 05, 06–08 results are not credible.
