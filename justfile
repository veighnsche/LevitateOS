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

kernels-build-plain distro:
    cargo xtask kernels build {{distro}}

kernels-build distro llm_profile="kernels_nightly" attempts="4" prompt_file="":
    if [ -n "{{prompt_file}}" ]; then cargo xtask kernels build {{distro}} --autofix --autofix-attempts {{attempts}} --llm-profile "{{llm_profile}}" --autofix-prompt-file "{{prompt_file}}"; else cargo xtask kernels build {{distro}} --autofix --autofix-attempts {{attempts}} --llm-profile "{{llm_profile}}"; fi

kernels-build-all-plain:
    cargo xtask kernels build-all

kernels-build-all llm_profile="kernels_nightly" attempts="4" prompt_file="":
    if [ -n "{{prompt_file}}" ]; then cargo xtask kernels build-all --autofix --autofix-attempts {{attempts}} --llm-profile "{{llm_profile}}" --autofix-prompt-file "{{prompt_file}}"; else cargo xtask kernels build-all --autofix --autofix-attempts {{attempts}} --llm-profile "{{llm_profile}}"; fi

kernels-rebuild distro:
    cargo xtask kernels build {{distro}} --rebuild

kernels-rebuild-all:
    cargo xtask kernels build-all --rebuild

# Boot into a stage (interactive serial, Ctrl-A X to exit)
[no-exit-message]
stage n distro="leviso":
    cargo xtask stages boot {{n}} {{distro}}

# Run automated stage test (pass/fail)
test n distro="levitate":
    cargo xtask stages test {{n}} {{distro}}

# Run all stage tests up to N
test-up-to n distro="levitate":
    cargo xtask stages test-up-to {{n}} {{distro}}

# Show stage test status
test-status distro="levitate":
    cargo xtask stages status {{distro}}

# Reset stage test state
test-reset distro="levitate":
    cargo xtask stages reset {{distro}}

# Build ISO via new distro-builder endpoint (`distro-variants` Stage 00 flow)
build distro="levitate" stage="00Build":
    distro_id="{{distro}}"; \
    case "$distro_id" in \
      leviso|levitate) distro_id="levitate" ;; \
      acorn|acornos) distro_id="acorn" ;; \
      iuppiter|iuppiteros) distro_id="iuppiter" ;; \
      ralph|ralphos) distro_id="ralph" ;; \
      *) echo "Unsupported distro '$distro_id' (expected levitate|acorn|iuppiter|ralph)" >&2; exit 2 ;; \
    esac; \
    cargo run -p distro-builder --bin distro-builder -- iso build "$distro_id" "{{stage}}"

# Build ISOs for all variants via new endpoint
build-all stage="00Build":
    cargo run -p distro-builder --bin distro-builder -- iso build-all "{{stage}}"

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
