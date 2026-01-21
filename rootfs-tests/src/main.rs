//! LevitateOS rootfs test runner.
//!
//! Tests the rootfs as a daily driver OS using systemd-nspawn.
//! Each test answers: "Can a user do X with this OS?"
//!
//! # STOP. READ. THEN ACT.
//!
//! Before modifying this crate:
//! 1. Read `tests/` to understand existing test patterns
//! 2. Read `container.rs` to understand how nspawn is used
//! 3. Check if similar test already exists before adding new ones

mod container;
mod tests;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;

use container::Container;
use tests::{all_tests, TestResult};

#[derive(Parser)]
#[command(name = "rootfs-tests")]
#[command(about = "Daily driver tests for LevitateOS rootfs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    Run {
        /// Path to rootfs tarball
        #[arg(long, default_value = "../leviso/output/levitateos-base.tar.xz")]
        tarball: PathBuf,

        /// Run only tests in a specific category
        #[arg(long)]
        category: Option<String>,

        /// Show detailed output for each test
        #[arg(long, short)]
        verbose: bool,
    },

    /// List all available tests
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { tarball, category, verbose } => {
            run_tests(&tarball, category, verbose)
        }
        Commands::List => list_tests(),
    }
}

fn run_tests(tarball: &PathBuf, category: Option<String>, verbose: bool) -> Result<()> {
    println!("LevitateOS Rootfs Tests");
    println!("=======================\n");
    println!("Testing: Can a user use this as a daily driver OS?\n");

    // Extract tarball
    let target_dir = PathBuf::from("/var/lib/machines/levitate-rootfs-test");
    println!("Extracting {} ...", tarball.display());
    let container = Container::from_tarball(tarball, &target_dir)?;
    println!("Ready.\n");

    // Get tests
    let all = all_tests();
    let tests: Vec<_> = if let Some(ref cat) = category {
        all.into_iter().filter(|t| t.category() == cat).collect()
    } else {
        all
    };

    if tests.is_empty() {
        println!("No tests found for category: {:?}", category);
        return Ok(());
    }

    // Run tests
    let start = Instant::now();
    let mut results: Vec<TestResult> = Vec::new();
    let mut current_category = String::new();

    for test in &tests {
        // Print category header
        if test.category() != current_category {
            if !current_category.is_empty() {
                println!();
            }
            current_category = test.category().to_string();
            println!("━━━ {} ━━━", current_category.to_uppercase());
        }

        // Run test
        let result = test.run(&container);
        print_result(&result, verbose);
        results.push(result);
    }

    // Summary
    println!("\n════════════════════════════════════════════════════════════\n");

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let duration = start.elapsed();

    if failed == 0 {
        println!("✓ All {} tests passed ({:.1}s)", passed, duration.as_secs_f64());
        println!("\nThis rootfs is ready for daily driver use.");
    } else {
        println!("✗ {}/{} tests failed ({:.1}s)\n", failed, results.len(), duration.as_secs_f64());

        println!("Failed tests:");
        for result in &results {
            if !result.passed {
                println!("\n  ✗ {}", result.name);
                println!("    ensures: {}", result.ensures);
                println!("    error: {}", result.output);
            }
        }
        std::process::exit(1);
    }

    Ok(())
}

fn print_result(result: &TestResult, verbose: bool) {
    let status = if result.passed { "✓" } else { "✗" };
    let time = format!("{:.1}s", result.duration.as_secs_f64());

    if result.passed {
        println!("  {} {} ({})", status, result.name, time);
        if verbose {
            println!("      ensures: {}", result.ensures);
            println!("      output: {}", truncate(&result.output, 60));
        }
    } else {
        println!("  {} {} ({}) - FAILED", status, result.name, time);
        println!("      ensures: {}", result.ensures);
        println!("      error: {}", result.output);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn list_tests() -> Result<()> {
    println!("LevitateOS Rootfs Tests\n");
    println!("Each test verifies the OS works for daily use.\n");

    let tests = all_tests();
    let mut current_category = String::new();

    for test in &tests {
        if test.category() != current_category {
            if !current_category.is_empty() {
                println!();
            }
            current_category = test.category().to_string();
            println!("{}:", current_category.to_uppercase());
        }
        println!("  • {}", test.name());
        println!("    ensures: {}", test.ensures());
    }

    println!("\nTotal: {} tests", tests.len());
    Ok(())
}
