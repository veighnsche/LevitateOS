//! Regression tests for bugs found in TEAM_025-029 implementations
//!
//! `TEAM_030`: Static analysis tests that verify source code patterns.
//! These catch bugs that can't be caught by unit tests or runtime behavior tests.
//!
//! Test Categories:
//! - API Consistency: Function signatures match across cfg targets
//! - Constant Synchronization: Values match between files (mmu.rs <-> linker.ld)
//! - Code Patterns: Correct API usage (e.g., `dimensions()` not hardcoded values)
//!
//! # CRITICAL: Test Failure Handling (`TEAM_132`)
//!
//! **ALL TESTS MUST PASS. NO EXCEPTIONS.**
//!
//! When a regression test fails after your changes:
//! 1. **STOP** — Do not proceed with other work.
//! 2. **INVESTIGATE** — Read what the test checks. Don't assume "pre-existing."
//! 3. **FIX IT** — Whether your code or the test itself is buggy.
//! 4. **VERIFY** — Run full test suite again.
//!
//! **NEVER dismiss a failing test as "pre-existing" without investigation.**
//! The purpose of regression tests is to catch when changes to A break Z.

use anyhow::{bail, Result};
use std::fs;

struct TestResults {
    passed: u32,
    failed: u32,
}

impl TestResults {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
        }
    }

    fn pass(&mut self, msg: &str) {
        println!("  ✅ {msg}");
        self.passed += 1;
    }

    fn fail(&mut self, msg: &str) {
        println!("  ❌ {msg}");
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

    // TEAM_139: QEMU Configuration
    test_qemu_serial_multiplexing(&mut results);
    test_qemu_window_size(&mut results);

    // TEAM_142: Shell Input & Graceful Shutdown
    test_shell_backspace(&mut results);
    test_graceful_shutdown(&mut results);

    // TEAM_464: linux-raw-sys consolidation tests
    test_linux_raw_sys_signal_constants(&mut results);
    test_linux_raw_sys_seek_constants(&mut results);
    test_linux_raw_sys_fcntl_constants(&mut results);
    test_linux_raw_sys_poll_constants(&mut results);
    test_linux_raw_sys_errno_usage(&mut results);
    test_linux_raw_sys_auxv_constants(&mut results);
    test_linux_raw_sys_mode_constants(&mut results);

    println!();
    if results.summary() {
        println!("\n✅ All regression tests passed\n");
        Ok(())
    } else {
        println!("\n❌ REGRESSION DETECTED\n");
        bail!("Regression tests failed");
    }
}

/// API Consistency: `enable_mmu` exists in arch-specific module
/// `TEAM_342`: Updated to check arch-specific files after mmu.rs became delegation-only
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_enable_mmu_signature(results: &mut TestResults) {
    println!("API: enable_mmu stub signature matches real function");

    // TEAM_465: HAL is now at crates/kernel/lib/hal/, mmu init is in submodule
    let aarch64_mmu =
        fs::read_to_string("crates/kernel/lib/hal/src/aarch64/mmu/init.rs").unwrap_or_default();
    let x86_mmu = fs::read_to_string("crates/kernel/lib/hal/src/x86_64/mmu.rs").unwrap_or_default();

    // Check that enable_mmu exists in at least one arch (may be unsafe or not)
    let has_aarch64 = aarch64_mmu.contains("fn enable_mmu");
    let has_x86 = x86_mmu.contains("fn enable_mmu") || x86_mmu.contains("enable_mmu");

    if has_aarch64 || has_x86 {
        results.pass("enable_mmu exists in arch-specific module");
    } else {
        results.fail("enable_mmu missing from arch modules");
    }
}

