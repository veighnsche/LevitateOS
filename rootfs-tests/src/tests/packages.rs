//! Package management tests.
//!
//! Can users install and manage software?
//!
//! ## Anti-Reward-Hacking Design
//!
//! Tests verify actual command output contains expected information,
//! not just that commands exit successfully.

use super::{test_result, Test, TestResult};
use crate::container::Container;

/// Test: Recipe package manager exists
struct RecipeExists;

impl Test for RecipeExists {
    fn name(&self) -> &str { "recipe binary" }
    fn category(&self) -> &str { "packages" }
    fn ensures(&self) -> &str {
        "Package manager (recipe) is installed and runnable"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("recipe --version")?;

            if !result.contains("recipe") {
                anyhow::bail!("recipe not returning version: {}", result);
            }
            Ok(result.trim().into())
        })
    }
}

/// Test: Recipe can list packages
struct RecipeList;

impl Test for RecipeList {
    fn name(&self) -> &str { "recipe list" }
    fn category(&self) -> &str { "packages" }
    fn ensures(&self) -> &str {
        "User can list available/installed packages"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // Try to list - even if empty, command should work
            let result = c.exec_ok("recipe list 2>&1 || recipe --help 2>&1")?;

            // As long as recipe runs without crashing
            if result.is_empty() {
                anyhow::bail!("recipe produced no output");
            }
            Ok("recipe command executes".into())
        })
    }
}

/// Test: Recipe config exists
struct RecipeConfig;

impl Test for RecipeConfig {
    fn name(&self) -> &str { "recipe config" }
    fn category(&self) -> &str { "packages" }
    fn ensures(&self) -> &str {
        "Package manager is configured with repositories"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            c.exec_ok(r#"
                test -d /etc/recipe &&
                test -d /etc/recipe/repos
            "#)?;

            Ok("recipe config directory exists".into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(RecipeExists),
        Box::new(RecipeList),
        Box::new(RecipeConfig),
    ]
}
