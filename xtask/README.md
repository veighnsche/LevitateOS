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

# Install-test checkpoints (boot/install regression signal)
cargo run -p levitate-xtask -- checkpoints boot 1 leviso
cargo run -p levitate-xtask -- checkpoints test 4 levitate
cargo run -p levitate-xtask -- checkpoints test-up-to 6 levitate
cargo run -p levitate-xtask -- checkpoints status levitate
cargo run -p levitate-xtask -- checkpoints reset levitate

# Kernel artifacts verification
cargo run -p levitate-xtask -- kernels check
cargo run -p levitate-xtask -- kernels check leviso

# Overnight kernel build (policy window enforced: 23:00 through 10:00 local time)
cargo run -p levitate-xtask -- kernels build-all-x86-64
# Rebuild even if already verified (alias: --force)
cargo run -p levitate-xtask -- kernels build-all-x86-64 --rebuild
```
