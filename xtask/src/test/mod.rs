use anyhow::{bail, Result};
use clap::Subcommand;

use crate::GlobalArgs;

mod behavior;
mod helpers;

#[derive(Subcommand)]
pub enum TestCommand {
    /// Run golden file tests
    Golden,

    /// Run behavior tests (LevitateOS VM)
    Behavior {
        /// Run only tests in this category
        #[clap(long, short = 'c')]
        category: Option<String>,

        /// Run only this specific test
        #[clap(long, short = 't')]
        test: Option<String>,

        /// List available tests without running
        #[clap(long)]
        list: bool,

        /// Show detailed output
        #[clap(long, short = 'v')]
        verbose: bool,

        /// Login as this user (default: root)
        #[clap(long, default_value = "root")]
        user: String,
    },

    /// Quick test: rebuild, boot, run levitate-test, return results
    Quick {
        /// Skip rebuild (use existing initramfs)
        #[clap(long)]
        no_build: bool,

        /// Login as this user (default: live)
        #[clap(long, default_value = "live")]
        user: String,
    },
}

pub fn run(cmd: &TestCommand, global: &GlobalArgs) -> Result<()> {
    match cmd {
        TestCommand::Golden => run_golden_tests(global),
        TestCommand::Behavior {
            category,
            test,
            list,
            verbose,
            user,
        } => run_behavior_tests(category.as_deref(), test.as_deref(), *list, *verbose, user),
        TestCommand::Quick { no_build, user } => run_quick_test(*no_build, user),
    }
}

fn run_golden_tests(global: &GlobalArgs) -> Result<()> {
    crate::common::info_println(global.quiet, "Running golden file tests...");
    println!("Golden tests not yet implemented");
    Ok(())
}

fn run_behavior_tests(
    category: Option<&str>,
    test_name: Option<&str>,
    list: bool,
    verbose: bool,
    user: &str,
) -> Result<()> {
    use behavior::registry::default_registry;

    let registry = default_registry();

    // List mode - just show available tests
    if list {
        println!("=== Available Behavior Tests ===\n");
        for cat in registry.categories() {
            println!("Category: {}", cat);
            for test in registry.by_category(cat) {
                println!(
                    "  {} - {} (phase {})",
                    test.name(),
                    test.description(),
                    test.phase()
                );
            }
            println!();
        }
        return Ok(());
    }

    println!("=== LevitateOS Behavior Tests ===");

    // Start test runner
    let mut runner = behavior::TestRunner::new(verbose)?;

    // Run tests
    if let Some(name) = test_name {
        // Run single test
        if let Some(test) = registry.get(name) {
            // Need to login first for non-boot tests
            if test.phase() > 0 {
                runner.wait_for_boot_and_login(user)?;
            }
            let result = runner.run_test(test.as_ref());
            println!(
                "[{}] {} - {}",
                if result.passed { "PASS" } else { "FAIL" },
                result.test_name,
                result.error.as_deref().unwrap_or("ok")
            );
        } else {
            anyhow::bail!("Test not found: {}", name);
        }
    } else if let Some(cat) = category {
        // Run category
        runner.wait_for_boot_and_login(user)?;
        for test in registry.by_category(cat) {
            let result = runner.run_test(test.as_ref());
            println!(
                "[{}] {}",
                if result.passed { "PASS" } else { "FAIL" },
                result.test_name
            );
        }
    } else {
        // Run all tests
        runner.run_all(&registry, user)?;
    }

    // Summarize and cleanup
    let all_passed = runner.summarize()?;
    runner.stop()?;

    if all_passed {
        Ok(())
    } else {
        anyhow::bail!("Some tests failed")
    }
}

fn run_quick_test(no_build: bool, user: &str) -> Result<()> {
    use helpers::TestVm;
    use std::process::Command;

    // Step 1: Build initramfs (unless skipped)
    if !no_build {
        println!("=== Building initramfs ===");
        let status = Command::new("cargo")
            .args(["run", "--bin", "builder", "--", "initramfs"])
            .status()?;
        if !status.success() {
            bail!("Failed to build initramfs");
        }
        println!();
    }

    // Step 2: Start VM (boots to levitate-test.target which runs tests automatically)
    println!("=== Starting VM (test mode) ===");
    let _ = user; // User parameter not needed for automated test mode
    let mut vm = TestVm::start_levitate()?;

    // Step 3: Wait for test to complete (look for summary line)
    if !vm.wait_for_pattern(r"passed:\d+ failed:\d+", 90)? {
        let output = vm.read_output()?;
        vm.stop()?;
        bail!(
            "Timeout waiting for test results.\nOutput:\n{}",
            output.chars().take(4000).collect::<String>()
        );
    }

    // Get test output and extract just the test results
    let test_output = vm.read_output()?;

    // Extract lines between ---TESTS--- and end
    let results: String = test_output
        .lines()
        .skip_while(|l| !l.contains("---TESTS---"))
        .collect::<Vec<_>>()
        .join("\n");

    println!("{}", results);

    // Check for failures
    let all_passed = test_output.contains("failed:0");

    vm.stop()?;

    if all_passed {
        Ok(())
    } else {
        bail!("Some tests failed")
    }
}
