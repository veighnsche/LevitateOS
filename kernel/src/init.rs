//! TEAM_146: System Initialization and Device Discovery
//!
//! This module contains initialization logic that changes frequently:
//! - Boot stage management
//! - Device discovery and driver initialization
//! - Filesystem mounting
//! - Userspace handoff
//!
//! Separated from boot.rs (arch-specific boot) for upgradability.

extern crate alloc;

use alloc::sync::Arc;

use los_hal::fdt::{self, Fdt};
use los_hal::gic;
use los_hal::mmu;
use los_hal::timer::{self, Timer};
use los_hal::{print, println};

use crate::arch;
use crate::task;

// =============================================================================
// Boot Stage Management
// =============================================================================

// TEAM_063: Define type-safe boot stages aligned with UEFI/Linux
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    EarlyHAL,    // SEC / setup_arch
    MemoryMMU,   // PEI / mm_init
    BootConsole, // DXE / console_init
    Discovery,   // DXE / BDS / vfs_caches_init
}

impl BootStage {
    pub const fn name(&self) -> &'static str {
        match self {
            BootStage::EarlyHAL => "Early HAL (SEC)",
            BootStage::MemoryMMU => "Memory & MMU (PEI)",
            BootStage::BootConsole => "Boot Console (DXE)",
            BootStage::Discovery => "Discovery & FS (DXE/BDS)",
        }
    }
}

static CURRENT_STAGE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// TEAM_063: Transition the system to a new boot stage.
/// Enforces order and logs to UART.
pub fn transition_to(stage: BootStage) {
    let stage_val = stage as u8;
    let prev = CURRENT_STAGE.swap(stage_val, core::sync::atomic::Ordering::SeqCst);

    // Ensure we don't go backwards (unless it's the first transition)
    if stage_val < prev && prev != 0 {
        println!(
            "[BOOT] WARNING: Unexpected transition from {:?} to {:?}",
            prev, stage
        );
    }

    println!("[BOOT] Stage {}: {}", stage_val + 1, stage.name());
}

/// TEAM_063: Minimal failsafe shell for critical boot failures.
pub fn maintenance_shell() -> ! {
    println!("\n[BOOT] Entering Maintenance Shell (FAILSAFE)");
    println!("Type 'reboot' to restart (not implemented) or interact via serial.");
    loop {
        print!("FAILSAFE> ");
        if let Some(c) = los_hal::console::read_byte() {
            let ch = c as char;
            match ch {
                '\r' | '\n' => println!(""),
                _ => print!("{}", ch),
            }
        }
        core::hint::spin_loop();
    }
}

// =============================================================================
// IRQ Handlers
// =============================================================================

// TEAM_045: IRQ handlers refactored to use InterruptHandler trait
struct TimerHandler;
impl gic::InterruptHandler for TimerHandler {
    fn handle(&self, _irq: u32) {
        // TEAM_083: Removed debug "T" output - was flooding console
        // Reload timer for next interrupt (10ms @ 100Hz)
        let freq = timer::API.read_frequency();
        timer::API.set_timeout(freq / 100);

        // TEAM_089: Keep GPU display active with periodic flush (~10Hz)
        // Only flush every 10th interrupt (100Hz / 10 = 10Hz)
        static COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let count = COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

        // TEAM_092: Verify timer is actually advancing (verbose-only)
        // TEAM_122: Removed verbose log to prevent deadlocks with UART lock
        // TEAM_148: Disabled [TICK] output to prevents prompt interleaving failures in behavior tests
        // if count % 100 == 0 {
        //     crate::verbose!("[TICK] count={}", count);
        // }

        // TEAM_129: GPU flush was commented out, causing black screen
        if count % 5 == 0 {
            if let Some(mut guard) = crate::gpu::GPU.try_lock() {
                if let Some(gpu_state) = guard.as_mut() {
                    let _ = gpu_state.flush();
                }
            }
        }

        // TEAM_070: Preemptive scheduling
        // TEAM_148: Disabled preemption from IRQ context to prevent corruption.
        // IRQ handlers must NOT yield. We rely on cooperative yielding in init/shell.
        // crate::task::yield_now();
    }
}

struct UartHandler;
impl gic::InterruptHandler for UartHandler {
    fn handle(&self, _irq: u32) {
        // TEAM_139: Note - UART RX interrupts may not fire when QEMU stdin is piped
        // Direct UART polling is used as fallback in console::read_byte()
        los_hal::console::handle_interrupt();
    }
}

static TIMER_HANDLER: TimerHandler = TimerHandler;
static UART_HANDLER: UartHandler = UartHandler;

// =============================================================================
// Initialization Sequence
// =============================================================================

