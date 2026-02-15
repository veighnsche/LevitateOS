#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

echo "[info] Repo: $root_dir"

usage() {
  cat <<'EOF'
Usage:
  tools/build-all-kernels-x86_64.sh [--force]

Policy:
  Kernel building is intentionally restricted to overnight hours to avoid
  accidental "laptop-melter" builds during the day.

  Allowed local time window: 23:00 (11pm) through 10:00 (10am).
  Outside that window, this script refuses to build.

Options:
  --force   Rebuild kernels even if they already appear fully built+verified.
EOF
}

FORCE=0
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
  shift
fi
if [[ $# -ne 0 ]]; then
  echo "[error] Unexpected arguments: $*" >&2
  usage >&2
  exit 2
fi

enforce_build_hours() {
  local hhmm now
  hhmm="$(date +%H%M)"
  now="$(date '+%Y-%m-%d %H:%M:%S %z')"

  # Allowed: 23:00-23:59, 00:00-10:00 (inclusive)
  if [[ "$hhmm" -ge 2300 || "$hhmm" -le 1000 ]]; then
    return 0
  fi

  cat >&2 <<EOF
[policy] Refusing to build kernels outside the allowed window.
         Allowed local time window: 23:00 (11pm) through 10:00 (10am).
         Current local time: $now

         If you really intend to build right now, rerun later during the window.
EOF
  exit 3
}

ensure_output_link() {
  local distro_dir="$1"
  local link_path="$root_dir/$distro_dir/output"
  local target_path="../.artifacts/out/$distro_dir"

  mkdir -p "$root_dir/.artifacts/out/$distro_dir"

  if [[ -L "$link_path" ]]; then
    return 0
  fi

  if [[ -e "$link_path" ]]; then
    echo "[error] $link_path exists but is not a symlink. Refusing to replace it." >&2
    echo "        Move it out of the way, or migrate it into .artifacts/out/$distro_dir first." >&2
    exit 1
  fi

  ln -s "$target_path" "$link_path"
}

# Centralized artifacts (compat shim: <distro>/output -> .artifacts/out/<distro>)
ensure_output_link "leviso"
ensure_output_link "AcornOS"
ensure_output_link "IuppiterOS"
ensure_output_link "RalphOS"

# Pull the chosen LTS (LevitateOS SSOT) so RalphOS uses the same.
kernel_spec="$root_dir/distro-spec/src/shared/kernel.rs"
LTS_VERSION="$(
  awk '
    /pub const LEVITATE_KERNEL/ { in_block=1 }
    in_block && /version:/ {
      gsub(/[",]/, "", $2);
      print $2;
      exit
    }
  ' "$kernel_spec"
)"
LTS_SHA256="$(
  awk '
    /pub const LEVITATE_KERNEL/ { in_block=1 }
    in_block && /sha256:/ {
      gsub(/[",]/, "", $2);
      print $2;
      exit
    }
  ' "$kernel_spec"
)"

if [[ -z "${LTS_VERSION:-}" || -z "${LTS_SHA256:-}" ]]; then
  echo "[error] Failed to parse LEVITATE_KERNEL version/sha256 from $kernel_spec" >&2
  exit 1
fi

echo "[info] LTS kernel (from distro-spec): $LTS_VERSION"

verify_one() {
  local distro_dir="$1"
  local want_suffix="$2"
  local out_dir="$root_dir/.artifacts/out/$distro_dir"
  local rel_file="$out_dir/kernel-build/include/config/kernel.release"
  local vmlinuz="$out_dir/staging/boot/vmlinuz"

  if [[ ! -f "$rel_file" ]]; then
    echo "[error] Missing kernel.release: $rel_file" >&2
    exit 1
  fi
  if [[ ! -f "$vmlinuz" ]]; then
    echo "[error] Missing vmlinuz: $vmlinuz" >&2
    exit 1
  fi

  local rel
  rel="$(tr -d '\n' < "$rel_file")"
  if [[ "$rel" != *"$want_suffix" ]]; then
    echo "[error] $distro_dir kernel.release '$rel' does not end with '$want_suffix' (theft mode?)" >&2
    exit 1
  fi

  if [[ ! -d "$out_dir/staging/lib/modules/$rel" && ! -d "$out_dir/staging/usr/lib/modules/$rel" ]]; then
    echo "[error] Missing modules dir for $distro_dir ($rel) under staging/{lib,usr/lib}/modules/" >&2
    exit 1
  fi

  echo "[ok] $distro_dir: $rel"
}

kernel_is_built() {
  local distro_dir="$1"
  local want_suffix="$2"
  if verify_one "$distro_dir" "$want_suffix" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

maybe_build() {
  local distro_dir="$1"
  local want_suffix="$2"
  shift 2

  if [[ "$FORCE" -eq 0 ]] && kernel_is_built "$distro_dir" "$want_suffix"; then
    echo "[skip] $distro_dir kernel already built+verified"
    return 0
  fi

  enforce_build_hours

  # If we're going to build, purge any partial/stale kernel payload first so we
  # don't end up with "modules from old build + kernel.release from new build".
  local out_dir="$root_dir/.artifacts/out/$distro_dir"
  rm -rf \
    "$out_dir/kernel-build" \
    "$out_dir/staging/boot/vmlinuz" \
    "$out_dir/staging/lib/modules" \
    "$out_dir/staging/usr/lib/modules"

  echo "[step] Build kernel: $distro_dir"
  "$@"
}

maybe_build "AcornOS" "-acorn" \
  bash -lc "cd \"$root_dir/AcornOS\" && LEVITATE_DISABLE_KERNEL_THEFT=1 cargo run -- build --dangerously-waste-the-users-time kernel"

maybe_build "IuppiterOS" "-iuppiter" \
  bash -lc "cd \"$root_dir/IuppiterOS\" && LEVITATE_DISABLE_KERNEL_THEFT=1 cargo run -- build --dangerously-waste-the-users-time kernel"

maybe_build "RalphOS" "-ralph" \
  bash -lc "
    cd \"$root_dir\" &&
    cargo build -p levitate-recipe &&
    recipe_bin=\"$root_dir/target/debug/recipe\" &&
    test -x \"\$recipe_bin\" &&
    \"\$recipe_bin\" install \"$root_dir/distro-builder/recipes/linux-base.rhai\" \
      --build-dir \"$root_dir/RalphOS/downloads\" \
      --recipes-path \"$root_dir/distro-builder/recipes\" \
      --define \"KERNEL_VERSION=$LTS_VERSION\" \
      --define \"KERNEL_SHA256=$LTS_SHA256\" \
      --define \"KERNEL_LOCALVERSION=-ralph\"
  "

maybe_build "leviso" "-levitate" \
  bash -lc "cd \"$root_dir/leviso\" && cargo run -- build --dangerously-waste-the-users-time kernel"

echo "[step] Verify kernels"
verify_one "AcornOS" "-acorn"
verify_one "IuppiterOS" "-iuppiter"
verify_one "RalphOS" "-ralph"
verify_one "leviso" "-levitate"

echo "[done] All x86_64 kernels built and verified in .artifacts/out/*"
