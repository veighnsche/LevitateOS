#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"

. "${REPO_ROOT}/distro-variants/_shared/s00_build_artifacts.sh"

: "${DISTRO_ID:?missing DISTRO_ID}"
: "${IDENTITY_OS_NAME:?missing IDENTITY_OS_NAME}"
: "${IDENTITY_OS_ID:?missing IDENTITY_OS_ID}"
: "${IDENTITY_OS_VERSION:?missing IDENTITY_OS_VERSION}"
: "${IDENTITY_ISO_LABEL:?missing IDENTITY_ISO_LABEL}"
: "${S00_LIVE_UKI_FILENAME:?missing S00_LIVE_UKI_FILENAME}"
: "${S00_EMERGENCY_UKI_FILENAME:?missing S00_EMERGENCY_UKI_FILENAME}"
: "${S00_DEBUG_UKI_FILENAME:?missing S00_DEBUG_UKI_FILENAME}"

OUTPUT_DIR="${REPO_ROOT}/.artifacts/out/${DISTRO_ID}"
KERNEL_OUTPUT_DIR="${KERNEL_OUTPUT_DIR:-${REPO_ROOT}/.artifacts/kernel/${DISTRO_ID}/current}"
BUILD_STAGE_DIRNAME="${BUILD_STAGE_DIRNAME:-s00-build}"
STAGE_ROOT_DIR="${STAGE_ROOT_DIR:-${OUTPUT_DIR}/${BUILD_STAGE_DIRNAME}}"
STAGE_RUN_DIR="${STAGE_RUN_DIR:-${STAGE_OUTPUT_DIR:-${STAGE_ROOT_DIR}}}"
STAGE_OUTPUT_DIR="${STAGE_OUTPUT_DIR:-${STAGE_RUN_DIR}}"
STAGE_ARTIFACT_TAG="${STAGE_ARTIFACT_TAG:-$(printf '%s' "$BUILD_STAGE_DIRNAME" | cut -c1-3)}"
KERNEL_RELEASE_PATH="${KERNEL_RELEASE_PATH:-${KERNEL_OUTPUT_DIR}/kernel-build/include/config/kernel.release}"
KERNEL_IMAGE_PATH="${KERNEL_IMAGE_PATH:-${KERNEL_OUTPUT_DIR}/staging/boot/vmlinuz}"
ISO_FILENAME="${ISO_FILENAME:-${IDENTITY_OS_ID}-x86_64-s00_build.iso}"
ISO_PATH="${ISO_PATH:-${STAGE_OUTPUT_DIR}/${ISO_FILENAME}}"

ROOTFS_PATH="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-filesystem.erofs"
INITRAMFS_LIVE_PATH="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-initramfs-live.cpio.gz"
LIVE_OVERLAY_DIR="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-live-overlay"
LIVE_OVERLAY_IMAGE="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-overlayfs.erofs"
INIT_TEMPLATE="${REPO_ROOT}/tools/recinit/templates/init_tiny.template"
BUSYBOX_URL="${BUSYBOX_URL:-https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox}"
BUSYBOX_PATH="${BUSYBOX_PATH:-${REPO_ROOT}/.artifacts/tools/busybox-static}"
BUSYBOX_SHA256="${BUSYBOX_SHA256:-6e123e7f3202a8c1e9b1f94d8941580a25135382b99e8d3e34fb858bba311348}"

ISO_LABEL="${IDENTITY_ISO_LABEL}"
OS_NAME="${IDENTITY_OS_NAME}"
OS_ID="${IDENTITY_OS_ID}"
OS_VERSION="${IDENTITY_OS_VERSION}"
STAGE_BOOT_LABEL="$(stage_boot_label "$BUILD_STAGE_DIRNAME")"
BUILD_LOG_PATH="${STAGE_OUTPUT_DIR}/build-log.json"
BUILD_STARTED_AT_UTC="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
BUILD_FINISHED_AT_UTC=""
BUILD_STATUS="failed"