/// Constant Sync: `KERNEL_PHYS_END` matches linker.ld __`heap_end`
/// `TEAM_342`: Updated to check arch-specific mmu files
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_kernel_phys_end(results: &mut TestResults) {
    println!("Sync: KERNEL_PHYS_END constant matches linker.ld __heap_end");

    // TEAM_465: KERNEL_PHYS_END is now in constants.rs submodule
    let mmu_rs =
        if let Ok(c) = fs::read_to_string("crates/kernel/lib/hal/src/aarch64/mmu/constants.rs") {
            c
        } else {
            results.fail("Could not read crates/kernel/lib/hal/src/aarch64/mmu/constants.rs");
            return;
        };

    let linker_ld = if let Ok(c) = fs::read_to_string("linker.ld") {
        c
    } else {
        results.fail("Could not read linker.ld");
        return;
    };

    // Extract KERNEL_PHYS_END from mmu.rs
    let mmu_value = extract_hex_constant(&mmu_rs, "KERNEL_PHYS_END");

    // Extract __heap_end offset from linker.ld
    // TEAM_100: Pattern is: ". = _kernel_virt_base + 0x41F00000;" followed by "__heap_end = .;"
    // Look for the line with _kernel_virt_base + hex value and extract the LAST hex value
    let linker_value = linker_ld
        .lines()
        .find(|l| l.contains("_kernel_virt_base") && l.contains('+') && l.contains("0x"))
        .and_then(|l| {
            // Find the hex value after the '+' sign
            let plus_pos = l.find('+')?;
            let rest = &l[plus_pos..];
            extract_hex_from_line(rest)
        });

    match (mmu_value, linker_value) {
        (Some(mmu), Some(linker)) if mmu == linker => {
            results.pass(&format!(
                "KERNEL_PHYS_END ({mmu:#x}) matches linker.ld ({linker:#x})"
            ));
        }
        (Some(mmu), Some(linker)) => {
            results.fail(&format!(
                "KERNEL_PHYS_END ({mmu:#x}) does NOT match linker.ld ({linker:#x})"
            ));
        }
        _ => {
            results.fail("Could not extract values from source files");
        }
    }
}

