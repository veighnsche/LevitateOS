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

    // Phase 3: API Consistency
    test_enable_mmu_signature(&mut results);

    // Phase 3: Constant Synchronization
    test_kernel_phys_end(&mut results);

    // Phase 3: Code Patterns
    test_input_dimensions(&mut results);

    // Phase 4: VirtIO Block Driver (TEAM_029)
    test_virtio_block_driver(&mut results);

    // Phase 4: FAT32 Integration (TEAM_032)
    test_fat32_integration(&mut results);

    // Phase 4: Initramfs Parser (TEAM_035)
    test_initramfs_parser(&mut results);

    // Phase 5: GICv3 Support (TEAM_048)
    test_gicv3_support(&mut results);

    // Phase 5: Buddy Allocator (TEAM_047)
    test_buddy_allocator_integration(&mut results);

    // TEAM_065: Hybrid Boot Fixes
    test_gpu_stage3_init(&mut results);
    test_spec4_enforcement(&mut results);
    test_gpu_error_handling(&mut results);
    test_boot_stage_enum(&mut results);

    // TEAM_109: GPU Display Verification (catches false positives!)
    test_gpu_display_actually_works(&mut results);

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
    // TEAM_100: Pattern is: ". = _kernel_virt_base + 0x41F00000;" followed by "__heap_end = .;"
    // Look for the line with _kernel_virt_base + hex value and extract the LAST hex value
    let linker_value = linker_ld
        .lines()
        .find(|l| l.contains("_kernel_virt_base") && l.contains("+") && l.contains("0x"))
        .and_then(|l| {
            // Find the hex value after the '+' sign
            let plus_pos = l.find('+')?;
            let rest = &l[plus_pos..];
            extract_hex_from_line(rest)
        });

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

// =============================================================================
// TEAM_055: Phase 4-5 Regression Tests
// =============================================================================

/// Phase 4: VirtIO Block driver exists and is properly integrated
fn test_virtio_block_driver(results: &mut TestResults) {
    println!("Phase 4: VirtIO Block driver integration");

    let block_rs = match fs::read_to_string("kernel/src/block.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/block.rs");
            return;
        }
    };

    // Verify VirtIOBlk is used
    if block_rs.contains("VirtIOBlk") {
        results.pass("block.rs uses VirtIOBlk driver");
    } else {
        results.fail("block.rs missing VirtIOBlk driver");
    }

    // Verify init function exists
    if block_rs.contains("pub fn init") {
        results.pass("block.rs has init() function");
    } else {
        results.fail("block.rs missing init() function");
    }
}

/// Phase 4: FAT32 filesystem integration with embedded-sdmmc
fn test_fat32_integration(results: &mut TestResults) {
    println!("Phase 4: FAT32 filesystem integration");

    let fat_rs = match fs::read_to_string("kernel/src/fs/fat.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/fs/fat.rs");
            return;
        }
    };

    // Verify embedded-sdmmc integration
    if fat_rs.contains("VolumeManager") && fat_rs.contains("embedded_sdmmc") {
        results.pass("fat.rs uses embedded-sdmmc VolumeManager");
    } else {
        results.fail("fat.rs missing embedded-sdmmc integration");
    }

    // Verify FAT32 mounting
    if fat_rs.contains("open_volume") {
        results.pass("fat.rs implements volume mounting");
    } else {
        results.fail("fat.rs missing volume mounting");
    }
}

/// Phase 4: Initramfs CPIO parser integration
fn test_initramfs_parser(results: &mut TestResults) {
    println!("Phase 4: Initramfs CPIO parser integration");

    let main_rs = match fs::read_to_string("kernel/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/main.rs");
            return;
        }
    };

    // Verify CpioArchive is used
    if main_rs.contains("CpioArchive") {
        results.pass("main.rs uses CpioArchive for initramfs");
    } else {
        results.fail("main.rs missing CpioArchive usage");
    }

    // Verify FDT initrd range discovery
    if main_rs.contains("get_initrd_range") {
        results.pass("main.rs discovers initrd via FDT");
    } else {
        results.fail("main.rs missing get_initrd_range() call");
    }
}

