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

use crate::fs::vfs::superblock::Superblock;
use crate::task::TaskControlBlock;
use crate::{arch, task};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use los_hal::{InterruptHandler, IrqId, print, println};

#[cfg(target_arch = "aarch64")]
use los_hal::aarch64::timer::Timer;

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

static CURRENT_STAGE: AtomicUsize = AtomicUsize::new(0);

/// TEAM_063: Transition the system to a new boot stage.
/// Enforces order and logs to UART.
pub fn transition_to(stage: BootStage) {
    let stage_val = stage as usize;
    let prev = CURRENT_STAGE.swap(stage_val, Ordering::SeqCst);

    // Ensure we don't go backwards (unless it's the first transition)
    if stage_val < prev && prev != 0 {
        log::warn!(
            "[BOOT] WARNING: Unexpected transition from {:?} to {:?}",
            prev,
            stage
        );
    }

    log::info!("[BOOT] Stage {}: {}", stage_val + 1, stage.name());
}

/// TEAM_063: Robust failsafe shell for critical boot failures.
pub fn maintenance_shell() -> ! {
    use alloc::string::String;
    use alloc::vec::Vec;

    log::info!("\n[BOOT] Entering Maintenance Shell (FAILSAFE)");
    log::info!("Type 'help' for available commands.");

    let mut buffer = String::new();

    loop {
        print!("FAILSAFE> ");

        // Loop for a single command line
        loop {
            if let Some(c) = los_hal::console::read_byte() {
                let ch = c as char;
                match ch {
                    '\r' | '\n' => {
                        println!(""); // New line
                        break;
                    }
                    '\x08' | '\x7f' => {
                        // Backspace / Delete
                        if !buffer.is_empty() {
                            buffer.pop();
                            // Move back, overwrite with space, move back again
                            print!("\x08 \x08");
                        }
                    }
                    c if c.is_control() => {
                        // Ignore other control characters
                    }
                    _ => {
                        // Echo and buffer
                        print!("{}", ch);
                        buffer.push(ch);
                    }
                }
            }
            core::hint::spin_loop();
        }

        // Process command
        let input = buffer.trim();
        if !input.is_empty() {
            let mut parts = input.split_whitespace();
            let cmd = parts.next().unwrap_or("");
            let _args: Vec<&str> = parts.collect();

            match cmd {
                "help" => {
                    println!("Available commands:");
                    println!("  help    - Show this help message");
                    println!("  reboot  - Reboot the system");
                    println!("  info    - Show system information");
                    println!("  clear   - Clear the screen");
                    println!("  panic   - Trigger a kernel panic (for testing)");
                }
                "reboot" => {
                    println!("Rebooting...");
                    // TEAM_063: Simple spin-wait reboot stub until PSCI is fully integrated
                    println!("Reboot not implemented. Use QEMU monitor to reset.");
                    loop {
                        core::hint::spin_loop();
                    }
                }
                "info" => {
                    println!("System Status: FAILSAFE MODE");
                    println!("Stage: {:?}", CURRENT_STAGE.load(Ordering::Relaxed));
                }
                "clear" => {
                    // ANSI escape code to clear screen
                    print!("\x1b[2J\x1b[1;1H");
                }
                "panic" => {
                    panic!("Manual panic triggered from maintenance shell");
                }
                _ => {
                    println!("Unknown command: '{}'. Type 'help' for list.", cmd);
                }
            }
        }

        buffer.clear();
    }
}

// =============================================================================
// IRQ Handlers
// =============================================================================

// TEAM_045: IRQ handlers refactored to use InterruptHandler trait
struct TimerHandler;
impl InterruptHandler for TimerHandler {
    fn handle(&self, _irq: u32) {
        #[cfg(target_arch = "aarch64")]
        {
            // Reload timer for next interrupt (10ms @ 100Hz)
            use los_hal::aarch64::timer;
            let freq = timer::API.read_frequency();
            timer::API.set_timeout(freq / 100);
        }

        // TEAM_089: Keep GPU display active with periodic flush (~10Hz)
        // Only flush every 10th interrupt (100Hz / 10 = 10Hz)
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);

        // TEAM_129: GPU flush was commented out, causing black screen
        if count % 5 == 0 {
            if let Some(mut guard) = crate::gpu::GPU.try_lock() {
                if let Some(gpu_state) = guard.as_mut() {
                    let _ = gpu_state.flush();
                }
            }
        }

        // TEAM_244: Poll UART for Ctrl+C as fallback for terminal mode
        // QEMU doesn't trigger UART RX interrupts when stdin is piped
        if los_hal::console::poll_for_ctrl_c() {
            crate::syscall::signal::signal_foreground_process(crate::syscall::signal::SIGINT);
        }

        // TEAM_070: Preemptive scheduling
        // TEAM_148: Disabled preemption from IRQ context to prevent corruption.
        // IRQ handlers must NOT yield. We rely on cooperative yielding in init/shell.
        // crate::task::yield_now();
    }
}