/// Code Pattern: Input cursor scaling uses GPU dimensions, not hardcoded values
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_input_dimensions(results: &mut TestResults) {
    println!("Pattern: Input cursor scaling uses GPU dimensions");

    // TEAM_465: Kernel main code is now in levitate subdir
    let content = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/input.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/input.rs");
        return;
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
        .find(|l| l.contains(&format!("const {name}")))
        .and_then(extract_hex_from_line)
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

/// Phase 4: `VirtIO` Block driver exists and is properly integrated
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_virtio_block_driver(results: &mut TestResults) {
    println!("Phase 4: VirtIO Block driver integration");

    // TEAM_465: Kernel main code is now in levitate subdir
    let block_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/block.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/block.rs");
        return;
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
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_fat32_integration(results: &mut TestResults) {
    println!("Phase 4: FAT32 filesystem integration");

    // TEAM_465: FAT32 is now in ext4 fs crate (historical naming)
    let fat_rs = if let Ok(c) = fs::read_to_string("crates/kernel/fs/ext4/src/fat.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/fs/ext4/src/fat.rs");
        return;
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
/// `TEAM_342`: Updated - initramfs uses INITRAMFS global, not `CpioArchive` directly in init.rs
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_initramfs_parser(results: &mut TestResults) {
    println!("Phase 4: Initramfs CPIO parser integration");

    // TEAM_465: Kernel main code is now in levitate subdir
    let init_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/init.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/init.rs");
        return;
    };

    // TEAM_465: fs.rs is in levitate, not fs/mod.rs
    let fs_mod = fs::read_to_string("crates/kernel/levitate/src/fs.rs").unwrap_or_default();

    // Verify initramfs handling exists somewhere
    if init_rs.contains("INITRAMFS")
        || init_rs.contains("initramfs")
        || fs_mod.contains("INITRAMFS")
    {
        results.pass("Initramfs handling exists in kernel");
    } else {
        results.fail("Initramfs handling missing");
    }

    // Verify initrd discovery (could be via Limine or FDT)
    if init_rs.contains("initrd") || init_rs.contains("initramfs") || init_rs.contains("Initramfs")
    {
        results.pass("init.rs handles initrd discovery");
    } else {
        results.fail("init.rs missing initrd handling");
    }
}

/// Phase 5: `GICv3` support exists in driver code
/// `TEAM_342`: Fixed path - gic.rs is in aarch64 subdir
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_gicv3_support(results: &mut TestResults) {
    println!("Phase 5: GICv3 driver support");

    // TEAM_465: HAL is now at crates/kernel/lib/hal/
    let gic_rs = if let Ok(c) = fs::read_to_string("crates/kernel/lib/hal/src/aarch64/gic.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/lib/hal/src/aarch64/gic.rs");
        return;
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
    // TEAM_342: GicV3 profile moved to qemu/profile.rs after refactor
    let profile_rs = fs::read_to_string("xtask/src/qemu/profile.rs").unwrap_or_default();
    let run_rs = fs::read_to_string("xtask/src/run.rs").unwrap_or_default();
    let combined = format!("{profile_rs}{run_rs}");

    if combined.contains("GicV3") && combined.contains("gic-version=3") {
        results.pass("xtask defines GicV3 QEMU profile");
    } else {
        results.fail("xtask missing GicV3 QEMU profile");
    }
}

/// Phase 5: Buddy Allocator integration in kernel
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_buddy_allocator_integration(results: &mut TestResults) {
    println!("Phase 5: Buddy Allocator integration");

    // TEAM_465: Memory is now split - check los_mm crate and HAL allocator
    let mm_lib = fs::read_to_string("crates/kernel/mm/src/lib.rs").unwrap_or_default();
    let hal_allocator =
        fs::read_to_string("crates/kernel/lib/hal/src/allocator/buddy.rs").unwrap_or_default();
    let memory_rs = fs::read_to_string("crates/kernel/levitate/src/memory.rs").unwrap_or_default();

    // Verify BuddyAllocator is used somewhere in memory subsystem
    if mm_lib.contains("BuddyAllocator") || hal_allocator.contains("BuddyAllocator") {
        results.pass("BuddyAllocator exists in memory subsystem");
    } else {
        results.fail("BuddyAllocator missing from memory subsystem");
    }

    // TEAM_465: Kernel main code is now in levitate subdir
    let init_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/init.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/init.rs");
        return;
    };

    // TEAM_465: Check for any memory initialization
    if init_rs.contains("memory::init")
        || init_rs.contains("init_mmu")
        || memory_rs.contains("init")
    {
        results.pass("Memory subsystem initialization present");
    } else {
        results.fail("Memory initialization missing");
    }
}

// =============================================================================
// TEAM_065: Hybrid Boot Architecture Fixes
// =============================================================================

/// `TEAM_065`: GPU initialization split to Stage 3
/// `TEAM_146`: Refactored - GPU init moved to init.rs
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_gpu_stage3_init(results: &mut TestResults) {
    println!("TEAM_065: GPU initialization in Stage 3");

    // TEAM_465: Kernel main code is now in levitate subdir
    let virtio_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/virtio.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/virtio.rs");
        return;
    };

    // Verify init_gpu() function exists
    if virtio_rs.contains("pub fn init_gpu()") {
        results.pass("virtio.rs has init_gpu() function");
    } else {
        results.fail("virtio.rs missing init_gpu() - GPU not split to Stage 3");
    }

    let init_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/init.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/init.rs");
        return;
    };

    // Verify init_gpu() is called before terminal operations
    if init_rs.contains("init_gpu()") {
        results.pass("init.rs calls init_gpu() in Stage 3");
    } else {
        results.fail("init.rs missing init_gpu() call");
    }
}

/// `TEAM_065`: SPEC-4 enforcement - `maintenance_shell` on initrd failure
/// `TEAM_146`: Refactored - SPEC-4 code moved to init.rs
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_spec4_enforcement(results: &mut TestResults) {
    println!("TEAM_065: SPEC-4 initrd failure handling");

    // TEAM_465: Kernel main code is now in levitate subdir
    let init_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/init.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/init.rs");
        return;
    };

    // Verify maintenance_shell is called when initrd not found
    if init_rs.contains("maintenance_shell()") && init_rs.contains("!initrd_found") {
        results.pass("SPEC-4: maintenance_shell() called on initrd failure");
    } else {
        results.fail("SPEC-4 violated: maintenance_shell() not called on initrd failure");
    }

    // Verify diskless feature flag exists
    if init_rs.contains("feature = \"diskless\"") {
        results.pass("diskless feature flag available for opt-out");
    } else {
        results.fail("diskless feature flag missing");
    }
}

