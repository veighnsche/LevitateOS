#!/bin/sh
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"

export DISTRO_ID="levitate"
exec "${REPO_ROOT}/distro-variants/_shared/00Build-build.sh"
