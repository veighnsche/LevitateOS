#!/bin/sh

# Shared Stage 00 (00Build) artifact helpers for distro-variants.

need_file() {
    if [ ! -s "$1" ]; then
        echo "missing required artifact: $1" >&2
        exit 1
    fi
}

need_dir() {
    if [ ! -d "$1" ]; then
        echo "missing required directory: $1" >&2
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

prepare_live_inputs() {
    prepare_s01_boot_inputs "$@"
}

prepare_s00_build_inputs() {
    if [ "$#" -ne 2 ]; then
        echo "prepare_s00_build_inputs requires <distro_id> <output_dir>" >&2
        exit 64
    fi

    distro_id="$1"
    output_dir="$2"
    rootfs_source_path_file="${output_dir}/.live-rootfs-source.path"
    legacy_source_path_file="${output_dir}/.s01-rootfs-source.path"

    run_distro_builder artifact prepare-s00-build-inputs "$distro_id" "$output_dir" 1>&2
    rm -f "$legacy_source_path_file"

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

prepare_s01_boot_inputs() {
    if [ "$#" -ne 2 ]; then
        echo "prepare_s01_boot_inputs requires <distro_id> <output_dir>" >&2
        exit 64
    fi

    distro_id="$1"
    output_dir="$2"
    rootfs_source_path_file="${output_dir}/.live-rootfs-source.path"
    legacy_source_path_file="${output_dir}/.s01-rootfs-source.path"

    run_distro_builder artifact prepare-s01-boot-inputs "$distro_id" "$output_dir" 1>&2
    rm -f "$legacy_source_path_file"

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

build_overlayfs_erofs() {
    if [ "$#" -ne 2 ]; then
        echo "build_overlayfs_erofs requires <source_dir> <output_path>" >&2
        exit 64
    fi

    run_distro_builder artifact build-overlayfs-erofs "$1" "$2"
}
