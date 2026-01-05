# Phase 1: Discovery and Safeguards

## Refactor Summary
Refactor the `xtask` CLI to use nested subcommands.
Current flat structure (`build`, `run`, `run-vnc`, `run-pixel6`, `gpu-dump`) is becoming unwieldy.
We need a structure that supports "way more commands" and debugging tools easily.

## Success Criteria
- CLI is organized into logical groups: `build`, `run`, `image`, `clean`.
- New `clean` command exists to handle QEMU locks.
- `cargo xtask test` remains top-level and functional (Rule 25).
- No functionality is lost.

## Behavioral Contracts
- **`cargo xtask test`**: Must accept `unit`, `behavior`, `regress`, `gicv3` or empty/all.
- **`cargo xtask build`**: Should be able to build just kernel or just userspace.
- **`cargo xtask run`**: Should default to running QEMU (default profile).

## Golden/Regression Tests
- The "Golden" standard here is the `xtask` functionality itself.
- **Verification:**
  1. `cargo xtask test behavior` must pass before and after.
  2. `cargo xtask run` must invoke QEMU correctly.

## Current Architecture Notes
- `xtask/src/main.rs` contains all logic.
- `Cli` struct uses specific `Commands` enum.
- Some commands like `RunVnc` are separate variants.

## Constraints
- `clap` version: We are using `clap` with `derive` feature.
- `Rule 25`: "A single command must be capable of running ALL tests". `cargo xtask test` must stay.

## Open Questions
- None.

## Steps

### Step 1: Map Current Commands
The current commands are:
- `Test`
- `Build` (Implies Kernel + Userspace + disk)
- `Run` (Implies Build + QEMU Default)
- `RunPixel6` (Implies Build + QEMU Pixel6)
- `GpuDump` (QMP screenshot)
- `RunVnc` (QEMU VNC)

### Step 2: Define New Mapping
- `Test` -> `Test`
- `Build` -> `Build { Kernel, Userspace, All }`. `xtask build` (no args) -> `Build::All`? Or `Build` command with optional subcommand? Clap supports this.
- `Run` -> `Run { Default, Pixel6, Vnc }`.
- `GpuDump` -> `Image { Dump }` or `Qmp { Dump }`? User said "make way more commands". Maybe `Debug { Dump }`?
- `RunVnc` -> `Run { Vnc }`.
- NEW `Clean` -> `Clean`.
