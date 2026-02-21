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

All distros share the same Stage ladder (00–08).
Differences below describe *policy and intent*, not structural deviations from the ladder.

| Area | LevitateOS | RalphOS | AcornOS | iuppiterOS |
|---|---|---|---|---|
| Visibility | Public | Public | Public | Private/internal |
| Purpose | Stable daily workstation | R4D sandbox + agent runtime | Lightweight daily system | HDD refurbishment / ephemeral tooling |
| Toolchain | glibc / systemd / GNU | glibc / systemd / GNU | musl / OpenRC / busybox | musl / OpenRC / busybox |
| Kernel Policy (00) | LTS (.artifacts/kernel/levitate/current/kernel-build) | LTS (.artifacts/kernel/ralph/current/kernel-build) | Mainline (.artifacts/kernel/acorn/current/kernel-build) | LTS (.artifacts/kernel/iuppiter/current/kernel-build) |
| Boot Policy (01) | Auto-login root (live) | Auto-login root (live) | Auto-login root (live) | Auto-login root (live) |
| Live Tools Scope (02) | Arch-parity + docs/TUI | Minimal automated installer | Arch-parity + docs/TUI | Minimal automated installer |
| Install UX (03) | Narrated logs | Verbose logs | Narrated logs | Verbose logs |
| Login Model (04) | User-defined account | Root only (pw protected) | User-defined account | Auto-login root (ephemeral) |
| Harness Authority (05) | User login + sudo verified | Root login verified | User login + sudo verified | Auto-login (ephemeral pass) |
| Runtime Validation (06) | Full integration test | Full e2e test | Integration test | e2e test |
| Update Mechanism (07) | A/B + high-value payload mutation | A/B swap validation | A/B + high-value payload mutation | A/B swap validation |
| Packaging Policy (08) | Public `ISO` + `qcow2` + `.img` | Public `qcow2` | Public `ISO` + `qcow2` + `.img` | Private `.img` |
| Package Manager | `recipe` | `recipe` | `recipe` | none |
| App Source | Rocky DVD ISO baseline | Rocky DVD ISO baseline | Alpine Extended baseline | Alpine Extended baseline |


## Stages

| Stage | Ladder Semantics (Proven Authority) | Game Savepoint Semantics (Spawnable State) |
|---|---|---|
| 00Build | Kernel + ISO build succeeds. | Not spawnable (build only). |
| 01Boot | Live ISO boots to ready state. | Spawn into minimal live environment. |
| 02LiveTools | Live ISO tools verified. | Spawn into live env with functional installer + toolchain. |
| 03Install | Disk installation completes. | Spawn into freshly installed system (pre-login verified). |
| 04LoginGate | Installed system reaches deterministic login boundary. | Spawn at login surface (TTY/DM/console ready). |
| 05Harness | Harness can reliably authenticate and execute commands. | Spawn into installed system with trusted automation access. |
| 06Runtime | Core programs pass integration tests under harness control. | Spawn into validated runtime baseline (canonical state). |
| 07Update | A/B slot edit + reboot into alternate slot verified. | Spawn into update-capable system with confirmed slot identity. |
| 08Package | 06 baseline convertible to release artifacts (`qcow2`, `.img`, ISO where applicable). | Spawn into distributable image derived from 06 baseline. |

## Stage Filesystem Delta Matrix

Cells describe the filesystem delta relative to the immediately preceding stage.

| Stage | `*-filesystem.erofs` delta vs previous stage | `*-overlayfs.erofs` delta vs previous stage | `*-initramfs-live.cpio.gz` delta vs previous stage |
|---|---|---|---|
| 00Build | Baseline rootfs payload (`s00-filesystem.erofs`) created from stage-00 producers. | Baseline empty/minimal live overlay payload (`s00-overlayfs.erofs`). | Baseline live initramfs payload (`s00-initramfs-live.cpio.gz`). |
| 01Boot | `s01 = s00 + boot additions` (boot/rootfs producers, stage test scripts, boot readiness wiring). | `s01-overlayfs.erofs` becomes non-empty live overlay with stage banner + required live service wiring (for Levitate: `sshd`). | Rebuilt for stage output naming; no intended functional delta from stage 00. |
| 02LiveTools | `s02 = s01 + live tools additions` (installer/tool binaries added into rootfs, plus stage test scripts). | Overlay policy reused from stage 01; practical delta is stage identity/banner (`S02 Live Tools`) with the same required live service wiring baseline. | Rebuilt for stage output naming; no intended functional delta from stage 01. |
| 03Install | N/A in current ISO `*fs` artifact pipeline (install validation stage, not a new live `*fs` image build stage). | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |
| 04LoginGate | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |
| 05Harness | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |
| 06Runtime | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |
| 07Update | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |
| 08Package | N/A in current ISO `*fs` artifact pipeline (packaging/conversion of verified baseline, not stage-local live `*fs` assembly). | N/A in current ISO `*fs` artifact pipeline. | N/A in current ISO `*fs` artifact pipeline. |

### Caveat

At 00 the ISO must build successfully, but it is not yet feature-complete.
Each subsequent stage validates additional functionality and requires rebuilding the ISO with the newly verified components included.
Each stage must assemble from its own stage-scoped non-kernel artifacts (`s00-build`, `s01-boot`, ...); non-kernel cross-stage reuse is not allowed.
Kernel artifacts are the only shared exception across stages.
Each stage artifact must satisfy a strict stage envelope: nothing required may be missing, and nothing from later stages may be present.
If a stage artifact includes payload outside its declared stage envelope, that stage must fail conformance.
The ISO at 06 represents the fully verified runtime baseline.
08 converts that verified baseline into distributable images.

## Legend

- `OK`: verified for that exact output target
- `X`: not verified yet
- `-`: not applicable

## Progress Table

| Stage | Lev x86_64 A/B | Lev x86_64 mutable | Lev aarch64 A/B | Lev aarch64 mutable | Ralph x86_64 A/B | Ralph aarch64 A/B | Acorn x86_64 A/B | Acorn x86_64 mutable | Acorn aarch64 A/B | Acorn aarch64 mutable | Iuppiter x86_64 A/B |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 00 | OK | OK | X | X | OK | X | OK | OK | X | X | OK |
| 01 | OK | OK | X | X | OK | X | OK | OK | X | X | OK |
| 02 | OK | OK | X | X | OK | X | OK | OK | X | X | OK |
| 03 | X | X | X | X | X | X | X | X | X | X | X |
| 04 | X | X | X | X | X | X | X | X | X | X | X |
| 05 | X | X | X | X | X | X | X | X | X | X | X |
| 06 | X | X | X | X | X | X | X | X | X | X | X |
| 07 | X | - | X | - | X | X | X | - | X | - | X |
| 08 | X | X | X | X | X | X | X | X | X | X | X |

## Notes

- Levitate/Acorn A/B columns are expected to remain `X` from install/layout-dependent stages onward (starting at 03) until A/B install flow is implemented.
- Ralph live install env is internal even though Ralph is public; 08 release target is public `qcow2`.
- Iuppiter remains private/internal; 08 release target is non-public `.img`.
- DO NOT UNDERESTIMATE 05: it is the stage where the harness becomes a trusted instrument (reliable login + readiness detection + command execution on an installed OS); without 05, 06–08 results are not credible.
