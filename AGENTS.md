# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml`: Rust workspace (builders, shared libs, tools, tests).
- `leviso/`, `AcornOS/`, `IuppiterOS/`: old distro entrypoints - for reference.
- `distro-variants/{levitate,acorn,ralph,iuppiter}`: new distro entrypoints that passes the strict conformance test. 
- `distro-builder/`, `distro-spec/`, `distro-contract/`: shared build engine + specs/contracts.
- `tools/`: standalone CLIs (`recipe`, `recstrap`, `recfstab`, `recchroot`, `reciso`, `recqemu`, `recart`, ...).
- `testing/`: Rust test harnesses (notably `install-tests/` stage tests and `rootfs-tests/`).
- `docs/`: Bun/Turbo workspaces (`docs/website`, `docs/tui`, `docs/content`).
- `llm-toolkit/`: Python utilities for local LLM workflows.

## Legacy Crates Are Read-Only
- Treat legacy distro crates as read-only by default: `leviso/`, `AcornOS/`, `IuppiterOS/`, `RalphOS/`.
- Do not add new wiring, conformance logic, stage logic, or feature work to legacy crates.
- Implement new behavior in `distro-variants/*`, `distro-contract`, `distro-builder`, and `testing/*`.
- Legacy crate edits are allowed only when the user explicitly requests them for a scoped compatibility fix.
- Never choose legacy-crate edits as a shortcut when equivalent work belongs in the new entrypoints.

This repo uses git submodules; prefer `git clone --recurse-submodules` or run
`git submodule update --init --recursive`.

