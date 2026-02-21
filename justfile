# LevitateOS development commands

# QEMU tools environment
# The tooling cache lives under .artifacts/tools/.tools.
tools_prefix := join(justfile_directory(), ".artifacts/tools/.tools")
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

# Fail fast on forbidden legacy stage/rootfs bindings.
policy-legacy:
    cargo xtask policy audit-legacy-bindings

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

# Internal delegate for stage booting.
# Keep `cargo xtask stages boot` as the only execution path for stage wrappers.
# Boundary rule: stage wrappers consume existing artifacts only.
# Do not add implicit ISO build steps here; freshness is explicit via `just build*`.
[script, no-exit-message]
_boot_stage n distro="levitate" inject="" inject_file="" ssh="false" no_shell="false" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub") ssh_privkey="" ssh_port="2222":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{ssh}}" = "true" ] && [ "{{n}}" != "1" ] && [ "{{n}}" != "01" ] && [ "{{n}}" != "2" ] && [ "{{n}}" != "02" ]; then
      echo "SSH boot mode supports only live stages 1/2 (got: {{n}})" >&2
      exit 2
    fi

    args=(cargo xtask stages boot {{n}} "{{distro}}")

    if [ "{{ssh}}" = "true" ]; then
      args+=(--ssh --ssh-port "{{ssh_port}}")
    fi

    if [ "{{no_shell}}" = "true" ]; then
      args+=(--no-shell)
    fi

    if [ -n "{{inject_file}}" ]; then
      args+=(--inject-file "{{inject_file}}")
    elif [ -n "{{inject}}" ]; then
      args+=(--inject "{{inject}}")
    elif [ -f "{{ssh_pubkey}}" ]; then
      tmp=$(mktemp)
      trap 'rm -f "$tmp"' EXIT
      key="$(tr -d '\n' < "{{ssh_pubkey}}")"
      printf 'SSH_AUTHORIZED_KEY=%s\n' "$key" > "$tmp"
      args+=(--inject-file "$tmp")
    fi

    if [ "{{ssh}}" = "true" ] && [ -n "{{ssh_privkey}}" ]; then
      args+=(--ssh-private-key "{{ssh_privkey}}")
    fi

    "${args[@]}"

# Boot into a stage (interactive serial, Ctrl-A X to exit)
[no-exit-message]
stage n distro="levitate" inject="" inject_file="" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub"):
    just _boot_stage {{n}} {{distro}} "{{inject}}" "{{inject_file}}" false false "{{ssh_pubkey}}" "" 2222

# Boot a live stage in background and SSH into it (no serial wrapper harness).
[no-exit-message]
stage-ssh n distro="levitate" inject="" inject_file="" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub") ssh_privkey=(env("HOME") + "/.ssh/id_ed25519") ssh_port="2222":
    just _boot_stage {{n}} {{distro}} "{{inject}}" "{{inject_file}}" true false "{{ssh_pubkey}}" "{{ssh_privkey}}" "{{ssh_port}}"

# Single-path Stage 01 parity gate (serial boot + SSH boot).
[script, no-exit-message]
s01-parity distro="levitate" inject="" inject_file="" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub") ssh_privkey=(env("HOME") + "/.ssh/id_ed25519") ssh_port="2222":
    #!/usr/bin/env bash
    set -euo pipefail
    just build 01Boot {{distro}}

    tmp_serial=""
    tmp_ssh=""
    cleanup() {
      [ -n "${tmp_serial}" ] && [ -f "${tmp_serial}" ] && rm -f "${tmp_serial}"
      [ -n "${tmp_ssh}" ] && [ -f "${tmp_ssh}" ] && rm -f "${tmp_ssh}"
    }
    trap cleanup EXIT INT TERM

    serial_args=()
    if [ -n "{{inject_file}}" ]; then
      serial_args=(--inject-file "{{inject_file}}")
    elif [ -n "{{inject}}" ]; then
      serial_args=(--inject "{{inject}}")
    elif [ -f "{{ssh_pubkey}}" ]; then
      tmp_serial="$(mktemp)"
      key="$(tr -d '\n' < "{{ssh_pubkey}}")"
      printf 'SSH_AUTHORIZED_KEY=%s\n' "$key" > "$tmp_serial"
      serial_args=(--inject-file "$tmp_serial")
    fi

    ssh_args=(--ssh --ssh-port "{{ssh_port}}" --no-shell)
    if [ -n "{{ssh_privkey}}" ]; then
      ssh_args+=(--ssh-private-key "{{ssh_privkey}}")
    fi
    if [ -n "{{inject_file}}" ]; then
      ssh_args+=(--inject-file "{{inject_file}}")
    elif [ -n "{{inject}}" ]; then
      ssh_args+=(--inject "{{inject}}")
    elif [ -f "{{ssh_pubkey}}" ]; then
      tmp_ssh="$(mktemp)"
      key="$(tr -d '\n' < "{{ssh_pubkey}}")"
      printf 'SSH_AUTHORIZED_KEY=%s\n' "$key" > "$tmp_ssh"
      ssh_args+=(--inject-file "$tmp_ssh")
    fi

    cargo xtask stages boot 1 "{{distro}}" --no-shell "${serial_args[@]}"
    cargo xtask stages boot 1 "{{distro}}" "${ssh_args[@]}"