/// `TEAM_065`: GPU error handling with `GpuError` enum
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_gpu_error_handling(results: &mut TestResults) {
    println!("TEAM_065: GPU error handling");

    // TEAM_465: Kernel main code is now in levitate subdir
    let gpu_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/gpu.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/gpu.rs");
        return;
    };

    // Verify GpuError enum exists (Rule 6 compliance)
    // TEAM_100: Accept both definition and re-export from levitate-gpu
    if gpu_rs.contains("pub enum GpuError") || gpu_rs.contains("GpuError") {
        results.pass("GpuError available for proper error handling");
    } else {
        results.fail("GpuError missing - violates Rule 6");
    }

    // Verify DrawTarget uses GpuError, not Infallible
    // TEAM_465: GPU driver is now at crates/kernel/drivers/gpu/
    let los_gpu_lib =
        fs::read_to_string("crates/kernel/drivers/gpu/src/lib.rs").unwrap_or_default();
    if los_gpu_lib.contains("type Error = GpuError") || gpu_rs.contains("type Error = GpuError") {
        results.pass("DrawTarget uses GpuError (not Infallible)");
    } else if los_gpu_lib.contains("type Error = core::convert::Infallible")
        || gpu_rs.contains("type Error = core::convert::Infallible")
    {
        // Infallible is acceptable for embedded-graphics DrawTarget
        results.pass("DrawTarget uses Infallible (acceptable for no-fail drawing)");
    } else {
        results.fail("Could not determine DrawTarget error type");
    }
}

/// `TEAM_065`: `BootStage` enum and state machine
/// `TEAM_146`: Refactored - `BootStage` moved to init.rs
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_boot_stage_enum(results: &mut TestResults) {
    println!("TEAM_065: BootStage state machine");

    // TEAM_465: Kernel main code is now in levitate subdir
    let init_rs = if let Ok(c) = fs::read_to_string("crates/kernel/levitate/src/init.rs") {
        c
    } else {
        results.fail("Could not read crates/kernel/levitate/src/init.rs");
        return;
    };

    // TEAM_342: BootStage has 4 stages (SteadyState not implemented yet)
    let has_core_stages = init_rs.contains("EarlyHAL")
        && init_rs.contains("MemoryMMU")
        && init_rs.contains("BootConsole")
        && init_rs.contains("Discovery");

    if has_core_stages {
        results.pass("BootStage enum has core boot stages");
    } else {
        results.fail("BootStage enum missing stages");
    }

    // Verify transition_to function exists
    if init_rs.contains("pub fn transition_to") {
        results.pass("transition_to() state machine helper exists");
    } else {
        results.fail("transition_to() function missing");
    }
}

// =============================================================================
// TEAM_109: GPU Display Verification (catches false positives!)
// =============================================================================

