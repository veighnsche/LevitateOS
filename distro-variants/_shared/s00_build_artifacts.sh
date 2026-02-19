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

prepare_stage_inputs() {
    if [ "$#" -ne 3 ]; then
        echo "prepare_stage_inputs requires <build_stage_dirname> <distro_id> <output_dir>" >&2
        exit 64
    fi

    build_stage_dirname="$1"
    distro_id="$2"
    output_dir="$3"

    case "$build_stage_dirname" in
        s00-build) stage_name="00Build"; stage_tag="s00" ;;
        s01-boot) stage_name="01Boot"; stage_tag="s01" ;;
        s02-live-tools) stage_name="02LiveTools"; stage_tag="s02" ;;
        *)
            echo "unsupported BUILD_STAGE_DIRNAME for prepare_stage_inputs: $build_stage_dirname" >&2
            exit 64
            ;;
    esac

    rootfs_source_path_file="${output_dir}/.${stage_tag}-live-rootfs-source.path"
    run_distro_builder artifact prepare-stage-inputs "$stage_name" "$distro_id" "$output_dir" 1>&2

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/${stage_tag}-live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

prepare_s02_live_tools_inputs() {
    if [ "$#" -ne 2 ]; then
        echo "prepare_s02_live_tools_inputs requires <distro_id> <output_dir>" >&2
        exit 64
    fi

    distro_id="$1"
    output_dir="$2"
    stage_tag="${STAGE_ARTIFACT_TAG:-s02}"
    rootfs_source_path_file="${output_dir}/.${stage_tag}-live-rootfs-source.path"

    run_distro_builder artifact prepare-s02-live-tools-inputs "$distro_id" "$output_dir" 1>&2

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/${stage_tag}-live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

prepare_s00_build_inputs() {
    if [ "$#" -ne 2 ]; then
        echo "prepare_s00_build_inputs requires <distro_id> <output_dir>" >&2
        exit 64
    fi

    distro_id="$1"
    output_dir="$2"
    stage_tag="${STAGE_ARTIFACT_TAG:-s00}"
    rootfs_source_path_file="${output_dir}/.${stage_tag}-live-rootfs-source.path"

    run_distro_builder artifact prepare-s00-build-inputs "$distro_id" "$output_dir" 1>&2

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/${stage_tag}-live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

prepare_s01_boot_inputs() {
    if [ "$#" -ne 2 ]; then
        echo "prepare_s01_boot_inputs requires <distro_id> <output_dir>" >&2
        exit 64
    fi

    distro_id="$1"
    output_dir="$2"
    stage_tag="${STAGE_ARTIFACT_TAG:-s01}"
    rootfs_source_path_file="${output_dir}/.${stage_tag}-live-rootfs-source.path"

    run_distro_builder artifact prepare-s01-boot-inputs "$distro_id" "$output_dir" 1>&2

    need_file "$rootfs_source_path_file"
    ROOTFS_SOURCE_DIR="$(tr -d '\n' < "$rootfs_source_path_file")"
    if [ -z "$ROOTFS_SOURCE_DIR" ]; then
        echo "invalid live rootfs source path file: $rootfs_source_path_file" >&2
        exit 1
    fi
    need_dir "$ROOTFS_SOURCE_DIR"
    need_dir "${output_dir}/${stage_tag}-live-overlay"

    printf '%s\n' "$ROOTFS_SOURCE_DIR"
}

build_overlayfs_erofs() {
    if [ "$#" -ne 2 ]; then
        echo "build_overlayfs_erofs requires <source_dir> <output_path>" >&2
        exit 64
    fi

    run_distro_builder artifact build-overlayfs-erofs "$1" "$2"
}

stage_boot_label() {
    if [ "$#" -ne 1 ]; then
        echo "stage_boot_label requires <build_stage_dirname>" >&2
        exit 64
    fi

    case "$1" in
        s00-build) printf '%s\n' "S00 Build" ;;
        s01-boot) printf '%s\n' "S01 Boot" ;;
        s02-live-tools) printf '%s\n' "S02 Live Tools" ;;
        *) echo "unsupported BUILD_STAGE_DIRNAME for stage_boot_label: $1" >&2; exit 64 ;;
    esac
}

merge_uki_cmdline() {
    if [ "$#" -ne 1 ]; then
        echo "merge_uki_cmdline requires <stage_specific_cmdline>" >&2
        exit 64
    fi

    stage_specific="$1"
    required="${STAGE_REQUIRED_KERNEL_CMDLINE:-}"

    if [ -n "$stage_specific" ] && [ -n "$required" ]; then
        merged="${stage_specific} ${required}"
    elif [ -n "$stage_specific" ]; then
        merged="$stage_specific"
    else
        merged="$required"
    fi

    printf '%s\n' "$merged"
}
