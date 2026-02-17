#!/bin/sh
set -eu

# Variant-native Stage 00 ISO assembly hook for LevitateOS.
# Kernel artifacts are treated as immutable inputs here.

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"

. "${REPO_ROOT}/distro-variants/_shared/s00_build_artifacts.sh"

OUTPUT_DIR="${REPO_ROOT}/.artifacts/out/levitate"
DISTRO_ID="levitate"
KERNEL_OUTPUT_DIR="${KERNEL_OUTPUT_DIR:-${OUTPUT_DIR}}"
BUILD_STAGE_DIRNAME="${BUILD_STAGE_DIRNAME:-s00-build}"
STAGE_OUTPUT_DIR="${STAGE_OUTPUT_DIR:-${KERNEL_OUTPUT_DIR}/${BUILD_STAGE_DIRNAME}}"
STAGE_ARTIFACT_TAG="${STAGE_ARTIFACT_TAG:-$(printf '%s' "$BUILD_STAGE_DIRNAME" | cut -c1-3)}"
KERNEL_RELEASE_PATH="${KERNEL_RELEASE_PATH:-${KERNEL_OUTPUT_DIR}/kernel-build/include/config/kernel.release}"
KERNEL_IMAGE_PATH="${KERNEL_IMAGE_PATH:-${KERNEL_OUTPUT_DIR}/staging/boot/vmlinuz}"
ISO_PATH="${ISO_PATH:-${STAGE_OUTPUT_DIR}/levitateos-x86_64-s00_build.iso}"

ROOTFS_PATH="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-filesystem.erofs"
INITRAMFS_LIVE_PATH="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-initramfs-live.cpio.gz"
LIVE_OVERLAY_DIR="${STAGE_OUTPUT_DIR}/live-overlay"
LIVE_OVERLAY_IMAGE="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-overlayfs.erofs"
INIT_TEMPLATE="${REPO_ROOT}/tools/recinit/templates/init_tiny.template"
BUSYBOX_URL="${BUSYBOX_URL:-https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox}"
BUSYBOX_PATH="${BUSYBOX_PATH:-${REPO_ROOT}/.artifacts/tools/busybox-static}"

ISO_LABEL="LEVITATEOS"
OS_NAME="LevitateOS"
OS_ID="levitateos"
OS_VERSION="1.0"

need_file "$KERNEL_RELEASE_PATH"
need_file "$KERNEL_IMAGE_PATH"
mkdir -p "$STAGE_OUTPUT_DIR"
rm -f "${STAGE_OUTPUT_DIR}/filesystem.erofs" \
      "${STAGE_OUTPUT_DIR}/initramfs-live.cpio.gz" \
      "${STAGE_OUTPUT_DIR}/overlayfs.erofs"

if [ "$BUILD_STAGE_DIRNAME" = "s00-build" ]; then
    ROOTFS_SOURCE_DIR="$(prepare_s00_build_inputs "$DISTRO_ID" "$STAGE_OUTPUT_DIR")"
else
    ROOTFS_SOURCE_DIR="$(prepare_s01_boot_inputs "$DISTRO_ID" "$STAGE_OUTPUT_DIR")"
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
if [ ! -s "$BUSYBOX_PATH" ]; then
    need_cmd curl
    mkdir -p "$(dirname "$BUSYBOX_PATH")"
    curl -L -o "$BUSYBOX_PATH" --progress-bar "$BUSYBOX_URL"
    chmod +x "$BUSYBOX_PATH"
fi
need_file "$BUSYBOX_PATH"
rm -f "$INITRAMFS_LIVE_PATH"
STAGE_INIT_TEMPLATE="${STAGE_OUTPUT_DIR}/${STAGE_ARTIFACT_TAG}-init.template"
cp "$INIT_TEMPLATE" "$STAGE_INIT_TEMPLATE"
printf '\n# stage-artifact-tag: %s\n' "$STAGE_ARTIFACT_TAG" >> "$STAGE_INIT_TEMPLATE"
if command -v recinit >/dev/null 2>&1; then
    recinit build-tiny \
        --modules-dir "$MODULES_DIR" \
        --busybox "$BUSYBOX_PATH" \
        --template "$STAGE_INIT_TEMPLATE" \
        --output "$INITRAMFS_LIVE_PATH" \
        --iso-label "$ISO_LABEL" \
        --rootfs-path "live/filesystem.erofs"
else
    need_cmd cargo
    cargo run -q -p recinit -- build-tiny \
        --modules-dir "$MODULES_DIR" \
        --busybox "$BUSYBOX_PATH" \
        --template "$STAGE_INIT_TEMPLATE" \
        --output "$INITRAMFS_LIVE_PATH" \
        --iso-label "$ISO_LABEL" \
        --rootfs-path "live/filesystem.erofs"
fi

need_file "$INITRAMFS_LIVE_PATH"

need_dir "$LIVE_OVERLAY_DIR"
rm -f "$LIVE_OVERLAY_IMAGE"
build_overlayfs_erofs "$LIVE_OVERLAY_DIR" "$LIVE_OVERLAY_IMAGE"
need_file "$LIVE_OVERLAY_IMAGE"

mkdir -p "$(dirname "$ISO_PATH")"

ISO_TMP="${ISO_PATH}.tmp"
ISO_SHA="${ISO_PATH%.iso}.sha512"
ISO_TMP_SHA_ALT1="${ISO_TMP}.sha512"
ISO_TMP_SHA_ALT2="${ISO_TMP%.*}.sha512"

rm -f "$ISO_TMP" "$ISO_TMP_SHA_ALT1" "$ISO_TMP_SHA_ALT2"

set -- \
    --kernel "$KERNEL_IMAGE_PATH" \
    --initrd "$INITRAMFS_LIVE_PATH" \
    --rootfs "$ROOTFS_PATH" \
    --label "$ISO_LABEL" \
    --output "$ISO_TMP" \
    --os-name "$OS_NAME" \
    --os-id "$OS_ID" \
    --os-version "$OS_VERSION" \
    --build-uki "LevitateOS:video=1920x1080:levitateos-live.efi" \
    --build-uki "LevitateOS (Emergency):emergency:levitateos-emergency.efi" \
    --build-uki "LevitateOS (Debug):debug:levitateos-debug.efi" \
    --overlay-image "$LIVE_OVERLAY_IMAGE"

if command -v reciso >/dev/null 2>&1; then
    reciso "$@"
else
    need_cmd cargo
    cargo run -q -p reciso -- "$@"
fi

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

echo "Built ISO: $ISO_PATH"