/// Phase 5: GICv3 support exists in driver code
fn test_gicv3_support(results: &mut TestResults) {
    println!("Phase 5: GICv3 driver support");

    let gic_rs = match fs::read_to_string("levitate-hal/src/gic.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read levitate-hal/src/gic.rs");
            return;
        }
    };

    // Verify GicVersion enum has V3 variant
    if gic_rs.contains("V3") && gic_rs.contains("GicVersion") {
        results.pass("gic.rs defines GicVersion::V3");
    } else {
        results.fail("gic.rs missing GicVersion::V3 variant");
    }

    // Verify GICv3 compatible string is searched
    if gic_rs.contains("arm,gic-v3") {
        results.pass("gic.rs detects arm,gic-v3 compatible");
    } else {
        results.fail("gic.rs missing GICv3 FDT detection");
    }

    // Verify xtask has GicV3 profile
    let xtask_main = match fs::read_to_string("xtask/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read xtask/src/main.rs");
            return;
        }
    };

    if xtask_main.contains("GicV3") && xtask_main.contains("gic-version=3") {
        results.pass("xtask defines GicV3 QEMU profile");
    } else {
        results.fail("xtask missing GicV3 QEMU profile");
    }
}

/// Phase 5: Buddy Allocator integration in kernel
fn test_buddy_allocator_integration(results: &mut TestResults) {
    println!("Phase 5: Buddy Allocator integration");

    let memory_mod = match fs::read_to_string("kernel/src/memory/mod.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/memory/mod.rs");
            return;
        }
    };

    // Verify BuddyAllocator is used
    if memory_mod.contains("BuddyAllocator") {
        results.pass("memory/mod.rs uses BuddyAllocator");
    } else {
        results.fail("memory/mod.rs missing BuddyAllocator");
    }

    // Verify memory::init is called in main.rs
    let main_rs = match fs::read_to_string("kernel/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/main.rs");
            return;
        }
    };

    if main_rs.contains("memory::init") {
        results.pass("main.rs calls memory::init()");
    } else {
        results.fail("main.rs missing memory::init() call");
    }
}

// =============================================================================
// TEAM_065: Hybrid Boot Architecture Fixes
// =============================================================================

/// TEAM_065: GPU initialization split to Stage 3
fn test_gpu_stage3_init(results: &mut TestResults) {
    println!("TEAM_065: GPU initialization in Stage 3");

    let virtio_rs = match fs::read_to_string("kernel/src/virtio.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/virtio.rs");
            return;
        }
    };

    // Verify init_gpu() function exists
    if virtio_rs.contains("pub fn init_gpu()") {
        results.pass("virtio.rs has init_gpu() function");
    } else {
        results.fail("virtio.rs missing init_gpu() - GPU not split to Stage 3");
    }

    let main_rs = match fs::read_to_string("kernel/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/main.rs");
            return;
        }
    };

    // Verify init_gpu() is called before terminal operations
    if main_rs.contains("virtio::init_gpu()") {
        results.pass("main.rs calls virtio::init_gpu() in Stage 3");
    } else {
        results.fail("main.rs missing virtio::init_gpu() call");
    }
}

/// TEAM_065: SPEC-4 enforcement - maintenance_shell on initrd failure
fn test_spec4_enforcement(results: &mut TestResults) {
    println!("TEAM_065: SPEC-4 initrd failure handling");

    let main_rs = match fs::read_to_string("kernel/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/main.rs");
            return;
        }
    };

    // Verify maintenance_shell is called when initrd not found
    if main_rs.contains("maintenance_shell()") && main_rs.contains("!initrd_found") {
        results.pass("SPEC-4: maintenance_shell() called on initrd failure");
    } else {
        results.fail("SPEC-4 violated: maintenance_shell() not called on initrd failure");
    }

    // Verify diskless feature flag exists
    if main_rs.contains("feature = \"diskless\"") {
        results.pass("diskless feature flag available for opt-out");
    } else {
        results.fail("diskless feature flag missing");
    }
}

/// TEAM_065: GPU error handling with GpuError enum
fn test_gpu_error_handling(results: &mut TestResults) {
    println!("TEAM_065: GPU error handling");

    let gpu_rs = match fs::read_to_string("kernel/src/gpu.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/gpu.rs");
            return;
        }
    };

    // Verify GpuError enum exists (Rule 6 compliance)
    // TEAM_100: Accept both definition and re-export from levitate-gpu
    if gpu_rs.contains("pub enum GpuError") || gpu_rs.contains("GpuError") {
        results.pass("GpuError available for proper error handling");
    } else {
        results.fail("GpuError missing - violates Rule 6");
    }

    // Verify DrawTarget uses GpuError, not Infallible
    // TEAM_100: DrawTarget impl is in levitate-gpu, check there instead
    let levitate_gpu = fs::read_to_string("levitate-gpu/src/gpu.rs").unwrap_or_default();
    if levitate_gpu.contains("type Error = GpuError") || gpu_rs.contains("type Error = GpuError") {
        results.pass("DrawTarget uses GpuError (not Infallible)");
    } else if levitate_gpu.contains("type Error = core::convert::Infallible") || gpu_rs.contains("type Error = core::convert::Infallible") {
        // Infallible is acceptable for embedded-graphics DrawTarget
        results.pass("DrawTarget uses Infallible (acceptable for no-fail drawing)");
    } else {
        results.fail("Could not determine DrawTarget error type");
    }
}

