//! Regression tests for bugs found in TEAM_025-029 implementations
//!
//! TEAM_030: Static analysis tests that verify source code patterns.
//! These catch bugs that can't be caught by unit tests or runtime behavior tests.
//!
//! Test Categories:
//! - API Consistency: Function signatures match across cfg targets
//! - Constant Synchronization: Values match between files (mmu.rs <-> linker.ld)
//! - Code Patterns: Correct API usage (e.g., dimensions() not hardcoded values)

use anyhow::{bail, Result};
use std::fs;

struct TestResults {
    passed: u32,
    failed: u32,
}

impl TestResults {
    fn new() -> Self {
        Self { passed: 0, failed: 0 }
    }

    fn pass(&mut self, msg: &str) {
        println!("  ✅ {}", msg);
        self.passed += 1;
    }

    fn fail(&mut self, msg: &str) {
        println!("  ❌ {}", msg);
        self.failed += 1;
    }

    fn summary(&self) -> bool {
        println!("\n=== Results ===");
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);
        self.failed == 0
    }
}

pub fn run() -> Result<()> {
    println!("=== Static Analysis / Regression Tests ===\n");

    let mut results = TestResults::new();

    // API Consistency
    test_enable_mmu_signature(&mut results);

    // Constant Synchronization
    test_kernel_phys_end(&mut results);

    // Code Patterns
    test_input_dimensions(&mut results);

    println!();
    if results.summary() {
        println!("\n✅ All regression tests passed\n");
        Ok(())
    } else {
        println!("\n❌ REGRESSION DETECTED\n");
        bail!("Regression tests failed");
    }
}

/// API Consistency: enable_mmu stub signature matches real function
fn test_enable_mmu_signature(results: &mut TestResults) {
    println!("API: enable_mmu stub signature matches real function");

    let content = match fs::read_to_string("levitate-hal/src/mmu.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read levitate-hal/src/mmu.rs");
            return;
        }
    };

    // Check for the correct 2-argument signature in both versions
    let has_aarch64 = content.contains("pub unsafe fn enable_mmu(ttbr0_phys: usize, ttbr1_phys: usize)");
    let has_stub = content.contains("pub unsafe fn enable_mmu(_ttbr0_phys: usize, _ttbr1_phys: usize)");

    if has_aarch64 && has_stub {
        results.pass("enable_mmu stub has 2 arguments (matches real function)");
    } else {
        results.fail("enable_mmu stub signature mismatch");
    }
}

/// Constant Sync: KERNEL_PHYS_END matches linker.ld __heap_end
fn test_kernel_phys_end(results: &mut TestResults) {
    println!("Sync: KERNEL_PHYS_END constant matches linker.ld __heap_end");

    let mmu_rs = match fs::read_to_string("levitate-hal/src/mmu.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read levitate-hal/src/mmu.rs");
            return;
        }
    };

    let linker_ld = match fs::read_to_string("linker.ld") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read linker.ld");
            return;
        }
    };

    // Extract KERNEL_PHYS_END from mmu.rs
    let mmu_value = extract_hex_constant(&mmu_rs, "KERNEL_PHYS_END");

    // Extract __heap_end offset from linker.ld
    // Line looks like: __heap_end = _kernel_virt_base + 0x41F00000;
    let linker_value = linker_ld
        .lines()
        .find(|l| l.contains("__heap_end") && l.contains("_kernel_virt_base"))
        .and_then(|l| extract_hex_from_line(l));

    match (mmu_value, linker_value) {
        (Some(mmu), Some(linker)) if mmu == linker => {
            results.pass(&format!(
                "KERNEL_PHYS_END ({:#x}) matches linker.ld ({:#x})",
                mmu, linker
            ));
        }
        (Some(mmu), Some(linker)) => {
            results.fail(&format!(
                "KERNEL_PHYS_END ({:#x}) does NOT match linker.ld ({:#x})",
                mmu, linker
            ));
        }
        _ => {
            results.fail("Could not extract values from source files");
        }
    }
}

/// Code Pattern: Input cursor scaling uses GPU dimensions, not hardcoded values
fn test_input_dimensions(results: &mut TestResults) {
    println!("Pattern: Input cursor scaling uses GPU dimensions");

    let content = match fs::read_to_string("kernel/src/input.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/input.rs");
            return;
        }
    };

    if content.contains("dimensions()") {
        // Also check there's no hardcoded 1024 in the calculation (excluding fallback)
        let has_hardcoded = content
            .lines()
            .filter(|l| !l.contains("Fallback") && !l.contains("//"))
            .any(|l| {
                (l.contains("event.value") || l.contains("event_value"))
                    && l.contains("1024")
                    && !l.contains("dimensions")
            });

        if has_hardcoded {
            results.fail("input.rs still has hardcoded 1024 in cursor calculation");
        } else {
            results.pass("input.rs uses GPU.dimensions() for cursor scaling");
        }
    } else {
        results.fail("input.rs doesn't call dimensions() - cursor scaling may be hardcoded");
    }
}

// Note: Build verification tests removed - redundant with:
// - Unit tests (run cargo test, which compiles for host)
// - Behavior tests (run cargo build for aarch64)

fn extract_hex_constant(content: &str, name: &str) -> Option<u64> {
    content
        .lines()
        .find(|l| l.contains(&format!("const {}", name)))
        .and_then(|l| extract_hex_from_line(l))
}

fn extract_hex_from_line(line: &str) -> Option<u64> {
    // Find 0x... pattern and parse it
    let start = line.find("0x")?;
    let rest = &line[start + 2..];
    let end = rest
        .find(|c: char| !c.is_ascii_hexdigit() && c != '_')
        .unwrap_or(rest.len());
    let hex_str = rest[..end].replace('_', "");
    u64::from_str_radix(&hex_str, 16).ok()
}
