# levitate-xtask

Repository developer tasks for LevitateOS. This is intentionally small and boring; it complements the `justfile`.

## Hard Organization Rules

- `src/main.rs` is an entrypoint only.
  - Allowed: argument parsing + one call to `app::run(...)`.
  - Forbidden: task logic, filesystem probing, command execution, etc.
- Tasks must be grouped by category folders under `src/tasks/`.
  - Example: `src/tasks/tooling/`, `src/tasks/distro/`, `src/tasks/docs/`.
- Every `mod.rs` must be a shim only.
  - Allowed: `pub mod ...`, `mod ...`, and `pub use ...` re-exports.
  - Forbidden: behavior, business logic, argument handling, IO, side effects.
- Actual task implementations live in non-`mod.rs` files (for example `src/tasks/<category>/<task>.rs`).
- Shared helpers live under `src/util/` (and follow the same `mod.rs` shim rule).

If you add a new task, you should be able to find it by path alone: `src/tasks/<category>/<task>.rs`.

## Usage

```bash
# Print exports matching the justfile (for interactive work)
eval "$(cargo run -p levitate-xtask -- env bash)"

# Validate local tooling expectations
cargo run -p levitate-xtask -- doctor

# Install/remove shared git hooks across workspace + Rust submodules
cargo run -p levitate-xtask -- hooks install
cargo run -p levitate-xtask -- hooks remove

# Install-test stages (boot/install regression signal)
cargo run -p levitate-xtask -- stages boot 1 levitate
cargo run -p levitate-xtask -- stages test 4 levitate
cargo run -p levitate-xtask -- stages test-up-to 6 levitate
cargo run -p levitate-xtask -- stages status levitate
cargo run -p levitate-xtask -- stages reset levitate

# Optional boot injection (for stage boot/test paths)
cargo run -p levitate-xtask -- stages boot 1 levitate --inject 'SSH_AUTHORIZED_KEY=ssh-ed25519 AAAA...'
cargo run -p levitate-xtask -- stages test 1 levitate --inject-file /tmp/payload.env

# Kernel artifacts verification
cargo run -p levitate-xtask -- kernels check
cargo run -p levitate-xtask -- kernels check levitate

# Build one kernel (x86_64; policy window enforced)
cargo run -p levitate-xtask -- kernels build levitate

# Overnight kernel builds (levitate + acorn + iuppiter + ralph; policy window enforced: 23:00 through 10:00 local time)
cargo run -p levitate-xtask -- kernels build-all
# Rebuild even if already verified
cargo run -p levitate-xtask -- kernels build-all --rebuild
```

## Deprecated Shell Wrappers

The repo previously had ad-hoc shell scripts for kernel build/check. They are intentionally deleted; use xtask directly:

- `cargo xtask kernels build <distro>`
- `cargo xtask kernels build-all`
- `cargo xtask kernels check`
