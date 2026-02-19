#!/bin/sh
#
# LevitateOS shared pre-commit hook for Rust submodules
#
# Installs into each submodule via: cargo xtask hooks install
#
# What it does:
#   1. Auto-fix: cargo fmt on staged .rs files (re-stages them)
#   2. Check:    cargo check (fast compilation check)
#   3. Check:    cargo clippy --no-deps (lint this crate only, not dependencies)
#   4. Test:     cargo test (unit tests only)
#
# Skippable with: git commit --no-verify
#

set -e

# ── Locate workspace root ──────────────────────────────────────────
find_workspace_root() {
    dir="$(pwd)"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/Cargo.toml" ] && grep -q '^\[workspace\]' "$dir/Cargo.toml" 2>/dev/null; then
            echo "$dir"
            return
        fi
        dir="$(dirname "$dir")"
    done
    echo ""
}

WORKSPACE_ROOT="$(find_workspace_root)"
if [ -z "$WORKSPACE_ROOT" ]; then
    echo "pre-commit: WARNING: Cannot find workspace root, skipping checks"
    exit 0
fi

# ── Determine what we're committing in ─────────────────────────────
# If cwd IS the workspace root, we're in the parent repo
# If cwd is a submodule, we have a specific crate to check
IS_PARENT_REPO=false
CRATE_NAME=""
if [ "$(pwd)" = "$WORKSPACE_ROOT" ]; then
    IS_PARENT_REPO=true
else
    if [ -f Cargo.toml ]; then
        CRATE_NAME=$(grep '^name' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    fi
fi

# Parent repo commits are just submodule pointer updates — no Rust checks needed
if [ "$IS_PARENT_REPO" = true ]; then
    echo ""
    echo ">>> Running parent-repo policy guard..."
    echo ""
    if (cd "$WORKSPACE_ROOT" && cargo xtask policy audit-legacy-bindings 2>&1); then
        echo "OK  Policy guard clean"
        exit 0
    else
        echo "FAIL Policy guard failed"
        echo ""
        echo "  Fix forbidden legacy bindings or skip with: git commit --no-verify"
        exit 1
    fi
fi

# No Cargo.toml means non-Rust submodule — skip
if [ -z "$CRATE_NAME" ]; then
    exit 0
fi

# ── Colors (if terminal) ───────────────────────────────────────────
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BOLD='' RESET=''
fi

pass() { printf "${GREEN}OK${RESET}  %s\n" "$1"; }
fail() { printf "${RED}FAIL${RESET} %s\n" "$1"; }
info() { printf "${YELLOW}>>>${RESET}  %s\n" "$1"; }

echo ""
info "Pre-commit checks for ${BOLD}${CRATE_NAME}${RESET}"
echo ""

# ── 1. Auto-format staged .rs files ───────────────────────────────
info "Formatting staged Rust files..."

STAGED_RS=$(git diff --cached --name-only --diff-filter=AM -- '*.rs' 2>/dev/null || true)

if [ -n "$STAGED_RS" ]; then
    # Format just this crate (not the whole workspace)
    if (cd "$WORKSPACE_ROOT" && cargo fmt -p "$CRATE_NAME" 2>/dev/null); then
        # Re-stage any files that were reformatted
        RESTAGED=0
        for f in $STAGED_RS; do
            if ! git diff --quiet -- "$f" 2>/dev/null; then
                git add "$f"
                RESTAGED=$((RESTAGED + 1))
            fi
        done
        if [ "$RESTAGED" -gt 0 ]; then
            pass "Formatted and re-staged $RESTAGED file(s)"
        else
            pass "Already formatted"
        fi
    else
        fail "cargo fmt failed"
        exit 1
    fi
else
    pass "No .rs files staged"
fi

# ── 2. cargo check (fast compile check) ───────────────────────────
info "Checking compilation..."

if (cd "$WORKSPACE_ROOT" && cargo check -p "$CRATE_NAME" 2>&1); then
    pass "Compiles"
else
    fail "Compilation failed"
    exit 1
fi

# ── 3. Clippy (this crate only, warnings as errors) ───────────────
info "Running clippy..."

# --no-deps: only lint this crate, not its dependencies
if (cd "$WORKSPACE_ROOT" && cargo clippy -p "$CRATE_NAME" --no-deps -- -D warnings 2>&1); then
    pass "Clippy clean"
else
    fail "Clippy found issues"
    echo ""
    echo "  Fix clippy warnings or skip with: git commit --no-verify"
    exit 1
fi

# ── 4. Fast unit tests ────────────────────────────────────────────
info "Running unit tests..."

if (cd "$WORKSPACE_ROOT" && cargo test -p "$CRATE_NAME" 2>&1); then
    pass "Tests pass"
else
    fail "Tests failed"
    echo ""
    echo "  Fix failing tests or skip with: git commit --no-verify"
    exit 1
fi

echo ""
printf "${GREEN}${BOLD}All pre-commit checks passed for ${CRATE_NAME}.${RESET}\n"
echo ""
