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

| Checkpoint | LevitateOS (`levitate`, `leviso/`) | AcornOS (`acorn`, `AcornOS/`) | IuppiterOS (`iuppiter`, `IuppiterOS/`) | RalphOS (`ralph`, `RalphOS/`) |
|---|---|---|---|---|
| CP0 (Build) | OK | HALF | ? | ? |
| CP1 (Live Boot) | OK | OK | ? | ? |
| CP2 (Live Tools) | OK | OK | ? | ? |
| CP3 (Installation) | TODO | TODO | ? | ? |
| CP4 (Installed Boot) | TODO | TODO | ? | ? |
| CP5 (Automated Login) | TODO | TODO | ? | ? |
| CP6 (Daily Driver Tools) | TODO | TODO | ? | ? |

Notes:
- AcornOS: kernel is currently "stolen" from LevitateOS (theft mode), so CP0 is HALF.