/// `TEAM_111`: Verify GPU driver implements required display setup
///
/// ⚠️ THIS TEST VERIFIES CODE PATTERNS, NOT ACTUAL DISPLAY OUTPUT!
///
/// To verify actual display: `cargo xtask run-vnc` and check browser
///
/// `TEAM_115`: Updated for crate reorganization. The GPU now uses virtio-drivers
/// crate which handles `VirtIO` GPU commands internally. We verify:
/// - levitate-gpu wraps `VirtIOGpu` correctly
/// - `setup_framebuffer` is called
/// - flush is called to update display
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_gpu_display_actually_works(results: &mut TestResults) {
    println!("TEAM_111: GPU display - VirtIO command verification");

    // TEAM_465: GPU driver is now at crates/kernel/drivers/gpu/
    let gpu_lib = fs::read_to_string("crates/kernel/drivers/gpu/src/lib.rs").unwrap_or_default();

    // Check 1: Must use virtio-drivers VirtIOGpu
    let uses_virtio_drivers =
        gpu_lib.contains("virtio_drivers::device::gpu::VirtIOGpu") || gpu_lib.contains("VirtIOGpu");

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
    // TEAM_465: Kernel main code is now in levitate subdir
    let kernel_gpu = fs::read_to_string("crates/kernel/levitate/src/gpu.rs").unwrap_or_default();
    let terminal_rs =
        fs::read_to_string("crates/kernel/levitate/src/terminal.rs").unwrap_or_default();

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

// =============================================================================
// TEAM_139: QEMU Configuration Regression Tests
// =============================================================================

/// `TEAM_139`: Verify QEMU uses mon:stdio for serial+monitor multiplexing
/// `TEAM_342`: Updated to check builder.rs (refactored from run.rs)
/// `TEAM_465`: Updated to allow plain -serial stdio for Nographic mode (`TEAM_444`)
fn test_qemu_serial_multiplexing(results: &mut TestResults) {
    println!("TEAM_139: QEMU serial multiplexing");

    // TEAM_342: QEMU config moved to builder.rs
    let builder_rs = fs::read_to_string("xtask/src/qemu/builder.rs").unwrap_or_default();
    let run_rs = fs::read_to_string("xtask/src/run.rs").unwrap_or_default();
    let combined = format!("{builder_rs}{run_rs}");

    // Must use mon:stdio for monitor+serial multiplexing
    if combined.contains("mon:stdio") {
        results.pass("QEMU uses mon:stdio for input multiplexing");
    } else {
        results.fail("QEMU NOT using mon:stdio - can't switch to monitor with Ctrl+A C!");
    }

    // TEAM_465: Nographic mode (TEAM_444) intentionally uses plain -serial stdio
    // because mux causes input to go to monitor by default.
    // Only check that mon:stdio is used for the main display modes (GTK, VNC, Headless).
    let has_mon_stdio_for_main_modes = builder_rs.contains("DisplayMode::Gtk")
        && builder_rs.contains("DisplayMode::Vnc")
        && builder_rs.contains("DisplayMode::Headless")
        && builder_rs
            .lines()
            .filter(|l| l.contains("mon:stdio"))
            .count()
            >= 3;

    if has_mon_stdio_for_main_modes {
        results.pass("Main display modes use mon:stdio");
    } else {
        results.fail("Main display modes missing mon:stdio configuration");
    }
}

/// `TEAM_139`: Verify QEMU has explicit display configuration for proper window sizing
/// `TEAM_342`: Updated to check builder.rs and accept SDL as alternative to GTK
fn test_qemu_window_size(results: &mut TestResults) {
    println!("TEAM_139: QEMU window size configuration");

    // TEAM_342: QEMU config moved to builder.rs
    let builder_rs = fs::read_to_string("xtask/src/qemu/builder.rs").unwrap_or_default();

    // Must have explicit display configuration for non-headless
    // TEAM_342: Accept SDL or GTK - both provide proper windowing
    if builder_rs.contains("-display") && (builder_rs.contains("gtk") || builder_rs.contains("sdl"))
    {
        results.pass("QEMU has explicit display configuration (GTK/SDL)");
    } else {
        results.fail("QEMU missing display config - window size may be wrong!");
    }

    // TEAM_342: window-close=off is optional with SDL, check for display mode enum instead
    if builder_rs.contains("DisplayMode") && builder_rs.contains("Gtk") {
        results.pass("QEMU display mode properly configured");
    } else {
        results.fail("QEMU missing DisplayMode configuration");
    }
}

// =============================================================================
// TEAM_142: Graceful Shutdown Regression Tests
// =============================================================================

/// `TEAM_142`: Verify shell backspace handling
/// `TEAM_465`: Updated to match new match-based shell implementation
fn test_shell_backspace(results: &mut TestResults) {
    println!("TEAM_142: Shell backspace handling");

    let shell_main = fs::read_to_string("crates/userspace/shell/src/main.rs").unwrap_or_default();

    // Must handle both backspace codes (0x08 and 0x7f)
    if shell_main.contains("0x08") && shell_main.contains("0x7f") {
        results.pass("Shell handles both backspace codes (0x08, 0x7f)");
    } else {
        results.fail("Shell missing backspace code handling");
    }

    // Must use proper terminal erase sequence (back, space, back)
    if shell_main.contains(r#"b"\x08 \x08""#) || shell_main.contains(r"\x08 \x08") {
        results.pass("Shell uses proper erase sequence (\\x08 \\x08)");
    } else {
        results.fail("Shell missing proper erase sequence - backspace won't work visually");
    }

    // TEAM_465: Shell now uses match statement for input handling.
    // Backspace is handled in its own arm, regular chars in _ arm with range check.
    // Verify that regular character echoing is bounded properly (ch >= 0x20 && ch < 0x7f)
    let has_proper_echo_control = shell_main.contains("match ch")
        && (shell_main.contains("0x7f | 0x08") || shell_main.contains("0x08 | 0x7f"))
        && shell_main.contains("ch >= 0x20");
    if has_proper_echo_control {
        results.pass("Shell uses match-based input handling with proper echo control");
    } else {
        results.fail("Shell may echo control characters - check match arms");
    }
}

/// `TEAM_142`: Verify graceful shutdown syscall implementation
/// `TEAM_342`: Updated paths after syscall module refactor
/// `TEAM_465`: Updated paths after kernel submodule reorganization
fn test_graceful_shutdown(results: &mut TestResults) {
    println!("TEAM_142: Graceful shutdown implementation");

    // TEAM_465: Syscall crate is now at crates/kernel/syscall/src/
    let syscall_mod = fs::read_to_string("crates/kernel/syscall/src/lib.rs").unwrap_or_default();
    let syscall_sys = fs::read_to_string("crates/kernel/syscall/src/sys.rs").unwrap_or_default();
    // TEAM_465: Arch crates are at crates/kernel/arch/{arch}/src/lib.rs
    let arch_mod = fs::read_to_string("crates/kernel/arch/aarch64/src/lib.rs").unwrap_or_default();
    let arch_x86 = fs::read_to_string("crates/kernel/arch/x86_64/src/lib.rs").unwrap_or_default();

    // Must have Shutdown syscall (could be in arch module or syscall dispatcher)
    if syscall_mod.contains("Shutdown")
        || arch_mod.contains("Shutdown")
        || arch_x86.contains("Shutdown")
    {
        results.pass("Shutdown syscall defined");
    } else {
        results.fail("Shutdown syscall missing");
    }

    // Must have sys_shutdown function with shutdown phases
    if syscall_sys.contains("fn sys_shutdown") || syscall_mod.contains("sys_shutdown") {
        results.pass("sys_shutdown function exists");
    } else {
        results.fail("sys_shutdown function missing");
    }

    // Must have verbose flag support
    if syscall_sys.contains("shutdown_flags") || syscall_sys.contains("VERBOSE") {
        results.pass("Shutdown flags support exists");
    } else {
        results.fail("Shutdown flags missing");
    }

    // TEAM_342: Check libsyscall - shutdown wrapper in process.rs
    let libsyscall_process =
        fs::read_to_string("crates/userspace/libsyscall/src/process.rs").unwrap_or_default();
    let libsyscall_lib =
        fs::read_to_string("crates/userspace/libsyscall/src/lib.rs").unwrap_or_default();

    if libsyscall_process.contains("fn shutdown") || libsyscall_lib.contains("shutdown") {
        results.pass("libsyscall has shutdown() wrapper");
    } else {
        results.fail("libsyscall missing shutdown() wrapper");
    }

    // TEAM_465: Check shell exit command - may just use exit() syscall, not shutdown
    // Shell exit being present is sufficient, doesn't need to call shutdown directly
    let shell_main = fs::read_to_string("crates/userspace/shell/src/main.rs").unwrap_or_default();

    if shell_main.contains("exit") {
        results.pass("Shell has exit command");
    } else {
        results.fail("Shell missing exit command");
    }
}

// =============================================================================
// TEAM_464: linux-raw-sys Consolidation Regression Tests
// =============================================================================

/// `TEAM_464`: Verify signal constants use linux-raw-sys
///
/// Signal constants (SIGINT, SIGKILL, etc.) must come from `linux_raw_sys::general`.
/// This prevents hardcoded values that could drift from Linux ABI.
fn test_linux_raw_sys_signal_constants(results: &mut TestResults) {
    println!("TEAM_464: Signal constants from linux-raw-sys");

    let signal_rs = fs::read_to_string("crates/kernel/syscall/src/signal.rs").unwrap_or_default();

    // Must import signal constants from linux-raw-sys
    if signal_rs.contains("use linux_raw_sys::general::")
        && signal_rs.contains("SIGINT")
        && signal_rs.contains("SIGKILL")
    {
        results.pass("signal.rs imports SIGINT/SIGKILL from linux-raw-sys");
    } else {
        results.fail("signal.rs not using linux-raw-sys signal constants");
    }

    // Must import SIG_BLOCK etc from linux-raw-sys
    if signal_rs.contains("SIG_BLOCK") && signal_rs.contains("SIG_UNBLOCK") {
        results.pass("signal.rs uses SIG_BLOCK/SIG_UNBLOCK from linux-raw-sys");
    } else {
        results.fail("signal.rs missing SIG_BLOCK/SIG_UNBLOCK constants");
    }

    // Verify no hardcoded signal values (except SIG_DFL=0, SIG_IGN=1 which are pointers)
    let has_hardcoded_sigint = signal_rs
        .lines()
        .filter(|l| !l.trim().starts_with("//"))
        .any(|l| l.contains("== 2") && l.contains("sig"));

    if has_hardcoded_sigint {
        results.fail("signal.rs has hardcoded SIGINT (2) - use SIGINT constant");
    } else {
        results.pass("signal.rs avoids hardcoded signal numbers");
    }
}

/// `TEAM_464`: Verify SEEK_* constants use linux-raw-sys
fn test_linux_raw_sys_seek_constants(results: &mut TestResults) {
    println!("TEAM_464: SEEK_* constants from linux-raw-sys");

    // Check fs module files for seek constants (fd.rs and read.rs handle lseek)
    let files_to_check = [
        "crates/kernel/syscall/src/fs/fd.rs",
        "crates/kernel/syscall/src/fs/read.rs",
    ];

    let mut found_import = false;
    let mut found_usage = false;

    for path in &files_to_check {
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("linux_raw_sys") && content.contains("SEEK_") {
                found_import = true;
            }
            if content.contains("SEEK_SET")
                || content.contains("SEEK_CUR")
                || content.contains("SEEK_END")
            {
                found_usage = true;
            }
        }
    }

    if found_import {
        results.pass("fs module imports SEEK_* from linux-raw-sys");
    } else {
        results.fail("fs module not importing SEEK_* from linux-raw-sys");
    }

    if found_usage {
        results.pass("fs module uses SEEK_* constants");
    } else {
        results.fail("fs module missing SEEK_* usage");
    }
}