## Build, Test, and Development Commands
- Build an ISO (Stage 00 endpoint): `just build levitate` (aliases: `leviso`, `acorn`, `iuppiter`, `ralph`).
- Build all ISOs (Stage 00 endpoint): `just build-all`.
- Boot interactively: `just stage 1 leviso` (live boot), `just stage 2 leviso` (live tools + interactive). Exit QEMU: `Ctrl-A X`.
- Run automated stages: `just test 4 levitate`, `just test-up-to 6 levitate`, `just test-status levitate` (also: `acorn`, `iuppiter`).
- Rust checks (CI-style): `cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.
- Install pre-commit hooks (fmt + clippy + unit tests): `cargo xtask hooks install`.
- Docs dev/build (Bun): `bun run dev`, `bun run build`, `bun run check`.

## Working in Dirty Trees
- If the git working tree already has changes, assume they are intentional and leave them untouched by default.
- Do not revert/remove unrelated diffs unless the user explicitly asks to clean up, minimize a PR, or revert specific files.
- It is still OK to mention that unrelated changes exist when it affects reviewability, submodule pointers, conflicts, or CI noise.
- Always assume all diffs are intentional, including diffs you do not recognize.
- Never suggest reverting changes. Only revert when the user explicitly requests reverting specific files/changes.
- If unrelated diffs affect reviewability or risk mixing concerns, ask whether to include them in the current scope or leave them alone.
- Keep a short running list of files you changed in the current task and include it in your final message.
- If a diff is in a file you touched in the current task, treat it as part of your work: re-open the file/diff to reorient; do not ask the user to explain your own changes.

## Commit Behavior ("Commit ALL")
- If the user asks to commit "ALL" (or asks for a "full clean working tree"), the goal is a fully clean `git status` in the superproject and in every submodule.
- Commit order:
- 1) Commit inside each dirty submodule first (one or more commits per submodule, grouped by theme with meaningful Conventional Commit messages).
- 2) Then commit the superproject changes (submodule pointer bumps, `.gitmodules`, workspace membership, docs, etc.), also grouped by theme with meaningful messages.
- Never commit files the agent believes should be gitignored (build outputs, caches, temp artifacts, secrets), even if they are currently unignored, unless the user explicitly asks to commit them.
- If "should-be-gitignored" files are present and are blocking a clean tree, prefer adding/adjusting `.gitignore` (and, if needed, removing them from tracking) rather than committing them.

## Conformance test (distro-contract)
- Primary theme: find and fix inconsistencies across the full stage ladder (Stage 00-Stage 08). Do not hide inconsistencies to make tests pass.
- Treat inconsistencies as first-class failures: path mismatches, schema drift, duplicate sources of truth, stage responsibility leakage, and per-distro behavior divergence without explicit declaration.
- Preserve stage boundaries from `stages.md`: each stage must enforce its own scope and must not absorb or mask failures that belong to another stage.
- Stage 00 is the build-capability exception (not a spawnable runtime state). Stage 01-Stage 08 are spawnable system-state stages and must remain independently auditable.
- For Stage 00 kernel conformance, enforce Recipe Rhai kernel orchestration (`distro-builder/recipes/linux.rhai` via `recipe install`) and kernel provenance invariants (`kconfig` validity, `kernel.release` version prefix, distro `CONFIG_LOCALVERSION` suffix match).
- Prefer one canonical declaration per invariant (single source of truth). When multiple values exist, converge or fail conformance.

## Stage Naming Policy
- Use canonical numbered stage names everywhere in new work: `00Build`, `01Boot`, `02LiveTools`, `03Install`, `04LoginGate`, `05Harness`, `06Runtime`, `07Update`, `08Package`.
- Do not introduce new unclear aliases such as `stage00`, `stage_00`, or mixed ad-hoc labels in new modules/files/APIs.
- For filesystem ordering, prefer numbered prefixes in filenames/modules (for example `s00_build`, `s01_boot`).
- Compatibility exception: keep existing schema keys/protocol fields unchanged where external compatibility depends on them (for example contract TOML keys that already use `stage_00`).

## Stage System

The stage loop is implemented in `testing/install-tests` (CLI: `cargo run --bin stages -- ...`) and is intended to be the fast E2E boot/install regression signal.

### Usage
- Run one: `just test 2 levitate` (or `acorn`, `iuppiter`)
- Run up to N: `just test-up-to 4 levitate`
- Show status: `just test-status levitate`
- Reset cached state: `just test-reset levitate`
- Prefer `just ...` wrappers: `justfile` exports PATH/LD_LIBRARY_PATH and `OVMF_PATH` for the repo-managed QEMU tooling under `leviso/downloads/.tools/`.
- Distro IDs: the harness uses `levitate`, `acorn`, `iuppiter`. The `just stage` helper uses `leviso`, `acorn`, `iuppiter`.

### What Stages Mean
- Stages are `1..=6` in code (`testing/install-tests/src/stages/mod.rs`).
- State is persisted under `.stages/<distro>.json` and is gated (stage N requires N-1 passed).
- If the ISO file mtime changes, cached results are invalidated automatically.
- Stage 03+ uses a temp disk + writable OVMF vars under `std::env::temp_dir()` (usually `/tmp`).

### Artifacts & Paths
- Stage state: `.stages/<distro>.json` (gitignored).
- Stage temp disk: `$TMPDIR/stage-<distro>-disk.qcow2`
- Stage temp OVMF vars: `$TMPDIR/stage-<distro>-vars.fd`
- Full `install-tests` runner temp disk: `$TMPDIR/leviso-install-test.qcow2`
- Full `install-tests` runner temp OVMF vars: `$TMPDIR/leviso-install-test-vars.fd`
- QMP smoke test temp artifacts: `$TMPDIR/leviso-qmp-smoke.qcow2`, `$TMPDIR/leviso-qmp-smoke-vars.fd`, `$TMPDIR/leviso-qmp-smoke.sock`
- Distro QEMU runners disk (interactive dev): `.artifacts/out/<DistroDir>/virtual-disk.qcow2` (legacy `<DistroDir>/output` is a symlink)

### Interactive QEMU (Justfile)
`just stage` is a manual QEMU runner (defined in `justfile`), currently only:
- `just stage 1 <distro>`: direct QEMU boot of the live ISO (serial)
- `just stage 4 <distro>`: direct QEMU boot of an already-installed disk from `.artifacts/out/<DistroDir>/*test.qcow2` (separate from the harness disk in `$TMPDIR`)

Note: the distro QEMU runners (`cargo run -- run`) use `.artifacts/out/<DistroDir>/virtual-disk.qcow2` (legacy `output/virtual-disk.qcow2` still works via symlink). The justfile stage-4 helper expects `*test.qcow2` + `*ovmf-vars.fd` under `.artifacts/out/<DistroDir>/`.

Note: the `stages` CLI accepts `--interactive`, and the WIP implementation lives in `testing/install-tests/src/interactive.rs`, but it is not currently wired up in `testing/install-tests/src/bin/stages.rs`. Installed interactive stages (3-6) are not implemented yet.
/home/vince/Projects/ralph4days/crates/ralphd
### On-ISO Stage Scripts
Shell scripts exist in `testing/install-tests/test-scripts/` (`stage-*.sh` + `lib/common.sh`) and are intended to ship on ISOs for manual debugging.
Wired for all three distros:
- AcornOS installs them into the live rootfs at `/usr/local/bin/stage-*.sh` and `/usr/local/lib/stage-tests/common.sh` (see `AcornOS/src/component/custom/mod.rs` and `AcornOS/src/component/definitions.rs`).
- IuppiterOS installs them into the live rootfs at `/usr/local/bin/stage-*.sh` and `/usr/local/lib/stage-tests/common.sh` (see `IuppiterOS/src/component/custom/mod.rs` and `IuppiterOS/src/component/definitions.rs`).
- LevitateOS installs them into the live rootfs at `/usr/local/bin/stage-*.sh` and `/usr/local/lib/stage-tests/common.sh` (see `leviso/src/component/custom/mod.rs` and `leviso/src/component/definitions.rs`).

To verify without booting, inspect the EROFS rootfs:
- `dump.erofs --path /usr/local/bin/stage-01-live-boot.sh .artifacts/out/<DistroDir>/filesystem.erofs`

### Kernel Builds (Nightly, Centralized)
Kernel compilation is centralized in `xtask` so it only happens during the allowed build-hours window (23:00 through 10:00 local time).

- Build one kernel (x86_64): `cargo xtask kernels build <distro>`
- Build all kernels (4 distros, x86_64): `cargo xtask kernels build-all`
- Rebuild regardless of existing artifacts: `cargo xtask kernels build-all --rebuild`

To verify whether a kernel is built for the right distro, check the kernel release suffix (from `CONFIG_LOCALVERSION` in each distro `kconfig`):
- LevitateOS: `file .artifacts/out/levitate/staging/boot/vmlinuz` should include `-levitate`
- AcornOS: `file .artifacts/out/acorn/staging/boot/vmlinuz` should include `-acorn`
- IuppiterOS: `file .artifacts/out/iuppiter/staging/boot/vmlinuz` should include `-iuppiter`
- RalphOS: `file .artifacts/out/ralph/staging/boot/vmlinuz` should include `-ralph`

If the suffix does not match, treat it as a broken kernel provenance/build and rebuild via `cargo xtask kernels build <distro> --rebuild`.

### Kernel Boundary Policy (Do Not Blur This)
- The "kernel" is strictly the kernel payload and provenance artifacts:
- `.artifacts/out/<DistroDir>/kernel-build/**` (including `include/config/kernel.release`, `arch/x86/boot/bzImage`, `.config`)
- `.artifacts/out/<DistroDir>/staging/boot/vmlinuz`
- `.artifacts/out/<DistroDir>/staging/usr/lib/modules/<kernel.release>/` (or equivalent modules tree under staging)
- Rebuilding the kernel means changing any artifact above. Kernel rebuilds must go through `cargo xtask kernels build ...` only.
- Everything else is non-kernel and may be rebuilt at any time without rebuilding the kernel.

### Non-Kernel Rebuild Policy (Always Allowed)
- Non-kernel artifacts include (not exhaustive):
- `.artifacts/out/<DistroDir>/filesystem.erofs`
- `.artifacts/out/<DistroDir>/initramfs-live.cpio.gz`
- `.artifacts/out/<DistroDir>/initramfs-installed.img`
- `.artifacts/out/<DistroDir>/*.iso`
- `.artifacts/out/<DistroDir>/*.sha512`
- Stage 00 ISO rebuilds must reuse existing kernel artifacts whenever kernel conformance/provenance already passes.
- Do not invoke kernel rebuilds as a shortcut when only ISO/rootfs/initramfs content changed.
- If ISO needs to be rebuilt, rebuild ISO (and dependent non-kernel artifacts) only.

### Reproducibility-First Build Policy (Strict)
- It is strictly forbidden to use ad-hoc/manual artifact surgery to "make builds pass" (for example: `cp`, `mv`, `ln -s`, direct edits under `.artifacts/out/*`, one-off directory reshuffles, or hand-fixing output paths).
- If Stage `00Build` fails due to layout/path/provenance mismatches, fix the code/contract/wiring in `distro-builder`, `distro-variants/*`, `distro-contract`, `distro-spec`, or `testing/*` so the build is reproducible from commands alone.
- "Build S00 ISO" means the repository code is correct end-to-end; it does not mean post-build manual patching of outputs.
- Any temporary manual workaround discovered during debugging must be replaced by a code fix before reporting success.
- Prefer deterministic verification commands (`just build <distro>`, `cargo run -p distro-builder --bin distro-builder -- iso build <distro> 00Build`, Stage 00 preflight) as proof of reproducibility.

## Centralized Artifact Store

Build outputs are centralized under `.artifacts/out/<DistroDir>/` (and downloads remain under `<DistroDir>/downloads/`). Legacy `<DistroDir>/output/` is a symlink to the centralized location. To make incremental work and cross-distro reuse less scattered, the repo also maintains a repo-local content-addressed artifact store:
- Store root (gitignored): `.artifacts/`
- Index: `.artifacts/index/<kind>/<input_key>.json` where `input_key` is the contents of an inputs-hash file (typically `.artifacts/out/<DistroDir>/.<artifact>-inputs.hash`)
- Blobs: `.artifacts/blobs/sha256/<prefix>/<sha256>`

Supported kinds (initial):
- `kernel_payload` (`.artifacts/out/<DistroDir>/staging/boot/vmlinuz` + kernel modules under `.artifacts/out/<DistroDir>/staging/{lib,usr/lib}/modules/`)
- `rootfs_erofs` (e.g. `.artifacts/out/<DistroDir>/filesystem.erofs`)
- `initramfs` (e.g. `.artifacts/out/<DistroDir>/initramfs-live.cpio.gz`)
- `install_initramfs` (LevitateOS only, e.g. `.artifacts/out/levitate/initramfs-installed.img`)
- `iso` (e.g. `.artifacts/out/<DistroDir>/*.iso`)
- `iso_checksum` (e.g. `.artifacts/out/<DistroDir>/*.sha512` or `.artifacts/out/<DistroDir>/*.iso.sha512`)

Tooling:
- Status: `cargo run -p recart -- status`
- List entries: `cargo run -p recart -- ls rootfs_erofs`
- GC unreferenced blobs: `cargo run -p recart -- gc`
- Prune (keep last N per kind, then GC): `cargo run -p recart -- prune --keep-last 3`
- Local artifact explorer UI (read-only): `cargo run -p recart -- serve` (open `http://127.0.0.1:8765/`)
- Local artifact explorer UI (mutations enabled): `cargo run -p recart -- serve --allow-mutate` (open the printed `?token=...` URL)

## Coding Style & Naming Conventions
- Rust: `cargo fmt` formatting; keep `cargo clippy -- -D warnings` clean. Avoid
  `unwrap()`/`panic!()` in production paths (see `QUALITY.md`).
- CLIs: quiet on success, loud on failure (Unix tool behavior).
- TypeScript (docs): formatted/linted via Biome (see `docs/content/biome.json`); use repo scripts (`bun run lint`, `bun run typecheck`).

## Testing Guidelines
- Prefer stage-based E2E coverage in `testing/install-tests/` (`just test ...`) for install/boot regressions.
- Add unit/integration tests for new public Rust APIs; keep tests deterministic.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`, `chore: ...`, optionally scoped (`feat(leviso): ...`).
- PRs should include: what changed, how to reproduce, and relevant test output (e.g., stage number/distro). Run fmt/clippy/tests before review.

## Security & Supply Chain
- Dependency/license policy is enforced via `cargo deny` (`deny.toml`). If you change dependencies, run `cargo deny check licenses`.
