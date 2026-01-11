//! TEAM_424: Syscall Conformance Test Suite
//!
//! Tests actual kernel syscall behavior against Linux ABI expectations.
//! Each test makes real syscalls and verifies the kernel returns correct results.
//!
//! Output format: TEST:<category>:<name>:PASS or TEST:<category>:<name>:FAIL:<reason>
//! This format allows xtask to parse results programmatically.

use std::process::exit;

mod fs;
mod mem;
mod process;
mod sync;

/// Test result for structured output
pub struct TestResult {
    pub category: &'static str,
    pub name: &'static str,
    pub passed: bool,
    pub reason: Option<String>,
}

impl TestResult {
    pub fn pass(category: &'static str, name: &'static str) -> Self {
        Self {
            category,
            name,
            passed: true,
            reason: None,
        }
    }

    pub fn fail(category: &'static str, name: &'static str, reason: impl Into<String>) -> Self {
        Self {
            category,
            name,
            passed: false,
            reason: Some(reason.into()),
        }
    }

    pub fn print(&self) {
        if self.passed {
            println!("TEST:{}:{}:PASS", self.category, self.name);
        } else {
            println!(
                "TEST:{}:{}:FAIL:{}",
                self.category,
                self.name,
                self.reason.as_deref().unwrap_or("unknown")
            );
        }
    }
}

/// Macro for defining conformance tests
#[macro_export]
macro_rules! conformance_test {
    ($category:expr, $name:expr, $body:block) => {{
        let result = std::panic::catch_unwind(|| $body);
        match result {
            Ok(Ok(())) => $crate::TestResult::pass($category, $name),
            Ok(Err(e)) => $crate::TestResult::fail($category, $name, e),
            Err(_) => $crate::TestResult::fail($category, $name, "panic"),
        }
    }};
}

/// Assert that a syscall returns success (>= 0)
#[macro_export]
macro_rules! assert_syscall_ok {
    ($result:expr) => {
        if $result < 0 {
            return Err(format!("syscall returned error: {}", $result));
        }
    };
    ($result:expr, $msg:expr) => {
        if $result < 0 {
            return Err(format!("{}: syscall returned error: {}", $msg, $result));
        }
    };
}

/// Assert that a syscall returns a specific error
#[macro_export]
macro_rules! assert_syscall_err {
    ($result:expr, $expected:expr) => {
        let expected_neg = -($expected as i64);
        if $result != expected_neg {
            return Err(format!(
                "expected error {} ({}), got {}",
                $expected, expected_neg, $result
            ));
        }
    };
}

/// Assert equality with descriptive error
#[macro_export]
macro_rules! assert_eq_desc {
    ($left:expr, $right:expr, $desc:expr) => {
        if $left != $right {
            return Err(format!("{}: expected {:?}, got {:?}", $desc, $right, $left));
        }
    };
}

fn main() {
    println!("=== LevitateOS Syscall Conformance Tests ===");
    println!("Testing kernel syscall behavior against Linux ABI\n");

    let mut results: Vec<TestResult> = Vec::new();

    // Run filesystem tests
    println!("--- File System Tests ---");
    results.extend(fs::run_tests());

    // Run memory tests
    println!("\n--- Memory Tests ---");
    results.extend(mem::run_tests());

    // Run process tests
    println!("\n--- Process Tests ---");
    results.extend(process::run_tests());

    // Run synchronization tests
    println!("\n--- Synchronization Tests ---");
    results.extend(sync::run_tests());

    // Print all results in structured format
    println!("\n=== Results ===");
    for result in &results {
        result.print();
    }

    // Summary
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let total = results.len();

    println!("\n=== Summary ===");
    println!("Total:  {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        println!("\nSYSCALL_CONFORMANCE:FAILED");
        exit(1);
    } else {
        println!("\nSYSCALL_CONFORMANCE:PASSED");
        exit(0);
    }
}
