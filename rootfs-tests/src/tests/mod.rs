//! Test modules for rootfs validation.
//!
//! Each test answers: "Can a user do X with this OS?"
//! Tests are behavioral - they DO things, not just check existence.

pub mod admin;
pub mod files;
pub mod network;
pub mod packages;
pub mod text;

use crate::container::Container;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Result of a single test.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub ensures: String,
    pub passed: bool,
    pub output: String,
    pub duration: Duration,
}

/// A test that can be run against the container.
pub trait Test: Send + Sync {
    /// Short test name.
    fn name(&self) -> &str;

    /// What this test ensures for the end user.
    fn ensures(&self) -> &str;

    /// Category for grouping.
    fn category(&self) -> &str;

    /// Run the test.
    fn run(&self, container: &Container) -> TestResult;
}

/// Helper to create a test result.
pub fn test_result(name: &str, ensures: &str, f: impl FnOnce() -> Result<String>) -> TestResult {
    let start = Instant::now();
    match f() {
        Ok(output) => TestResult {
            name: name.to_string(),
            ensures: ensures.to_string(),
            passed: true,
            output,
            duration: start.elapsed(),
        },
        Err(e) => TestResult {
            name: name.to_string(),
            ensures: ensures.to_string(),
            passed: false,
            output: format!("{:#}", e),
            duration: start.elapsed(),
        },
    }
}

/// Collect all tests.
pub fn all_tests() -> Vec<Box<dyn Test>> {
    let mut tests: Vec<Box<dyn Test>> = Vec::new();
    tests.extend(files::tests());
    tests.extend(text::tests());
    tests.extend(admin::tests());
    tests.extend(packages::tests());
    tests.extend(network::tests());
    tests
}
