# Checkpoints

Human-maintained progress table for the checkpoint-based dev loop in
`testing/install-tests` (CLI: `cargo run --bin checkpoints -- ...`).

## Legend

- `OK`: implemented + expected to pass
- `HALF`: partially done (see CP0)
- `TODO`: not done yet
- `?`: unknown / not recently verified

## Checkpoint Definitions

- `CP0` (Build): custom Linux kernel + bootable ISO.
  - If the ISO builds but the kernel is reused/stolen ("theft mode", DEV-only), this is `HALF`.
- `CP1`: Live Boot (ISO boots in QEMU and reaches a known-good marker)
- `CP2`: Live Tools (expected live tools execute successfully)
- `CP3`: Installation (scripted install to disk succeeds)
- `CP4`: Installed Boot (boots from disk after install)
- `CP5`: Automated Login (harness can login + run commands)
- `CP6`: Daily Driver Tools (expected tools present on installed system)

## Progress Table

### x86_64

| OS | CP0 | CP1 | CP2 | CP3 | CP4 | CP5 | CP6 |
|---|---|---|---|---|---|---|---|
| LevitateOS (`levitate`, `leviso/`) | OK | OK | OK | TODO | TODO | TODO | TODO |
| AcornOS (`acorn`, `AcornOS/`) | HALF | OK | OK | TODO | TODO | TODO | TODO |
| IuppiterOS (`iuppiter`, `IuppiterOS/`) | ? | ? | ? | ? | ? | ? | ? |

### AArch64

| OS | CP0 | CP1 | CP2 | CP3 | CP4 | CP5 | CP6 |
|---|---|---|---|---|---|---|---|
| LevitateOS (`levitate`, `leviso/`) | ? | ? | ? | ? | ? | ? | ? |
| AcornOS (`acorn`, `AcornOS/`) | ? | ? | ? | ? | ? | ? | ? |

Notes:
- AcornOS: kernel is currently "stolen" from LevitateOS (theft mode), so CP0 is HALF (verified: `.artifacts/out/AcornOS/staging/boot/vmlinuz` is `*-levitate`).
