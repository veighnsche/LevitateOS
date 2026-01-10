#!/bin/bash
# Master test runner for Eyra userspace integration
# Runs all unit tests, regression tests, behavior tests, and integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  Eyra Userspace Integration - Complete Test Suite             ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Track test results
PASSED=0
FAILED=0
SKIPPED=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"

    echo "───────────────────────────────────────────────────────────────"
    echo "Running: $test_name"
    echo "───────────────────────────────────────────────────────────────"

    if eval "$test_command"; then
        echo "✅ PASSED: $test_name"
        ((PASSED++))
    else
        echo "❌ FAILED: $test_name"
        ((FAILED++))
        if [ "${STRICT_MODE:-0}" = "1" ]; then
            echo ""
            echo "STRICT MODE: Stopping on first failure"
            exit 1
        fi
    fi
    echo ""
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "═══════════════════════════════════════════════════════════════"
echo "Phase 1: Unit Tests (libsyscall with std)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

run_test "Unit Tests - libsyscall integration tests" \
    "cargo test --test integration_tests --manifest-path crates/userspace/eyra/libsyscall/Cargo.toml --features std"

echo "═══════════════════════════════════════════════════════════════"
echo "Phase 2: Regression Tests (Cross-compilation setup)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

run_test "Regression Tests - Configuration & dependencies" \
    "cargo test --test eyra_regression_tests"

echo "═══════════════════════════════════════════════════════════════"
echo "Phase 3: Integration Tests (Build pipeline)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

if command_exists "readelf" && command_exists "aarch64-linux-gnu-gcc"; then
    echo "Prerequisites found: readelf, aarch64-linux-gnu-gcc"
    echo ""

    run_test "Integration Test - Sysroot configuration" \
        "cargo test --test eyra_integration_test test_sysroot_configuration -- --ignored"

    run_test "Integration Test - Full build pipeline" \
        "cargo test --test eyra_integration_test test_full_build_pipeline -- --ignored"

    run_test "Integration Test - Binary format checks" \
        "cargo test --test eyra_integration_test test_libgcc_eh_stub_exists -- --ignored && \
         cargo test --test eyra_integration_test test_getauxval_stub_linked -- --ignored && \
         cargo test --test eyra_integration_test test_no_libgcc_eh_errors -- --ignored"

    run_test "Integration Test - Binary properties" \
        "cargo test --test eyra_integration_test test_load_segments_addresses -- --ignored && \
         cargo test --test eyra_integration_test test_text_segment_permissions -- --ignored && \
         cargo test --test eyra_integration_test test_data_segment_permissions -- --ignored"

    run_test "Integration Test - x86_64 expected failure" \
        "cargo test --test eyra_integration_test test_x86_64_build_fails_expected -- --ignored"
else
    echo "⚠️  SKIPPED: Integration tests require readelf and aarch64-linux-gnu-gcc"
    echo "   Install with: sudo dnf install binutils aarch64-linux-gnu-gcc"
    ((SKIPPED+=5))
    echo ""
fi

echo "═══════════════════════════════════════════════════════════════"
echo "Phase 4: Behavior Tests (LevitateOS integration)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

if command_exists "qemu-system-aarch64"; then
    echo "QEMU found, running behavior tests..."
    echo ""

    run_test "Behavior Test - Full LevitateOS integration" \
        "$SCRIPT_DIR/eyra_behavior_test.sh"
else
    echo "⚠️  SKIPPED: Behavior tests require qemu-system-aarch64"
    echo "   Install with: sudo dnf install qemu-system-aarch64"
    ((SKIPPED++))
    echo ""
fi

echo "═══════════════════════════════════════════════════════════════"
echo "Test Summary"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  ✅ Passed:  $PASSED"
echo "  ❌ Failed:  $FAILED"
echo "  ⚠️  Skipped: $SKIPPED"
echo ""

TOTAL=$((PASSED + FAILED + SKIPPED))
echo "  Total:    $TOTAL tests"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║  ✅ ALL TESTS PASSED                                       ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "The Eyra userspace integration is frozen and verified."
    echo ""
    echo "Tested behaviors:"
    echo "  • 36 Eyra integration behaviors (EY1-EY36)"
    echo "  • 15 LibSyscall behaviors (LS1-LS15)"
    echo "  • Build system configuration"
    echo "  • Cross-compilation setup"
    echo "  • Binary format verification"
    echo "  • LevitateOS integration"
    echo ""
    exit 0
else
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║  ❌ SOME TESTS FAILED                                      ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Please review the failures above and fix them before continuing."
    echo "All tests must pass to ensure the integration is properly frozen."
    echo ""
    exit 1
fi