TMP_EMPTY_OVERLAY_DIR=""
TMP_STAGE_INIT_TEMPLATE=""
TMP_STAGE_INIT_TEMPLATE_EDIT=""
cleanup_tmp_artifacts() {
    if [ -n "${TMP_EMPTY_OVERLAY_DIR:-}" ] && [ -d "${TMP_EMPTY_OVERLAY_DIR}" ]; then
        rm -rf "${TMP_EMPTY_OVERLAY_DIR}"
    fi
    if [ -n "${TMP_STAGE_INIT_TEMPLATE:-}" ] && [ -f "${TMP_STAGE_INIT_TEMPLATE}" ]; then
        rm -f "${TMP_STAGE_INIT_TEMPLATE}"
    fi
    if [ -n "${TMP_STAGE_INIT_TEMPLATE_EDIT:-}" ] && [ -f "${TMP_STAGE_INIT_TEMPLATE_EDIT}" ]; then
        rm -f "${TMP_STAGE_INIT_TEMPLATE_EDIT}"
    fi
}

json_escape() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

file_size_bytes() {
    target="$1"
    if [ ! -f "$target" ]; then
        printf '0'
        return 0
    fi
    if command -v stat >/dev/null 2>&1; then
        stat -c '%s' "$target" 2>/dev/null && return 0
    fi
    wc -c < "$target" | tr -d ' '
}

write_build_log_json() {
    BUILD_FINISHED_AT_UTC="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    rootfs_size="$(file_size_bytes "$ROOTFS_PATH")"
    initramfs_size="$(file_size_bytes "$INITRAMFS_LIVE_PATH")"
    overlay_size="$(file_size_bytes "$LIVE_OVERLAY_IMAGE")"
    iso_size="$(file_size_bytes "$ISO_PATH")"
    sha_size="$(file_size_bytes "$ISO_SHA")"
    cat > "$BUILD_LOG_PATH" <<EOF
{
  "schema": 1,
  "component": "s00-build-shared",
  "distro_id": "$(json_escape "$DISTRO_ID")",
  "stage_dirname": "$(json_escape "$BUILD_STAGE_DIRNAME")",
  "stage_artifact_tag": "$(json_escape "$STAGE_ARTIFACT_TAG")",
  "status": "$(json_escape "$BUILD_STATUS")",
  "started_at_utc": "$(json_escape "$BUILD_STARTED_AT_UTC")",
  "finished_at_utc": "$(json_escape "$BUILD_FINISHED_AT_UTC")",
  "run_dir": "$(json_escape "$STAGE_OUTPUT_DIR")",
  "artifacts": {
    "rootfs_erofs": { "path": "$(json_escape "$ROOTFS_PATH")", "size_bytes": $rootfs_size },
    "initramfs_live": { "path": "$(json_escape "$INITRAMFS_LIVE_PATH")", "size_bytes": $initramfs_size },
    "overlay_erofs": { "path": "$(json_escape "$LIVE_OVERLAY_IMAGE")", "size_bytes": $overlay_size },
    "iso": { "path": "$(json_escape "$ISO_PATH")", "size_bytes": $iso_size },
    "iso_sha512": { "path": "$(json_escape "$ISO_SHA")", "size_bytes": $sha_size }
  }
}
EOF
}

on_exit() {
    write_build_log_json
    cleanup_tmp_artifacts
}
trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

need_file "$KERNEL_RELEASE_PATH"
need_file "$KERNEL_IMAGE_PATH"
mkdir -p "$STAGE_OUTPUT_DIR"
ROOTFS_SOURCE_DIR="$(prepare_stage_inputs "$BUILD_STAGE_DIRNAME" "$DISTRO_ID" "$STAGE_OUTPUT_DIR")"
if [ -d "$LIVE_OVERLAY_DIR" ]; then
    rm -rf "$LIVE_OVERLAY_DIR"
fi

rm -f "$ROOTFS_PATH"
build_rootfs_erofs "$ROOTFS_SOURCE_DIR" "$ROOTFS_PATH"
need_file "$ROOTFS_PATH"