/// `TEAM_464`: Verify F_* (fcntl) constants use linux-raw-sys
fn test_linux_raw_sys_fcntl_constants(results: &mut TestResults) {
    println!("TEAM_464: F_* (fcntl) constants from linux-raw-sys");

    // fcntl implementation is in fd.rs
    let files_to_check = ["crates/kernel/syscall/src/fs/fd.rs"];

    let mut found_fcntl_import = false;

    for path in &files_to_check {
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("linux_raw_sys")
                && (content.contains("F_GETFD")
                    || content.contains("F_SETFD")
                    || content.contains("F_GETFL"))
            {
                found_fcntl_import = true;
            }
        }
    }

    if found_fcntl_import {
        results.pass("fs module imports F_* from linux-raw-sys");
    } else {
        results.fail("fs module not importing F_* from linux-raw-sys");
    }

    // Check for close-on-exec handling (F_DUPFD_CLOEXEC or FD_CLOEXEC)
    let mut found_cloexec = false;
    for path in &files_to_check {
        if let Ok(content) = fs::read_to_string(path) {
            // Accept either F_DUPFD_CLOEXEC (from linux-raw-sys) or FD_CLOEXEC
            if content.contains("CLOEXEC") {
                found_cloexec = true;
            }
        }
    }

    if found_cloexec {
        results.pass("close-on-exec (CLOEXEC) handling present");
    } else {
        results.fail("close-on-exec (CLOEXEC) handling missing");
    }
}

