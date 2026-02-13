# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml`: Rust workspace (builders, shared libs, tools, tests).
- `leviso/`, `AcornOS/`, `IuppiterOS/`: distro entrypoints (build ISO, run QEMU).
- `distro-builder/`, `distro-spec/`, `distro-contract/`: shared build engine + specs/contracts.
- `tools/`: standalone CLIs (`recipe`, `recstrap`, `recfstab`, `recchroot`, `reciso`, `recqemu`, ...).
- `testing/`: Rust test harnesses (notably `install-tests/` checkpoint tests and `rootfs-tests/`).
- `docs/`: Bun/Turbo workspaces (`docs/website`, `docs/tui`, `docs/content`).
- `llm-toolkit/`: Python utilities for local LLM workflows.

This repo uses git submodules; prefer `git clone --recurse-submodules` or run
`git submodule update --init --recursive`.

## Build, Test, and Development Commands
- Build an ISO: `just build leviso` (or `cd leviso && cargo run -- build`).
- Boot interactively: `just checkpoint 1 leviso` (exit QEMU: `Ctrl-A X`).
- Run automated checkpoints: `just test 4 levitate`, `just test-up-to 6 levitate`, `just test-status levitate` (also: `acorn`, `iuppiter`).
- Rust checks (CI-style): `cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.
- Install pre-commit hooks (fmt + clippy + unit tests): `tools/install-hooks.sh`.
- Docs dev/build (Bun): `bun run dev`, `bun run build`, `bun run check`.

## Checkpoint System

The checkpoint loop is implemented in `testing/install-tests` (CLI: `cargo run --bin checkpoints -- ...`) and is intended to be the fast E2E boot/install regression signal.

### Usage
- Run one: `just test 2 levitate` (or `acorn`, `iuppiter`)
- Run up to N: `just test-up-to 4 levitate`
- Show status: `just test-status levitate`
- Reset cached state: `just test-reset levitate`

### What Checkpoints Mean
- Checkpoints are `1..=6` in code (`testing/install-tests/src/checkpoints/mod.rs`).
- State is persisted under `.checkpoints/<distro>.json` and is gated (checkpoint N requires N-1 passed).
- If the ISO file mtime changes, cached results are invalidated automatically.
- Checkpoint 3+ uses temp artifacts:
  - Disk: `/tmp/checkpoint-<distro>-disk.qcow2`
  - OVMF vars: `/tmp/checkpoint-<distro>-vars.fd`

### Interactive QEMU (Justfile)
`just checkpoint` is a separate/manual runner (defined in `justfile`), currently only:
- `just checkpoint 1 <distro>`: live ISO boot
- `just checkpoint 4 <distro>`: boot an already-installed disk from `<DistroDir>/output/*test.qcow2`

Note: `checkpoints --interactive` exists as a CLI flag but is not currently wired up to the WIP interactive implementation in `testing/install-tests/src/interactive.rs`.

### On-ISO Checkpoint Scripts
Shell scripts exist in `testing/install-tests/test-scripts/` (`checkpoint-*.sh` + `lib/common.sh`) and are intended to ship on ISOs for manual debugging.
Currently:
- AcornOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `AcornOS/src/component/custom/mod.rs` and `AcornOS/src/component/definitions.rs`).
- IuppiterOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `IuppiterOS/src/component/custom/mod.rs` and `IuppiterOS/src/component/definitions.rs`).
- LevitateOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `leviso/src/component/custom/mod.rs` and `leviso/src/component/definitions.rs`).

### Kernel “Theft Mode” (DEV-only)
For Alpine-based distros (AcornOS/IuppiterOS), the shared kernel recipe (`distro-builder/recipes/linux.rhai`) may reuse/steal a prebuilt kernel from `leviso/output/kernel-build` instead of compiling.
To force a real kernel build from source, pass the kernel flag and the confirmation flag, e.g.:
- `cd AcornOS && cargo run -- build --kernel --dangerously-waste-the-users-time`

## Coding Style & Naming Conventions
- Rust: `cargo fmt` formatting; keep `cargo clippy -- -D warnings` clean. Avoid
  `unwrap()`/`panic!()` in production paths (see `QUALITY.md`).
- CLIs: quiet on success, loud on failure (Unix tool behavior).
- TypeScript (docs): formatted/linted via Biome (see `docs/content/biome.json`); use repo scripts (`bun run lint`, `bun run typecheck`).

## Testing Guidelines
- Prefer checkpoint-based E2E coverage in `testing/install-tests/` (`just test ...`) for install/boot regressions.
- Add unit/integration tests for new public Rust APIs; keep tests deterministic.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`, `chore: ...`, optionally scoped (`feat(leviso): ...`).
- PRs should include: what changed, how to reproduce, and relevant test output (e.g., checkpoint number/distro). Run fmt/clippy/tests before review.

## Security & Supply Chain
- Dependency/license policy is enforced via `cargo deny` (`deny.toml`). If you change dependencies, run `cargo deny check licenses`.