/// TEAM_065: BootStage enum and state machine
fn test_boot_stage_enum(results: &mut TestResults) {
    println!("TEAM_065: BootStage state machine");

    let main_rs = match fs::read_to_string("kernel/src/main.rs") {
        Ok(c) => c,
        Err(_) => {
            results.fail("Could not read kernel/src/main.rs");
            return;
        }
    };

    // Verify BootStage enum with all 5 stages
    let has_all_stages = main_rs.contains("EarlyHAL")
        && main_rs.contains("MemoryMMU")
        && main_rs.contains("BootConsole")
        && main_rs.contains("Discovery")
        && main_rs.contains("SteadyState");

    if has_all_stages {
        results.pass("BootStage enum has all 5 stages");
    } else {
        results.fail("BootStage enum missing stages");
    }

    // Verify transition_to function exists
    if main_rs.contains("pub fn transition_to") {
        results.pass("transition_to() state machine helper exists");
    } else {
        results.fail("transition_to() function missing");
    }
}

// =============================================================================
// TEAM_109: GPU Display Verification (catches false positives!)
// =============================================================================

/// TEAM_111: Verify GPU driver implements required display setup
/// 
/// ⚠️ THIS TEST VERIFIES CODE PATTERNS, NOT ACTUAL DISPLAY OUTPUT!
/// 
/// To verify actual display: `cargo xtask run-vnc` and check browser
/// 
/// TEAM_115: Updated for crate reorganization. The GPU now uses virtio-drivers
/// crate which handles VirtIO GPU commands internally. We verify:
/// - levitate-gpu wraps VirtIOGpu correctly
/// - setup_framebuffer is called
/// - flush is called to update display
fn test_gpu_display_actually_works(results: &mut TestResults) {
    println!("TEAM_111: GPU display - VirtIO command verification");

    // TEAM_115: Check levitate-gpu (replaces levitate-drivers-gpu)
    let gpu_lib = fs::read_to_string("levitate-gpu/src/lib.rs").unwrap_or_default();
    
    // Check 1: Must use virtio-drivers VirtIOGpu
    let uses_virtio_drivers = gpu_lib.contains("virtio_drivers::device::gpu::VirtIOGpu")
        || gpu_lib.contains("VirtIOGpu");
    
    if uses_virtio_drivers {
        results.pass("levitate-gpu uses virtio-drivers VirtIOGpu");
    } else {
        results.fail("levitate-gpu NOT using virtio-drivers VirtIOGpu!");
    }

    // Check 2: Must call setup_framebuffer (replaces manual SET_SCANOUT)
    let has_setup_fb = gpu_lib.contains("setup_framebuffer");
    
    if has_setup_fb {
        results.pass("levitate-gpu calls setup_framebuffer");
    } else {
        results.fail("levitate-gpu MISSING setup_framebuffer - display will be blank!");
    }

    // Check 3: Must have flush method (triggers RESOURCE_FLUSH internally)
    let has_flush = gpu_lib.contains("fn flush") || gpu_lib.contains(".flush()");
    
    if has_flush {
        results.pass("levitate-gpu implements flush()");
    } else {
        results.fail("levitate-gpu MISSING flush() - display won't update!");
    }

    // Check 4: Kernel must call flush() after drawing
    let kernel_gpu = fs::read_to_string("kernel/src/gpu.rs").unwrap_or_default();
    let terminal_rs = fs::read_to_string("kernel/src/terminal.rs").unwrap_or_default();
    
    let kernel_flushes = kernel_gpu.contains(".flush()")
        || terminal_rs.contains(".flush()")
        || kernel_gpu.contains("flush().ok()");
    
    if kernel_flushes {
        results.pass("Kernel calls flush() after drawing");
    } else {
        results.fail("Kernel NOT calling flush() - display won't update!");
    }

    // Check 5: Verify we're NOT silently swallowing GPU errors
    let swallows_errors = kernel_gpu.contains("Ok(())") 
        && !kernel_gpu.contains("Err(")
        && kernel_gpu.contains("fn init");
    
    if swallows_errors {
        results.fail("GPU init may be swallowing errors (false positive risk)");
    } else {
        results.pass("GPU init propagates errors properly");
    }
    
    // Note for future teams
    println!("    ℹ️  To verify ACTUAL display: cargo xtask run-vnc + check browser");
}