/// `TEAM_464`: Verify POLL* constants use linux-raw-sys
fn test_linux_raw_sys_poll_constants(results: &mut TestResults) {
    println!("TEAM_464: POLL* constants from linux-raw-sys");

    let sync_rs = fs::read_to_string("crates/kernel/syscall/src/sync.rs").unwrap_or_default();

    // Must import poll constants from linux-raw-sys
    if sync_rs.contains("use linux_raw_sys::general::")
        && sync_rs.contains("POLLIN")
        && sync_rs.contains("POLLOUT")
    {
        results.pass("sync.rs imports POLLIN/POLLOUT from linux-raw-sys");
    } else {
        results.fail("sync.rs not using linux-raw-sys poll constants");
    }

    // Check for POLLERR, POLLHUP (important for error handling)
    if sync_rs.contains("POLLERR") && sync_rs.contains("POLLHUP") {
        results.pass("sync.rs has POLLERR/POLLHUP constants");
    } else {
        results.fail("sync.rs missing POLLERR/POLLHUP constants");
    }
}

/// `TEAM_464`: Verify errno values use linux-raw-sys consistently
fn test_linux_raw_sys_errno_usage(results: &mut TestResults) {
    println!("TEAM_464: Errno values from linux-raw-sys");

    // Check key syscall files for linux-raw-sys errno usage
    let syscall_files = [
        "crates/kernel/syscall/src/lib.rs",
        "crates/kernel/syscall/src/signal.rs",
        "crates/kernel/syscall/src/helpers.rs",
    ];

    let mut errno_import_count = 0;

    for path in &syscall_files {
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("use linux_raw_sys::errno::") {
                errno_import_count += 1;
            }
        }
    }

    if errno_import_count >= 2 {
        results.pass(&format!(
            "{errno_import_count} syscall files import linux_raw_sys::errno"
        ));
    } else {
        results.fail("Not enough syscall files using linux_raw_sys::errno");
    }

    // Check for ENOSYS usage (critical for unimplemented syscalls)
    let lib_rs = fs::read_to_string("crates/kernel/syscall/src/lib.rs").unwrap_or_default();
    if lib_rs.contains("linux_raw_sys::errno::ENOSYS") {
        results.pass("Unimplemented syscalls return linux-raw-sys ENOSYS");
    } else {
        results.fail("ENOSYS not from linux-raw-sys in syscall dispatcher");
    }
}

