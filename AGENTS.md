# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml`: Rust workspace (builders, shared libs, tools, tests).
- `leviso/`, `AcornOS/`, `IuppiterOS/`: distro entrypoints (build ISO, run QEMU).
- `distro-builder/`, `distro-spec/`, `distro-contract/`: shared build engine + specs/contracts.
- `tools/`: standalone CLIs (`recipe`, `recstrap`, `recfstab`, `recchroot`, `reciso`, `recqemu`, `recart`, ...).
- `testing/`: Rust test harnesses (notably `install-tests/` checkpoint tests and `rootfs-tests/`).
- `docs/`: Bun/Turbo workspaces (`docs/website`, `docs/tui`, `docs/content`).
- `llm-toolkit/`: Python utilities for local LLM workflows.

This repo uses git submodules; prefer `git clone --recurse-submodules` or run
`git submodule update --init --recursive`.

## Build, Test, and Development Commands
- Build an ISO: `just build leviso` (or `cd leviso && cargo run -- build`).
- Boot interactively: `just checkpoint 1 leviso` (live boot), `just checkpoint 2 leviso` (live tools + interactive). Exit QEMU: `Ctrl-A X`.
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
- Prefer `just ...` wrappers: `justfile` exports PATH/LD_LIBRARY_PATH and `OVMF_PATH` for the repo-managed QEMU tooling under `leviso/downloads/.tools/`.
- Distro IDs: the harness uses `levitate`, `acorn`, `iuppiter`. The `just checkpoint` helper uses `leviso`, `acorn`, `iuppiter`.

### What Checkpoints Mean
- Checkpoints are `1..=6` in code (`testing/install-tests/src/checkpoints/mod.rs`).
- State is persisted under `.checkpoints/<distro>.json` and is gated (checkpoint N requires N-1 passed).
- If the ISO file mtime changes, cached results are invalidated automatically.
- Checkpoint 3+ uses a temp disk + writable OVMF vars under `std::env::temp_dir()` (usually `/tmp`).

### Artifacts & Paths
- Checkpoint state: `.checkpoints/<distro>.json` (gitignored).
- Checkpoint temp disk: `$TMPDIR/checkpoint-<distro>-disk.qcow2`
- Checkpoint temp OVMF vars: `$TMPDIR/checkpoint-<distro>-vars.fd`
- Full `install-tests` runner temp disk: `$TMPDIR/leviso-install-test.qcow2`
- Full `install-tests` runner temp OVMF vars: `$TMPDIR/leviso-install-test-vars.fd`
- QMP smoke test temp artifacts: `$TMPDIR/leviso-qmp-smoke.qcow2`, `$TMPDIR/leviso-qmp-smoke-vars.fd`, `$TMPDIR/leviso-qmp-smoke.sock`
- Distro QEMU runners disk (interactive dev): `.artifacts/out/<DistroDir>/virtual-disk.qcow2` (legacy `<DistroDir>/output` is a symlink)

### Interactive QEMU (Justfile)
`just checkpoint` is a manual QEMU runner (defined in `justfile`), currently only:
- `just checkpoint 1 <distro>`: direct QEMU boot of the live ISO (serial)
- `just checkpoint 4 <distro>`: direct QEMU boot of an already-installed disk from `.artifacts/out/<DistroDir>/*test.qcow2` (separate from the harness disk in `$TMPDIR`)

Note: the distro QEMU runners (`cargo run -- run`) use `.artifacts/out/<DistroDir>/virtual-disk.qcow2` (legacy `output/virtual-disk.qcow2` still works via symlink). The justfile checkpoint-4 helper expects `*test.qcow2` + `*ovmf-vars.fd` under `.artifacts/out/<DistroDir>/`.

Note: the `checkpoints` CLI accepts `--interactive`, and the WIP implementation lives in `testing/install-tests/src/interactive.rs`, but it is not currently wired up in `testing/install-tests/src/bin/checkpoints.rs`. Installed interactive checkpoints (3-6) are not implemented yet.

