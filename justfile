# LevitateOS development commands

# QEMU tools environment
tools_prefix := join(justfile_directory(), "leviso/downloads/.tools")
export PATH := tools_prefix / "usr/bin" + ":" + tools_prefix / "usr/libexec" + ":" + env("PATH")
export LD_LIBRARY_PATH := tools_prefix / "usr/lib64"
export OVMF_PATH := tools_prefix / "usr/share/edk2/ovmf/OVMF_CODE.fd"

# -----------------------------------------------------------------------------
# xtask wrappers
# Prefer invoking xtask from just, and keep the justfile itself logic-light.

# Print the environment exports that the justfile sets for QEMU/tooling.
#
# Usage:
#   eval "$(just env bash)"
env shell="bash":
    cargo xtask env {{shell}}

# Check that local toolchain/tools match what this repo expects.
doctor:
    cargo xtask doctor

# Install/remove the shared pre-commit hook into the workspace + Rust submodules.
hooks-install:
    cargo xtask hooks install

hooks-remove:
    cargo xtask hooks remove

# Kernel helpers (x86_64).
kernels-check:
    cargo xtask kernels check

kernels-check-one distro:
    cargo xtask kernels check {{distro}}

kernels-build-all-x86-64:
    cargo xtask kernels build-all-x86-64

kernels-rebuild-all-x86-64:
    cargo xtask kernels build-all-x86-64 --rebuild

# Boot into a checkpoint stage (interactive serial, Ctrl-A X to exit)
[no-exit-message]
checkpoint n distro="leviso":
    cargo xtask checkpoints boot {{n}} {{distro}}

# Run automated checkpoint test (pass/fail)
test n distro="levitate":
    cargo xtask checkpoints test {{n}} {{distro}}

# Run all checkpoint tests up to N
test-up-to n distro="levitate":
    cargo xtask checkpoints test-up-to {{n}} {{distro}}

# Show checkpoint test status
test-status distro="levitate":
    cargo xtask checkpoints status {{distro}}

# Reset checkpoint test state
test-reset distro="levitate":
    cargo xtask checkpoints reset {{distro}}

# Build ISO
build distro="leviso":
    cd {{distro}} && cargo run -- build

# Docs content (shared by website + tui)
docs-content-build:
    cd docs/content && bun run build

docs-content-check:
    cd docs/content && bun run check

# Website (Astro)
website-dev:
    cd docs/website && bun run dev

website-build:
    cd docs/website && bun run build

website-typecheck:
    cd docs/website && bun run typecheck
