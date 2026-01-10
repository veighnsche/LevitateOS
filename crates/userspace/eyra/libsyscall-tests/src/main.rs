//! Integration test runner for libsyscall with std support
//!
//! TEAM_380: Migrated from libsyscall/tests/ to enable std support via Eyra.
//! This binary runs all integration tests and reports results with colored output.

use std::process::exit;

mod errno_tests;
mod integration_tests;
mod memory_tests;
mod path_validation;
mod time_tests;

fn main() {
    println!("=== LevitateOS libsyscall Integration Tests ===\n");

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;

    // Run errno tests
    println!("Running errno tests...");
    let (p, f) = errno_tests::run_tests();
    passed += p;
    failed += f;
    total += p + f;

    // Run integration tests
    println!("\nRunning integration tests...");
    let (p, f) = integration_tests::run_tests();
    passed += p;
    failed += f;
    total += p + f;

    // Run memory tests
    println!("\nRunning memory tests...");
    let (p, f) = memory_tests::run_tests();
    passed += p;
    failed += f;
    total += p + f;

    // Run path validation tests
    println!("\nRunning path validation tests...");
    let (p, f) = path_validation::run_tests();
    passed += p;
    failed += f;
    total += p + f;

    // Run time tests
    println!("\nRunning time tests...");
    let (p, f) = time_tests::run_tests();
    passed += p;
    failed += f;
    total += p + f;

    // Summary
    println!("\n=== Test Summary ===");
    println!("Total:  {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        println!("\n❌ TESTS FAILED");
        exit(1);
    } else {
        println!("\n✅ ALL TESTS PASSED");
        exit(0);
    }
}