/// Run the full system initialization sequence.
///
/// This is called from kmain after early boot (heap, MMU) is complete.
/// It initializes all subsystems and spawns the init process.
pub fn run() -> ! {
    // --- Stage 2: Memory & MMU (already done in kmain, just transition) ---
    transition_to(BootStage::MemoryMMU);
    arch::init_mmu();

    arch::exceptions::init();
    crate::verbose!("Exceptions initialized.");

    // --- GIC and Timer Setup ---
    let dtb_phys = arch::get_dtb_phys();
    let dtb_slice = dtb_phys.map(|phys| {
        let ptr = phys as *const u8;
        // Assume 1MB for early discovery
        unsafe { core::slice::from_raw_parts(ptr, 1024 * 1024) }
    });
    let fdt = dtb_slice.and_then(|slice| Fdt::new(slice).ok());

    let gic_api = gic::get_api(fdt.as_ref());
    crate::verbose!("Detected GIC version: {:?}", gic_api.version());
    gic_api.init();

    // TEAM_047: Initialize physical memory management (Buddy Allocator)
    if let Some(slice) = dtb_slice {
        crate::memory::init(slice);
    }

    // TEAM_045: Register IRQ handlers using trait objects
    gic::register_handler(gic::IrqId::VirtualTimer, &TIMER_HANDLER);
    gic::register_handler(gic::IrqId::Uart, &UART_HANDLER);
    gic_api.enable_irq(gic::IrqId::VirtualTimer.irq_number());
    gic_api.enable_irq(gic::IrqId::Uart.irq_number());

    crate::verbose!("Core drivers initialized.");

    // --- Multitasking Setup ---
    let bootstrap_task = Arc::new(task::TaskControlBlock::new_bootstrap());
    unsafe {
        task::set_current_task(bootstrap_task);
    }

    // TEAM_071: Demo tasks gated behind feature to not break behavior tests
    #[cfg(feature = "multitask-demo")]
    {
        task::scheduler::SCHEDULER.add_task(Arc::new(task::TaskControlBlock::new(
            task1 as *const () as usize,
        )));
        task::scheduler::SCHEDULER.add_task(Arc::new(task::TaskControlBlock::new(
            task2 as *const () as usize,
        )));
    }

    // Set initial timer timeout
    timer::API.set_timeout(timer::API.read_frequency() / 100);
    timer::API.enable();
    crate::verbose!("Timer initialized.");

    // --- Stage 3: Boot Console (GPU + Terminal) ---
    transition_to(BootStage::BootConsole);
    init_display();

    // --- Stage 4: Discovery (VirtIO, Filesystem, Init) ---
    transition_to(BootStage::Discovery);
    init_devices();
    arch::print_boot_regs();

    let initrd_found = init_userspace(dtb_slice);

    // TEAM_065: SPEC-4 enforcement per Rule 14 (Fail Loud, Fail Fast)
    #[cfg(not(feature = "diskless"))]
    if !initrd_found {
        println!("[BOOT] SPEC-4: Initrd required but not found.");
        println!("\n[ERROR] Initramfs not found!");
        println!("Dropping to maintenance shell...");
        maintenance_shell();
    }

    #[cfg(feature = "diskless")]
    if !initrd_found {
        crate::verbose!("Diskless mode: continuing without initrd");
    }

    // Initialize Filesystem (Phase 4)
    init_filesystem();

    // GPU regression verification
    verify_gpu_state();

    println!("\n[SUCCESS] LevitateOS System Ready.");
    println!("--------------------------------------");

    // TEAM_146: Enable interrupts and exit bootstrap task.
    // Note: verbose! calls removed here due to race condition - timer interrupt
    // can fire immediately after enable() and preempt before verbose! executes.
    unsafe { los_hal::interrupts::enable() };
    crate::task::task_exit();
}

// =============================================================================
// Initialization Helpers
// =============================================================================

/// Initialize GPU and terminal display.
fn init_display() {
    // TEAM_065: Initialize GPU FIRST (Stage 3 requirement)
    let gpu_available = crate::virtio::init_gpu();

    if gpu_available {
        crate::verbose!("GPU initialized successfully.");
    } else {
        // SPEC-1: Explicit fallback to serial-only mode
        println!("[BOOT] SPEC-1: No GPU found, using serial-only console");
    }

    // SC2.2, SC2.3: Get resolution from GPU
    let (_width, _height) = match crate::gpu::get_resolution() {
        Some((w, h)) => {
            println!("[TERM] GPU resolution: {}x{}", w, h);
            (w, h)
        }
        None => {
            // SPEC-1: Fallback resolution for terminal sizing only
            println!("[TERM] SPEC-1: Using fallback resolution 1280x800 (serial mode)");
            (1280, 800)
        }
    };

    // TEAM_081: Initialize global GPU terminal for dual console output
    crate::terminal::init();

    // TEAM_087: Re-enable dual console now that GPU deadlock is fixed (TEAM_086)
    los_hal::console::set_secondary_output(crate::terminal::write_str);

    println!("Terminal initialized.");
}

