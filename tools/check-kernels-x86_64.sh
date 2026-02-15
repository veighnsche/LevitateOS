#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

usage() {
  cat <<'EOF'
Usage:
  tools/check-kernels-x86_64.sh [distro]

Distros:
  leviso | AcornOS | IuppiterOS | RalphOS

What it checks (per distro):
  - .artifacts/out/<distro>/kernel-build/include/config/kernel.release
  - .artifacts/out/<distro>/staging/boot/vmlinuz
  - modules dir exists under staging/{lib,usr/lib}/modules/<kernel.release>

Exit codes:
  0  all requested distros are built+verified
  1  one or more requested distros missing/invalid
  2  usage error
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

want_distro="${1:-}"
if [[ -n "$want_distro" && $# -ne 1 ]]; then
  echo "[error] Too many arguments." >&2
  usage >&2
  exit 2
fi

verify_one() {
  local distro_dir="$1"
  local want_suffix="$2"
  local out_dir="$root_dir/.artifacts/out/$distro_dir"
  local rel_file="$out_dir/kernel-build/include/config/kernel.release"
  local vmlinuz="$out_dir/staging/boot/vmlinuz"

  if [[ ! -f "$rel_file" ]]; then
    echo "[missing] $distro_dir: $rel_file"
    return 1
  fi
  if [[ ! -f "$vmlinuz" ]]; then
    echo "[missing] $distro_dir: $vmlinuz"
    return 1
  fi

  local rel
  rel="$(tr -d '\n' < "$rel_file")"
  if [[ "$rel" != *"$want_suffix" ]]; then
    echo "[bad] $distro_dir: kernel.release '$rel' does not end with '$want_suffix'"
    return 1
  fi

  if [[ ! -d "$out_dir/staging/lib/modules/$rel" && ! -d "$out_dir/staging/usr/lib/modules/$rel" ]]; then
    echo "[missing] $distro_dir: modules dir for '$rel' under staging/{lib,usr/lib}/modules/"
    return 1
  fi

  echo "[ok] $distro_dir: $rel"
  return 0
}

declare -A suffix=(
  ["AcornOS"]="-acorn"
  ["IuppiterOS"]="-iuppiter"
  ["RalphOS"]="-ralph"
  ["leviso"]="-levitate"
)

distros=( "leviso" "AcornOS" "IuppiterOS" "RalphOS" )
if [[ -n "$want_distro" ]]; then
  if [[ -z "${suffix[$want_distro]:-}" ]]; then
    echo "[error] Unknown distro '$want_distro'." >&2
    usage >&2
    exit 2
  fi
  distros=( "$want_distro" )
fi

fail=0
for d in "${distros[@]}"; do
  if ! verify_one "$d" "${suffix[$d]}"; then
    fail=1
  fi
done

exit "$fail"