struct UartHandler;
impl InterruptHandler for UartHandler {
    fn handle(&self, _irq: u32) {
        // TEAM_139: Note - UART RX interrupts may not fire when QEMU stdin is piped
        // Direct UART polling is used as fallback in console::read_byte()
        los_hal::console::handle_interrupt();

        // TEAM_244: Check if Ctrl+C was received and signal foreground process
        // This enables Ctrl+C to work even when no process is reading stdin
        if los_hal::console::check_and_clear_ctrl_c() {
            crate::syscall::signal::signal_foreground_process(crate::syscall::signal::SIGINT);
        }
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

    // --- Stage 3: Boot Console (GPU, Terminal) ---
    transition_to(BootStage::BootConsole);
    init_display();

    // --- Interrupt Controller and Timer Setup ---
    {
        // TEAM_316: x86_64 Multiboot path - skip APIC/IOAPIC init because phys_to_virt()
        // for APIC addresses (0xFEE00000) fails - outside 1GB PMO range.
        // PIT still works via I/O ports. Interrupts use legacy PIC mode.
        #[cfg(not(target_arch = "x86_64"))]
        {
            // TEAM_255: Initialize interrupt controller using generic HAL interface
            let ic = los_hal::active_interrupt_controller();
            ic.init();

            // TEAM_255/TEAM_303: Register IRQ handlers using generic HAL traits
            ic.register_handler(IrqId::VirtualTimer, &TIMER_HANDLER);
            ic.register_handler(IrqId::Uart, &UART_HANDLER);
            ic.enable_irq(ic.map_irq(IrqId::VirtualTimer));
            ic.enable_irq(ic.map_irq(IrqId::Uart));
        }

        #[cfg(target_arch = "x86_64")]
        {
            // TEAM_316: Skip IOAPIC routing - uses phys_to_virt() which fails
            // PIT is already initialized in HAL, just ensure it's running
            los_hal::pit::Pit::init(100);
            
            // TEAM_318: Register timer handler for GPU flush on x86_64
            // Uses existing apic::register_handler() which doesn't need MMIO access
            los_hal::x86_64::interrupts::apic::register_handler(32, &TIMER_HANDLER);
        }

        crate::verbose!("Core drivers initialized.");

        #[cfg(target_arch = "aarch64")]
        {
            use los_hal::aarch64::timer;
            // Set initial timer timeout
            timer::API.set_timeout(timer::API.read_frequency() / 100);
            timer::API.enable();
        }

        crate::verbose!("Timer initialized.");
    }

    // --- Stage 4: Discovery (VirtIO, Filesystem, Init) ---
    transition_to(BootStage::Discovery);
    init_devices();
    arch::print_boot_regs();

    let initrd_found = init_userspace();

    // TEAM_065: SPEC-4 enforcement per Rule 14 (Fail Loud, Fail Fast)
    #[cfg(not(feature = "diskless"))]
    if !initrd_found {
        log::error!("[BOOT] SPEC-4: Initrd required but not found.");
        log::error!("\n[ERROR] Initramfs not found!");
        log::info!("Dropping to maintenance shell...");
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

    log::info!("\n[SUCCESS] LevitateOS System Ready.");
    log::info!("--------------------------------------");

    // TEAM_146: Enable interrupts and exit bootstrap task.
    // Note: verbose! calls removed here due to race condition - timer interrupt
    // can fire immediately after enable() and preempt before verbose! executes.
    // SAFETY: Enabling interrupts is required for multitasking and timer-based
    // scheduling. This is safe as we have finished core initialization.
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
        log::warn!("[BOOT] SPEC-1: No GPU found, using serial-only console");
    }

    // SC2.2, SC2.3: Get resolution from GPU
    let (_width, _height) = match crate::gpu::get_resolution() {
        Some((w, h)) => {
            log::info!("[TERM] GPU resolution: {}x{}", w, h);
            (w, h)
        }
        None => {
            // SPEC-1: Fallback resolution for terminal sizing only
            log::warn!("[TERM] SPEC-1: Using fallback resolution 1280x800 (serial mode)");
            (1280, 800)
        }
    };

    // TEAM_081: Initialize global GPU terminal for dual console output
    crate::terminal::init();

    // TEAM_087: Re-enable dual console now that GPU deadlock is fixed (TEAM_086)
    los_hal::console::set_secondary_output(crate::terminal::write_str);

    log::info!("Terminal initialized.");
    
    // TEAM_320: Output GPU display status to host shell for debugging
    crate::gpu::debug_display_status();
}

/// Initialize VirtIO devices (block, network, input).
fn init_devices() {
    crate::virtio::init();
}

/// Initialize userspace: load and spawn init from initramfs.
///
/// Returns true if init was successfully spawned.
fn init_userspace() -> bool {
    // TEAM_284: Use unified BootInfo to locate initramfs
    let boot_info =
        crate::boot::boot_info().expect("BootInfo must be available for userspace init");

    log::info!("Detecting Initramfs...");

    let Some(initrd_region) = boot_info.initramfs else {
        log::error!("[BOOT] ERROR: No initramfs provided in BootInfo");
        return false;
    };

    let start = initrd_region.base;
    let size = initrd_region.size;
    let end = start + size;

    log::debug!(
        "Initramfs found at 0x{:x} - 0x{:x} ({} bytes)",
        start,
        end,
        size
    );

    // TEAM_289: Limine module.addr() returns VA directly (HHDM-mapped).
    // Only call phys_to_virt if address looks like a physical address.
    // HHDM VAs start at 0xffff800... on x86_64. Limine loads modules into HHDM.
    let initrd_va = if start >= 0xFFFF_0000_0000_0000 {
        // Already a virtual address (HHDM or kernel higher-half)
        start
    } else {
        // Physical address - translate
        los_hal::mmu::phys_to_virt(start)
    };
    // SAFETY: The initramfs region is provided by the bootloader and is guaranteed
    // to be valid and mapped in the HHDM.
    let initrd_slice = unsafe { core::slice::from_raw_parts(initrd_va as *const u8, size) };

    // TEAM_205: Initialize Initramfs VFS
    let sb = crate::fs::initramfs::init_vfs(initrd_slice);

    // Set as root in dcache
    let root_inode = sb.root();
    crate::fs::vfs::dcache().set_root(crate::fs::vfs::Dentry::root(root_inode));

    log::debug!("Files in initramfs:");
    for entry in sb.archive.iter() {
        log::trace!(" - {}", entry.name);
    }

    // TEAM_120: Store initramfs globally for syscalls
    {
        let mut global_archive = crate::fs::INITRAMFS.lock();
        *global_archive = Some(sb);
    }

    // Spawn init from initramfs
    spawn_init()
}

/// Spawn the init process from initramfs.
fn spawn_init() -> bool {
    log::info!("[BOOT] spawning init task...");

    // Get copy of archive to avoid holding the lock
    let archive_data = {
        let archive_lock = crate::fs::INITRAMFS.lock();
        archive_lock
            .as_ref()
            .and_then(|sb| sb.archive.iter().find(|e| e.name == "init"))
            .map(|e| e.data)
    };

    let Some(elf_data) = archive_data else {
        log::error!("[BOOT] ERROR: init not found in initramfs");
        return false;
    };

    // TEAM_121: Spawn init and let the scheduler take over.
    match task::process::spawn_from_elf(elf_data, task::fd_table::new_shared_fd_table()) {
        Ok(task) => {
            let tcb = Arc::new(TaskControlBlock::from(task));
            task::scheduler::SCHEDULER.add_task(tcb);
            log::info!("[BOOT] Init process scheduled.");
            true
        }
        Err(e) => {
            log::error!("[BOOT] ERROR: Failed to spawn init: {:?}", e);
            false
        }
    }
}

/// Initialize filesystem subsystem.
fn init_filesystem() {
    log::info!("Mounting filesystems...");

    // TEAM_207: Initialize mount table (registers / and /tmp entries)
    crate::fs::mount::init();

    match crate::fs::init() {
        Ok(_) => {
            log::info!("[BOOT] Filesystem initialized successfully.");
        }
        Err(e) => {
            log::warn!("[BOOT] WARNING: Filesystem mount failed: {}", e);
            log::warn!("  (Check tinyos_disk.img format/presence)");
        }
    }

    // TEAM_207: Initialize tmpfs and mount at /tmp dentry
    crate::fs::tmpfs::init();
    mount_tmpfs_at_dentry();
}

/// TEAM_207: Mount tmpfs at /tmp in the dcache
fn mount_tmpfs_at_dentry() {
    use crate::fs::vfs::dentry::Dentry;
    use crate::fs::vfs::superblock::Superblock;
    use alloc::string::String;

    // Get tmpfs superblock
    let tmpfs_lock = crate::fs::tmpfs::TMPFS.lock();
    let Some(tmpfs) = tmpfs_lock.as_ref() else {
        log::warn!("[BOOT] WARNING: tmpfs not initialized");
        return;
    };

    // Get root dentry
    let root = match crate::fs::vfs::dcache().root() {
        Some(r) => r,
        None => {
            log::warn!("[BOOT] WARNING: No root dentry for tmpfs mount");
            return;
        }
    };

    // Create /tmp dentry if it doesn't exist
    let tmp_dentry = root.lookup_child("tmp").unwrap_or_else(|| {
        let d = Arc::new(Dentry::new(
            String::from("tmp"),
            Some(Arc::downgrade(&root)),
            None, // No inode yet - mount will provide it
        ));
        root.add_child(Arc::clone(&d));
        d
    });

    // Mount tmpfs at this dentry
    tmp_dentry.mount(Arc::clone(tmpfs) as Arc<dyn Superblock>);
    crate::verbose!("[BOOT] Mounted tmpfs at /tmp");
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