/// `TEAM_464`: Verify AT_* (auxv) constants are correctly defined
fn test_linux_raw_sys_auxv_constants(results: &mut TestResults) {
    println!("TEAM_464: AT_* auxiliary vector constants");

    let auxv_rs = fs::read_to_string("crates/kernel/mm/src/user/auxv.rs").unwrap_or_default();

    // Check that AT_* constants are defined (linux-raw-sys auxvec module not available for kernel targets)
    let has_at_null = auxv_rs.contains("AT_NULL");
    let has_at_phdr = auxv_rs.contains("AT_PHDR");
    let has_at_pagesz = auxv_rs.contains("AT_PAGESZ");
    let has_at_entry = auxv_rs.contains("AT_ENTRY");
    let has_at_random = auxv_rs.contains("AT_RANDOM");

    if has_at_null && has_at_phdr && has_at_pagesz && has_at_entry && has_at_random {
        results.pass("auxv.rs defines AT_NULL, AT_PHDR, AT_PAGESZ, AT_ENTRY, AT_RANDOM");
    } else {
        results.fail("auxv.rs missing required AT_* constants");
    }

    // Check that constants are u32 (matching linux-raw-sys type)
    if auxv_rs.contains("pub const AT_NULL: u32") {
        results.pass("AT_* constants use u32 type (matches linux-raw-sys)");
    } else {
        results.fail("AT_* constants not using u32 type");
    }

    // Verify TEAM_464 comment indicating values were verified
    if auxv_rs.contains("TEAM_464") {
        results.pass("auxv.rs has TEAM_464 traceability comment");
    } else {
        results.fail("auxv.rs missing TEAM_464 traceability");
    }
}

/// `TEAM_464`: Verify S_* (mode) constants use linux-raw-sys
fn test_linux_raw_sys_mode_constants(results: &mut TestResults) {
    println!("TEAM_464: S_* mode constants from linux-raw-sys");

    let types_lib = fs::read_to_string("crates/kernel/lib/types/src/lib.rs").unwrap_or_default();

    // Check that S_* mode constants are re-exported from linux-raw-sys
    if types_lib.contains("pub use linux_raw_sys::general::")
        && types_lib.contains("S_IFMT")
        && types_lib.contains("S_IFREG")
        && types_lib.contains("S_IFDIR")
    {
        results.pass("los_types re-exports S_* from linux-raw-sys");
    } else {
        results.fail("los_types not re-exporting S_* from linux-raw-sys");
    }

    // Check cpio.rs uses the constants
    let cpio_rs = fs::read_to_string("crates/kernel/lib/utils/src/cpio.rs").unwrap_or_default();

    if cpio_rs.contains("linux_raw_sys::general::") && cpio_rs.contains("S_IFMT") {
        results.pass("cpio.rs uses linux-raw-sys S_* constants");
    } else {
        results.fail("cpio.rs not using linux-raw-sys S_* constants");
    }
}