KERNEL_RELEASE="$(tr -d '\n' < "$KERNEL_RELEASE_PATH")"
MODULES_DIR="${KERNEL_OUTPUT_DIR}/staging/usr/lib/modules/${KERNEL_RELEASE}"
if [ ! -d "$MODULES_DIR" ]; then
    MODULES_DIR="${KERNEL_OUTPUT_DIR}/staging/lib/modules/${KERNEL_RELEASE}"
fi
if [ ! -d "$MODULES_DIR" ]; then
    echo "missing modules dir for recinit: $MODULES_DIR" >&2
    exit 1
fi

need_file "$INIT_TEMPLATE"
calc_sha256() {
    target="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$target" | awk '{print $1}'
        return 0
    fi
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$target" | awk '{print $1}'
        return 0
    fi
    echo "missing required command: sha256sum or shasum" >&2
    exit 1
}

verify_busybox_sha256() {
    target="$1"
    if [ -z "$BUSYBOX_SHA256" ]; then
        echo "missing BUSYBOX_SHA256 for '$target'; set BUSYBOX_SHA256 or pre-seed verified binary at BUSYBOX_PATH" >&2
        exit 1
    fi
    actual="$(calc_sha256 "$target")"
    if [ "$actual" != "$BUSYBOX_SHA256" ]; then
        echo "busybox sha256 mismatch for '$target': expected '$BUSYBOX_SHA256', got '$actual'" >&2
        exit 1
    fi
}

if [ ! -s "$BUSYBOX_PATH" ]; then
    need_cmd curl
    mkdir -p "$(dirname "$BUSYBOX_PATH")"
    curl --fail --show-error --location --retry 3 --retry-delay 2 --progress-bar \
        -o "$BUSYBOX_PATH" "$BUSYBOX_URL"
    chmod +x "$BUSYBOX_PATH"
fi
need_file "$BUSYBOX_PATH"
verify_busybox_sha256 "$BUSYBOX_PATH"

rm -f "$INITRAMFS_LIVE_PATH"
TMP_STAGE_INIT_TEMPLATE="$(mktemp "${STAGE_OUTPUT_DIR}/.tmp-${STAGE_ARTIFACT_TAG}-init.XXXXXX")"
cp "$INIT_TEMPLATE" "$TMP_STAGE_INIT_TEMPLATE"
printf '\n# stage-artifact-tag: %s\n' "$STAGE_ARTIFACT_TAG" >> "$TMP_STAGE_INIT_TEMPLATE"
expected_init_msg="msg \"${OS_NAME} ${STAGE_BOOT_LABEL} initramfs starting...\""
placeholder_init_msg='msg "LevitateOS initramfs starting..."'
if grep -Fq "$placeholder_init_msg" "$TMP_STAGE_INIT_TEMPLATE"; then
    TMP_STAGE_INIT_TEMPLATE_EDIT="${TMP_STAGE_INIT_TEMPLATE}.edit"
    sed "s|$placeholder_init_msg|$expected_init_msg|" \
        "$TMP_STAGE_INIT_TEMPLATE" > "$TMP_STAGE_INIT_TEMPLATE_EDIT"
    mv -f "$TMP_STAGE_INIT_TEMPLATE_EDIT" "$TMP_STAGE_INIT_TEMPLATE"
fi
if ! grep -Fq "$expected_init_msg" "$TMP_STAGE_INIT_TEMPLATE"; then
    echo "failed to materialize init template banner; expected line not found: $expected_init_msg" >&2
    exit 1
fi

need_cmd cargo
cargo run -q -p recinit -- build-tiny \
    --modules-dir "$MODULES_DIR" \
    --busybox "$BUSYBOX_PATH" \
    --template "$TMP_STAGE_INIT_TEMPLATE" \
    --output "$INITRAMFS_LIVE_PATH" \
    --iso-label "$ISO_LABEL" \
    --rootfs-path "live/filesystem.erofs"
