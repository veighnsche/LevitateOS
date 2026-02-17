#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="${REPO_ROOT}/.artifacts/out/levitate"
KERNEL_OUTPUT_DIR="${KERNEL_OUTPUT_DIR:-${OUTPUT_DIR}}"
BUILD_STAGE_DIRNAME="${BUILD_STAGE_DIRNAME:-s01-boot}"
STAGE_OUTPUT_DIR="${STAGE_OUTPUT_DIR:-${KERNEL_OUTPUT_DIR}/${BUILD_STAGE_DIRNAME}}"

ISO_PATH="${ISO_PATH:-${STAGE_OUTPUT_DIR}/levitateos-x86_64-s01_boot.iso}"
export ISO_PATH

exec "${SCRIPT_DIR}/00Build-build.sh"