/// Initialize VirtIO devices (block, network, input).
fn init_devices() {
    crate::virtio::init();
}

/// Initialize userspace: load and spawn init from initramfs.
///
/// Returns true if init was successfully spawned.
fn init_userspace(dtb_slice: Option<&[u8]>) -> bool {
    // TEAM_121: SPEC-4 Initrd Discovery with Fail-Fast (Rule 14)
    println!("Detecting Initramfs...");

    let Some(slice) = dtb_slice else {
        println!("[BOOT] ERROR: No DTB provided - cannot locate initramfs");
        return false;
    };

    match fdt::get_initrd_range(slice) {
        Ok((start, end)) => {
            let size = end - start;
            println!(
                "Initramfs found at 0x{:x} - 0x{:x} ({} bytes)",
                start, end, size
            );

            // Initramfs access via High VA (TTBR1) for stability
            let initrd_va = mmu::phys_to_virt(start);
            let initrd_slice = unsafe { core::slice::from_raw_parts(initrd_va as *const u8, size) };
            let archive = crate::fs::initramfs::CpioArchive::new(initrd_slice);

            println!("Files in initramfs:");
            for entry in archive.iter() {
                println!(" - {}", entry.name);
            }

            // TEAM_120: Store initramfs globally for syscalls
            {
                let mut global_archive = crate::fs::INITRAMFS.lock();
                *global_archive = Some(archive);
            }

            // Spawn init from initramfs
            spawn_init()
        }
        Err(e) => {
            println!("[BOOT] ERROR: No initramfs in DTB: {:?}", e);
            false
        }
    }
}

/// Spawn the init process from initramfs.
fn spawn_init() -> bool {
    println!("[BOOT] spawning init task...");

    // Get copy of archive to avoid holding the lock
    let archive_data = {
        let archive_lock = crate::fs::INITRAMFS.lock();
        archive_lock
            .as_ref()
            .and_then(|a| a.iter().find(|e| e.name == "init"))
            .map(|e| e.data)
    };

    let Some(elf_data) = archive_data else {
        println!("[BOOT] ERROR: init not found in initramfs");
        return false;
    };

    // TEAM_121: Spawn init and let the scheduler take over.
    match task::process::spawn_from_elf(elf_data) {
        Ok(task) => {
            let tcb = Arc::new(task::TaskControlBlock::from(task));
            task::scheduler::SCHEDULER.add_task(tcb);
            println!("[BOOT] Init process scheduled.");
            true
        }
        Err(e) => {
            println!("[BOOT] ERROR: Failed to spawn init: {:?}", e);
            false
        }
    }
}

/// Initialize filesystem subsystem.
fn init_filesystem() {
    println!("Mounting filesystems...");
    match crate::fs::init() {
        Ok(_) => {
            println!("[BOOT] Filesystem initialized successfully.");
        }
        Err(e) => {
            println!("[BOOT] WARNING: Filesystem mount failed: {}", e);
            println!("  (Check tinyos_disk.img format/presence)");
        }
    }

    // TEAM_195: Initialize tmpfs for writable /tmp
    crate::fs::tmpfs::init();
    println!("[BOOT] Tmpfs initialized at /tmp");
}

/// TEAM_129: GPU regression test verification.
fn verify_gpu_state() {
    // Manually trigger one GPU flush to ensure framebuffer content is pushed
    if let Some(mut guard) = crate::gpu::GPU.try_lock() {
        if let Some(gpu_state) = guard.as_mut() {
            let _ = gpu_state.flush();
        }
    }

    let flush_count = crate::gpu::flush_count();
    crate::verbose!("[GPU_TEST] Flush count: {}", flush_count);
    if flush_count == 0 {
        crate::verbose!("[GPU_TEST] WARNING: GPU flush count is 0 - display may be black!");
    }

    if let Some((_total, non_black)) = crate::gpu::framebuffer_has_content() {
        crate::verbose!(
            "[GPU_TEST] Framebuffer: {} non-black pixels of {} total",
            non_black,
            _total
        );
        if non_black == 0 {
            crate::verbose!(
                "[GPU_TEST] WARNING: Framebuffer is entirely black - no content rendered!"
            );
        }
    }
}

// =============================================================================
// Demo Tasks (Feature-Gated)
// =============================================================================

// TEAM_071: Demo tasks for preemption verification
#[cfg(feature = "multitask-demo")]
fn task1() {
    loop {
        println!("[TASK1] Hello from task 1!");
        for _ in 0..5000000 {
            core::hint::spin_loop();
        }
    }
}

#[cfg(feature = "multitask-demo")]
fn task2() {
    loop {
        println!("[TASK2] Hello from task 2!");
        for _ in 0..5000000 {
            core::hint::spin_loop();
        }
    }
}