need_file "$INITRAMFS_LIVE_PATH"

TMP_EMPTY_OVERLAY_DIR="$(mktemp -d "${STAGE_OUTPUT_DIR}/.tmp-${STAGE_ARTIFACT_TAG}-overlay.XXXXXX")"

rm -f "$LIVE_OVERLAY_IMAGE"
build_overlayfs_erofs "$TMP_EMPTY_OVERLAY_DIR" "$LIVE_OVERLAY_IMAGE"
need_file "$LIVE_OVERLAY_IMAGE"

mkdir -p "$(dirname "$ISO_PATH")"

ISO_TMP="${ISO_PATH}.tmp"
ISO_SHA="${ISO_PATH%.iso}.sha512"
ISO_TMP_SHA_ALT1="${ISO_TMP}.sha512"
ISO_TMP_SHA_ALT2="${ISO_TMP%.*}.sha512"

rm -f "$ISO_TMP" "$ISO_TMP_SHA_ALT1" "$ISO_TMP_SHA_ALT2"

LIVE_UKI_CMDLINE="$(merge_uki_cmdline "${S00_LIVE_CMDLINE-}")"
EMERGENCY_UKI_CMDLINE="$(merge_uki_cmdline "emergency")"
DEBUG_UKI_CMDLINE="$(merge_uki_cmdline "debug")"

set -- \
    --kernel "$KERNEL_IMAGE_PATH" \
    --initrd "$INITRAMFS_LIVE_PATH" \
    --rootfs "$ROOTFS_PATH" \
    --label "$ISO_LABEL" \
    --output "$ISO_TMP" \
    --os-name "$OS_NAME" \
    --os-id "$OS_ID" \
    --os-version "$OS_VERSION" \
    --build-uki "${OS_NAME} ${STAGE_BOOT_LABEL}:${LIVE_UKI_CMDLINE}:${S00_LIVE_UKI_FILENAME}" \
    --build-uki "${OS_NAME} ${STAGE_BOOT_LABEL} (Emergency):${EMERGENCY_UKI_CMDLINE}:${S00_EMERGENCY_UKI_FILENAME}" \
    --build-uki "${OS_NAME} ${STAGE_BOOT_LABEL} (Debug):${DEBUG_UKI_CMDLINE}:${S00_DEBUG_UKI_FILENAME}" \
    --overlay-image "$LIVE_OVERLAY_IMAGE"

set -- "$@" --live-payload-layout iso-files

need_cmd cargo
cargo run -q -p reciso -- "$@"

if [ ! -f "$ISO_TMP" ]; then
    echo "reciso finished without producing ISO: $ISO_TMP" >&2
    exit 1
fi

mv -f "$ISO_TMP" "$ISO_PATH"

if [ -f "$ISO_TMP_SHA_ALT1" ]; then
    mv -f "$ISO_TMP_SHA_ALT1" "$ISO_SHA"
elif [ -f "$ISO_TMP_SHA_ALT2" ]; then
    mv -f "$ISO_TMP_SHA_ALT2" "$ISO_SHA"
fi

BUILD_STATUS="success"
printf '%s\n' "Artifact summary:"
printf '  %s (%s bytes)\n' "$ROOTFS_PATH" "$(file_size_bytes "$ROOTFS_PATH")"
printf '  %s (%s bytes)\n' "$INITRAMFS_LIVE_PATH" "$(file_size_bytes "$INITRAMFS_LIVE_PATH")"
printf '  %s (%s bytes)\n' "$LIVE_OVERLAY_IMAGE" "$(file_size_bytes "$LIVE_OVERLAY_IMAGE")"
printf '  %s (%s bytes)\n' "$ISO_PATH" "$(file_size_bytes "$ISO_PATH")"
if [ -f "$ISO_SHA" ]; then
    printf '  %s (%s bytes)\n' "$ISO_SHA" "$(file_size_bytes "$ISO_SHA")"
fi
printf '  %s\n' "$BUILD_LOG_PATH"

echo "Built ISO: $ISO_PATH"
