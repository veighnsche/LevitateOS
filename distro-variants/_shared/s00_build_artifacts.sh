#!/bin/sh

# Shared Stage 00 (00Build) artifact helpers for distro-variants.

need_file() {
    if [ ! -s "$1" ]; then
        echo "missing required artifact: $1" >&2
        exit 1
    fi
}

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "missing required command: $1" >&2
        exit 1
    fi
}

run_distro_builder() {
    if [ -n "${DISTRO_BUILDER_BIN:-}" ] && [ -x "${DISTRO_BUILDER_BIN}" ]; then
        "${DISTRO_BUILDER_BIN}" "$@"
        return
    fi

    if command -v distro-builder >/dev/null 2>&1; then
        distro-builder "$@"
        return
    fi

    need_cmd cargo
    cargo run -q -p distro-builder --bin distro-builder -- "$@"
}

build_rootfs_erofs() {
    if [ "$#" -ne 2 ]; then
        echo "build_rootfs_erofs requires <source_dir> <output_path>" >&2
        exit 64
    fi

    run_distro_builder artifact build-rootfs-erofs "$1" "$2"
}

prepare_stage00_minimal_rootfs_dir() {
    if [ "$#" -ne 1 ]; then
        echo "prepare_stage00_minimal_rootfs_dir requires <rootfs_dir>" >&2
        exit 64
    fi

    rootfs_dir="$1"
    rm -rf "$rootfs_dir"
    mkdir -p "$rootfs_dir"

    # Deterministic Stage 00 minimal rootfs marker.
    printf 'stage=00Build\nprofile=minimal\n' >"${rootfs_dir}/.stage00-minimal-rootfs"
}

build_overlayfs_erofs() {
    if [ "$#" -ne 2 ]; then
        echo "build_overlayfs_erofs requires <source_dir> <output_path>" >&2
        exit 64
    fi

    run_distro_builder artifact build-overlayfs-erofs "$1" "$2"
}