### On-ISO Checkpoint Scripts
Shell scripts exist in `testing/install-tests/test-scripts/` (`checkpoint-*.sh` + `lib/common.sh`) and are intended to ship on ISOs for manual debugging.
Wired for all three distros:
- AcornOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `AcornOS/src/component/custom/mod.rs` and `AcornOS/src/component/definitions.rs`).
- IuppiterOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `IuppiterOS/src/component/custom/mod.rs` and `IuppiterOS/src/component/definitions.rs`).
- LevitateOS installs them into the live rootfs at `/usr/local/bin/checkpoint-*.sh` and `/usr/local/lib/checkpoint-tests/common.sh` (see `leviso/src/component/custom/mod.rs` and `leviso/src/component/definitions.rs`).

To verify without booting, inspect the EROFS rootfs:
- `dump.erofs --path /usr/local/bin/checkpoint-1-live-boot.sh .artifacts/out/<DistroDir>/filesystem.erofs`

### Kernel "Theft Mode" (DEV-only)
For Alpine-based distros (AcornOS/IuppiterOS), the shared kernel recipe (`distro-builder/recipes/linux.rhai`) may reuse/steal a prebuilt kernel from `.artifacts/out/leviso/kernel-build` instead of compiling.
To force a real kernel build from source, pass the kernel flag and the confirmation flag, e.g.:
- `cd AcornOS && cargo run -- build --kernel --dangerously-waste-the-users-time`

To verify whether a kernel is "built for this distro" vs "stolen", check the kernel release suffix (from `CONFIG_LOCALVERSION` in each distro `kconfig`):
- LevitateOS: `file .artifacts/out/leviso/staging/boot/vmlinuz` should include `-levitate`
- AcornOS: `file .artifacts/out/AcornOS/staging/boot/vmlinuz` should include `-acorn`
- IuppiterOS: `file .artifacts/out/IuppiterOS/staging/boot/vmlinuz` should include `-iuppiter`

If AcornOS/IuppiterOS show `*-levitate`, that's theft-mode (kernel provenance is LevitateOS). A stronger confirmation is that the `sha256sum` of `staging/boot/vmlinuz` matches `.artifacts/out/leviso/staging/boot/vmlinuz`.

## Centralized Artifact Store

Build outputs are centralized under `.artifacts/out/<DistroDir>/` (and downloads remain under `<DistroDir>/downloads/`). Legacy `<DistroDir>/output/` is a symlink to the centralized location. To make incremental work and cross-distro reuse less scattered, the repo also maintains a repo-local content-addressed artifact store:
- Store root (gitignored): `.artifacts/`
- Index: `.artifacts/index/<kind>/<input_key>.json` where `input_key` is the contents of an inputs-hash file (typically `.artifacts/out/<DistroDir>/.<artifact>-inputs.hash`)
- Blobs: `.artifacts/blobs/sha256/<prefix>/<sha256>`

Supported kinds (initial):
- `kernel_payload` (`.artifacts/out/<DistroDir>/staging/boot/vmlinuz` + kernel modules under `.artifacts/out/<DistroDir>/staging/{lib,usr/lib}/modules/`)
- `rootfs_erofs` (e.g. `.artifacts/out/<DistroDir>/filesystem.erofs`)
- `initramfs` (e.g. `.artifacts/out/<DistroDir>/initramfs-live.cpio.gz`)
- `install_initramfs` (LevitateOS only, e.g. `.artifacts/out/leviso/initramfs-installed.img`)
- `iso` (e.g. `.artifacts/out/<DistroDir>/*.iso`)
- `iso_checksum` (e.g. `.artifacts/out/<DistroDir>/*.sha512` or `.artifacts/out/<DistroDir>/*.iso.sha512`)

Tooling:
- Status: `cargo run -p recart -- status`
- List entries: `cargo run -p recart -- ls rootfs_erofs`
- GC unreferenced blobs: `cargo run -p recart -- gc`
- Prune (keep last N per kind, then GC): `cargo run -p recart -- prune --keep-last 3`

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