# Run automated stage test (pass/fail)
test n distro="levitate" inject="" inject_file="" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub"):
    if [ -n "{{inject_file}}" ]; then \
      cargo xtask stages test {{n}} {{distro}} --inject-file "{{inject_file}}"; \
    elif [ -n "{{inject}}" ]; then \
      cargo xtask stages test {{n}} {{distro}} --inject "{{inject}}"; \
    elif [ -f "{{ssh_pubkey}}" ]; then \
      tmp=$(mktemp); \
      trap 'rm -f "$tmp"' EXIT; \
      key="$(tr -d '\n' < "{{ssh_pubkey}}")"; \
      printf 'SSH_AUTHORIZED_KEY=%s\n' "$key" > "$tmp"; \
      cargo xtask stages test {{n}} {{distro}} --inject-file "$tmp"; \
    else \
      cargo xtask stages test {{n}} {{distro}}; \
    fi

# Run all stage tests up to N
test-up-to n distro="levitate" inject="" inject_file="" ssh_pubkey=(env("HOME") + "/.ssh/id_ed25519.pub"):
    if [ -n "{{inject_file}}" ]; then \
      cargo xtask stages test-up-to {{n}} {{distro}} --inject-file "{{inject_file}}"; \
    elif [ -n "{{inject}}" ]; then \
      cargo xtask stages test-up-to {{n}} {{distro}} --inject "{{inject}}"; \
    elif [ -f "{{ssh_pubkey}}" ]; then \
      tmp=$(mktemp); \
      trap 'rm -f "$tmp"' EXIT; \
      key="$(tr -d '\n' < "{{ssh_pubkey}}")"; \
      printf 'SSH_AUTHORIZED_KEY=%s\n' "$key" > "$tmp"; \
      cargo xtask stages test-up-to {{n}} {{distro}} --inject-file "$tmp"; \
    else \
      cargo xtask stages test-up-to {{n}} {{distro}}; \
    fi

# Show stage test status
test-status distro="levitate":
    cargo xtask stages status {{distro}}

# Reset stage test state
test-reset distro="levitate":
    cargo xtask stages reset {{distro}}

# Build ISO via new distro-builder endpoint (`distro-variants` Stage flow).
# Human-friendly: use `<stage-or-distro> [<stage-or-distro>]`.
# `distro-builder` canonicalizes missing/default values and aliases.
[script, no-exit-message]
build *args:
    #!/usr/bin/env bash
    set -euo pipefail

    cargo run -p distro-builder --bin distro-builder -- iso build {{args}}

# Build stage ISOs from 00 up to N (inclusive) for a distro.
# Usage: just build-up-to 2 levitate
[script, no-exit-message]
build-up-to n distro="levitate":
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{n}}" in
      0|00) target=0 ;;
      1|01) target=1 ;;
      2|02) target=2 ;;
      *)
        echo "build-up-to supports stages 0..2 (got: {{n}})" >&2
        exit 2
        ;;
    esac

    stages=(00Build 01Boot 02LiveTools)
    for i in $(seq 0 "$target"); do
      stage="${stages[$i]}"
      echo "==> Building ${stage} for {{distro}}"
      just build "{{distro}}" "${stage}"
    done

# Build ISOs for all variants via new endpoint
build-all *args:
    cargo run -p distro-builder --bin distro-builder -- iso build-all {{args}}

# Remove stage artifacts output tree (all stage run directories and manifests).
clean-out:
    rm -rf .artifacts/out

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
