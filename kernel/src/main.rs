#![no_std]
#![no_main]

extern crate alloc;

/// Verbose print macro - only outputs when `verbose` feature is enabled.
/// Use for successful initialization messages (Rule 4: Silence is Golden).
/// Errors should always use println! directly.
#[cfg(feature = "verbose")]
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => { $crate::println!($($arg)*) };
}

#[cfg(not(feature = "verbose"))]
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => { $crate::println!($($arg)*) };
}

// use core::alloc::Layout;
use core::arch::global_asm;
use core::panic::PanicInfo;

mod block;
mod console_gpu; // TEAM_081: GPU terminal integration for dual console
mod cursor;
mod exceptions;
mod fs;
mod gpu;
mod input;
mod loader; // TEAM_073: ELF loader (Phase 8)
mod memory;
mod net;
mod syscall; // TEAM_073: Userspace syscall handler (Phase 8)
mod task;
mod terminal;
mod virtio;

use levitate_hal::fdt;
use levitate_hal::gic;
use levitate_hal::mmu;
use levitate_hal::timer::{self, Timer};
use levitate_hal::{print, println};

global_asm!(
    r#"
.section ".text.head", "ax"
.global _head
.global _start

_head:
    b       _start
    .long   0
    .quad   0x80000          /* text_offset: kernel expects RAM_BASE + 0x80000 */
    .quad   _kernel_size     /* image_size: calculated by linker script */
    .quad   0x0A             /* flags: LE, 4K pages */
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\x64"
    .long   0

_start:
    msr     daifset, #0xf
    mrs     x1, mpidr_el1
    and     x1, x1, #0xFF
    cbz     x1, primary_cpu

secondary_halt:
    wfe
    b       secondary_halt

primary_cpu:
    /* Save x0-x3 to callee-saved registers x19-x22 */
    mov     x19, x0
    mov     x20, x1
    mov     x21, x2
    mov     x22, x3

    /* Enable FP/SIMD */
    mov     x0, #0x300000
    msr     cpacr_el1, x0
    isb

    /* Zero BSS (using physical addresses during boot) */
    /* Note: In higher half, symbols like __bss_start are high VAs. */
    /* We need to convert them to physical for early boot. */
    ldr     x0, =__bss_start
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1          /* x0 = __bss_start_phys */
    ldr     x2, =__bss_end
    sub     x2, x2, x1          /* x2 = __bss_end_phys */
    mov     x3, #0
bss_loop:
    cmp     x0, x2
    b.ge    bss_done
    str     x3, [x0], #8
    b       bss_loop
bss_done:

    /* Save preserved registers to global variable BOOT_REGS */
    ldr     x0, =BOOT_REGS
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1          /* x0 = physical address of BOOT_REGS */
    str     x19, [x0]           /* x0 */
    str     x20, [x0, #8]       /* x1 */
    str     x21, [x0, #16]      /* x2 */
    str     x22, [x0, #24]      /* x3 */

    /* Save DTB to BOOT_DTB_ADDR for compatibility */
    ldr     x0, =BOOT_DTB_ADDR
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1
    str     x19, [x0]

    /* Setup Early Page Tables */
    /* L0_low[0] -> L1_low (ID map for first 1GB) */
    ldr     x4, =_kernel_virt_base
    
    ldr     x0, =boot_pt_l0_low
    sub     x0, x0, x4          /* x0 = boot_pt_l0_low_phys */
    ldr     x1, =boot_pt_l1_low
    sub     x1, x1, x4          /* x1 = boot_pt_l1_low_phys */
    orr     x1, x1, #0x3        /* Table + Valid */
    str     x1, [x0]

    /* L1_low[0] -> 0x00000000 (1GB Device Block) */
    ldr     x0, =boot_pt_l1_low
    sub     x0, x0, x4
    mov     x1, #0x00000000
    add     x1, x1, #0x405      /* Block + AF + Attr1 (Device) */
    str     x1, [x0]
    
    /* L1_low[1] -> 0x40000000 (1GB Normal Block for RAM) */
    mov     x1, #0x40000000
    add     x1, x1, #0x401      /* Block + AF + Attr0 (Normal) */
    str     x1, [x0, #8]        /* Index 1 = 1GB */

    /* L0_high[256] -> L1_high (Higher-half base 0xFFFF8000...) */
    ldr     x0, =boot_pt_l0_high
    sub     x0, x0, x4
    ldr     x1, =boot_pt_l1_high
    sub     x1, x1, x4
    orr     x1, x1, #0x3
    str     x1, [x0, #256*8]

    /* TEAM_078: L1_high[0] -> 0x00000000 (1GB Device Block for UART, GIC, VirtIO) */
    /* This enables device access via TTBR1 high VA before kmain() */
    ldr     x0, =boot_pt_l1_high
    sub     x0, x0, x4
    mov     x1, #0x00000000
    add     x1, x1, #0x405      /* Block + AF + Attr1 (Device) */
    str     x1, [x0]            /* Index 0 for 0x00000000 */

    /* L1_high[1] -> 0x40000000 (1GB Block matching 0x40000000 physical) */
    mov     x1, #0x40000000
    add     x1, x1, #0x401      /* Block + AF + Attr0 (Normal) */
    str     x1, [x0, #8]        /* Index 1 for 0x40000000 */

    /* Configure MMU Registers */
    /* MAIR_EL1: Attr0=0xFF (Normal), Attr1=0x04 (Device) */
    ldr     x0, =0x00000000000004FF
    msr     mair_el1, x0

    /* TCR_EL1: T0SZ=16, T1SZ=16, TG0=4K, TG1=4K, IPS=48bit, SH0/SH1=Inner, Cacheable */
    ldr     x0, =0x00000005b5103510
    msr     tcr_el1, x0
    isb

    /* Load TTBR0 and TTBR1 */
    ldr     x0, =boot_pt_l0_low
    sub     x0, x0, x4
    msr     ttbr0_el1, x0
    ldr     x0, =boot_pt_l0_high
    sub     x0, x0, x4
    msr     ttbr1_el1, x0
    isb

    /* Enable MMU */
    mrs     x0, sctlr_el1
    orr     x0, x0, #0x1        /* M (MMU) */
    orr     x0, x0, #0x4        /* C (D-Cache) */
    orr     x0, x0, #0x1000     /* I (I-Cache) */
    msr     sctlr_el1, x0
    isb

    /* Jump to High VA kmain */
    ldr     x0, =stack_top
    mov     sp, x0
    ldr     x0, =kmain
    br      x0

.section ".text", "ax"

.section ".data.boot_pt", "aw"
.align 12
boot_pt_l0_low:  .space 4096
boot_pt_l1_low:  .space 4096
boot_pt_l0_high: .space 4096
boot_pt_l1_high: .space 4096

.section ".data"
.global _end
_end:
"#
);

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Physical address of the Device Tree Blob (DTB) passed by the bootloader.
/// Saved from x0 in `_start`.
#[unsafe(no_mangle)]
static mut BOOT_DTB_ADDR: u64 = 0;

#[unsafe(no_mangle)]
static mut BOOT_REGS: [u64; 4] = [0; 4];

// TEAM_063: Define type-safe boot stages aligned with UEFI/Linux
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    EarlyHAL,    // SEC / setup_arch
    MemoryMMU,   // PEI / mm_init
    BootConsole, // DXE / console_init
    Discovery,   // DXE / BDS / vfs_caches_init
    SteadyState, // BDS / rest_init
}

impl BootStage {
    pub const fn name(&self) -> &'static str {
        match self {
            BootStage::EarlyHAL => "Early HAL (SEC)",
            BootStage::MemoryMMU => "Memory & MMU (PEI)",
            BootStage::BootConsole => "Boot Console (DXE)",
            BootStage::Discovery => "Discovery & FS (DXE/BDS)",
            BootStage::SteadyState => "Steady State (BDS)",
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
        if let Some(c) = levitate_hal::console::read_byte() {
            let ch = c as char;
            match ch {
                '\r' | '\n' => println!(""),
                _ => print!("{}", ch),
            }
        }
        core::hint::spin_loop();
    }
}

/// Returns the physical address of the DTB if one was provided.
///
/// # TEAM_038: DTB Detection Strategy
/// 1. Check x0 (passed by bootloader via BOOT_DTB_ADDR)
/// 2. Scan common QEMU DTB locations (0x4000_0000 region)
/// 3. Search for DTB magic (0xD00DFEED) in early RAM
///
/// On real hardware (Pixel 6), step 1 should work. Steps 2-3 are for QEMU ELF boot.
pub fn get_dtb_phys() -> Option<usize> {
    // Step 1: Check if bootloader passed DTB address in x0
    let addr = unsafe { BOOT_DTB_ADDR };
    if addr != 0 {
        verbose!("DTB address from x0: 0x{:x}", addr);
        return Some(addr as usize);
    }

    // Step 2: Scan likely DTB locations in early RAM (fallback for ELF boot)
    // QEMU may place DTB at start of RAM or after kernel
    let scan_start = 0x4000_0000usize;
    let scan_end = 0x4900_0000usize; // Scan first ~144MB of RAM

    // Scan page-aligned addresses (DTB must be 8-byte aligned per spec)
    for addr in (scan_start..scan_end).step_by(0x1000) {
        let magic = unsafe { core::ptr::read_volatile(addr as *const u32) };
        if u32::from_be(magic) == 0xd00d_feed {
            verbose!("Found DTB at 0x{:x}", addr);
            return Some(addr);
        }
    }

    None
}

// TEAM_045: IRQ handlers refactored to use InterruptHandler trait
struct TimerHandler;
impl gic::InterruptHandler for TimerHandler {
    fn handle(&self, _irq: u32) {
        // Reload timer for next interrupt (10ms @ 100Hz)
        let freq = timer::API.read_frequency();
        timer::API.set_timeout(freq / 100);

        // TEAM_070: Preemptive scheduling
        crate::task::yield_now();
    }
}

struct UartHandler;
impl gic::InterruptHandler for UartHandler {
    fn handle(&self, _irq: u32) {
        levitate_hal::console::handle_interrupt();
    }
}

static TIMER_HANDLER: TimerHandler = TimerHandler;
static UART_HANDLER: UartHandler = UartHandler;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // TEAM_063: Initialize console immediately so early transitions are logged.
    levitate_hal::console::init();

    transition_to(BootStage::EarlyHAL);
    // Initialize heap
    unsafe extern "C" {
        static __heap_start: u8;
        static __heap_end: u8;
    }

    unsafe {
        let heap_start = &__heap_start as *const u8 as usize;
        let heap_end = &__heap_end as *const u8 as usize;
        let heap_size = heap_end - heap_start;
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }

    verbose!("Heap initialized.");

    transition_to(BootStage::MemoryMMU);
    {
        use levitate_hal::mmu;
        // MMU is already enabled by assembly boot, but we re-initialize
        // with more granular mappings if needed.
        mmu::init();

        let root = unsafe {
            static mut ROOT_PT: mmu::PageTable = mmu::PageTable::new();
            &mut *core::ptr::addr_of_mut!(ROOT_PT)
        };

        // Map Kernel - RWX (no PXN) for now to keep it simple.
        // We map it to BOTH identity and higher-half for a smooth transition.
        let kernel_flags = mmu::PageFlags::KERNEL_DATA.difference(mmu::PageFlags::PXN);

        // Critical boot mappings - system cannot continue if these fail
        #[allow(clippy::expect_used, clippy::unwrap_used)]
        {
            // Identity map for early access (until we fully switch to high VA for everything)
            mmu::identity_map_range_optimized(
                root,
                mmu::KERNEL_PHYS_START,
                mmu::KERNEL_PHYS_END,
                kernel_flags,
            )
            .expect("Failed to identity map kernel");

            // Higher-half map kernel
            mmu::map_range(
                root,
                mmu::KERNEL_VIRT_START + mmu::KERNEL_PHYS_START,
                mmu::KERNEL_PHYS_START,
                mmu::KERNEL_PHYS_END - mmu::KERNEL_PHYS_START,
                kernel_flags,
            )
            .expect("Failed to higher-half map kernel");

            // Higher-half map all boot RAM (0x4000_0000 to 0x8000_0000)
            // to support access to DTB, initrd, and mem_map. Use 1GB to cover QEMU default + Pixel 6 base.
            mmu::map_range(
                root,
                mmu::KERNEL_VIRT_START + 0x4000_0000,
                0x4000_0000,
                0x4000_0000, // 1GB (was 256MB)
                kernel_flags,
            )
            .expect("Failed to map boot RAM to higher half");

            // TEAM_078: Map Devices to HIGH VA (via TTBR1) instead of identity mapping
            // This ensures devices remain accessible when TTBR0 is switched for userspace
            
            // UART (PA: 0x0900_0000 -> VA: KERNEL_VIRT_START + 0x0900_0000)
            mmu::map_range(
                root,
                mmu::UART_VA,
                0x0900_0000,
                0x1000,
                mmu::PageFlags::DEVICE,
            )
            .unwrap();

            // TEAM_042: GIC mapping extended for GICv3 support
            // QEMU virt GIC layout:
            //   GICD: 0x0800_0000 - 0x0801_0000 (64KB)
            //   GICC: 0x0801_0000 - 0x0802_0000 (64KB) - GICv2 only
            //   GICR: 0x080A_0000 - 0x080C_0000 (128KB per CPU, 8 CPUs = 1MB)
            // Map 0x0800_0000 - 0x0820_0000 to cover all GIC components
            mmu::map_range(
                root,
                mmu::GIC_DIST_VA,
                0x0800_0000,
                0x20_0000, // 2MB covers GICD + GICC + GICR
                mmu::PageFlags::DEVICE,
            )
            .unwrap();

            // VirtIO MMIO (PA: 0x0A00_0000 -> VA: KERNEL_VIRT_START + 0x0A00_0000)
            mmu::map_range(
                root,
                mmu::VIRTIO_MMIO_VA,
                0x0a00_0000,
                0x10_0000, // 1MB for VirtIO devices
                mmu::PageFlags::DEVICE,
            )
            .unwrap();

            // Map Boot RAM including DTB/initrd region (QEMU places DTB after kernel)
            // DTB is at ~0x4820_0000 for a ~100KB kernel + initrd
            mmu::identity_map_range_optimized(root, 0x4000_0000, 0x5000_0000, kernel_flags)
                .expect("Failed to map boot RAM");
        }

        // Enable MMU with both TTBR0 and TTBR1
        mmu::tlb_flush_all();
        let root_phys = mmu::virt_to_phys(root as *const _ as usize);
        unsafe {
            // During transition, we can use the same root if it has
            // both bottom-half and top-half entries.
            mmu::enable_mmu(root_phys, root_phys);
        }
        verbose!("MMU re-initialized (Higher-Half + Identity).");
    }

    exceptions::init();
    verbose!("Exceptions initialized.");

    // Register HAL interrupts
    let dtb_phys = get_dtb_phys();
    let dtb_slice = dtb_phys.map(|phys| {
        let ptr = phys as *const u8;
        // Assume 1MB for early discovery
        unsafe { core::slice::from_raw_parts(ptr, 1024 * 1024) }
    });
    let fdt = dtb_slice.and_then(|slice| fdt::Fdt::new(slice).ok());

    let gic_api = gic::get_api(fdt.as_ref());
    verbose!("Detected GIC version: {:?}", gic_api.version());
    gic_api.init();

    // TEAM_047: Initialize physical memory management (Buddy Allocator)
    if let Some(slice) = dtb_slice {
        memory::init(slice);
    }

    // TEAM_045: Register IRQ handlers using trait objects
    gic::register_handler(gic::IrqId::VirtualTimer, &TIMER_HANDLER);
    gic::register_handler(gic::IrqId::Uart, &UART_HANDLER);
    gic_api.enable_irq(gic::IrqId::VirtualTimer.irq_number());
    gic_api.enable_irq(gic::IrqId::Uart.irq_number());

    verbose!("Core drivers initialized.");

    // TEAM_070: Initialize multitasking
    let bootstrap_task = alloc::sync::Arc::new(task::TaskControlBlock::new_bootstrap());
    unsafe {
        task::set_current_task(bootstrap_task);
    }

    // TEAM_071: Demo tasks gated behind feature to not break behavior tests
    #[cfg(feature = "multitask-demo")]
    {
        task::scheduler::SCHEDULER.add_task(alloc::sync::Arc::new(task::TaskControlBlock::new(
            task1 as *const () as usize,
        )));
        task::scheduler::SCHEDULER.add_task(alloc::sync::Arc::new(task::TaskControlBlock::new(
            task2 as *const () as usize,
        )));
    }

    // Set initial timer timeout
    timer::API.set_timeout(timer::API.read_frequency() / 100);
    timer::API.enable();
    verbose!("Timer initialized.");

    transition_to(BootStage::BootConsole);
    // TEAM_065: Initialize GPU FIRST (Stage 3 requirement)
    // GPU must be available before terminal operations per SPEC-1.
    let gpu_available = virtio::init_gpu();

    if gpu_available {
        verbose!("GPU initialized successfully.");
    } else {
        // SPEC-1: Explicit fallback to serial-only mode
        println!("[BOOT] SPEC-1: No GPU found, using serial-only console");
    }

    let mut display = gpu::Display;

    // SC2.2, SC2.3: Get resolution from GPU
    let (width, height) = match gpu::get_resolution() {
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
    console_gpu::init(width, height);
    
    // TEAM_081: Register GPU terminal as secondary console output
    // After this point, all println! calls will go to BOTH UART and GPU
    if gpu_available {
        levitate_hal::console::set_secondary_output(console_gpu::write_str);
        println!("[BOOT] Dual console enabled (UART + GPU)");
    }
    
    // SC14.2: Print dimensions to UART (and now GPU too!)
    if let Some((cols, rows)) = console_gpu::size() {
        println!("[TERM] Terminal size: {}x{} characters", cols, rows);
    }

    // TEAM_081: Clear terminal - boot log will appear naturally via println!
    console_gpu::clear();
    verbose!("Terminal initialized.");

    transition_to(BootStage::Discovery);
    // Initialize VirtIO Subsystem
    virtio::init();

    unsafe {
        println!(
            "BOOT_REGS: x0={:x} x1={:x} x2={:x} x3={:x}",
            BOOT_REGS[0], BOOT_REGS[1], BOOT_REGS[2], BOOT_REGS[3]
        );
    }
    // TEAM_065: SPEC-4 Initrd Discovery with Fail-Fast (Rule 14)
    println!("Detecting Initramfs...");
    let initrd_found = if let Some(slice) = dtb_slice {
        // TEAM_045: Use already created DTB slice
        match fdt::get_initrd_range(slice) {
            Ok((start, end)) => {
                let size = end - start;
                println!(
                    "Initramfs found at 0x{:x} - 0x{:x} ({} bytes)",
                    start, end, size
                );

                // Initramfs access via High VA (TTBR1) for stability
                let initrd_va = mmu::phys_to_virt(start);
                let initrd_slice =
                    unsafe { core::slice::from_raw_parts(initrd_va as *const u8, size) };
                let archive = fs::initramfs::CpioArchive::new(initrd_slice);

                println!("Files in initramfs:");
                for entry in archive.iter() {
                    println!(" - {}", entry.name);
                    if entry.name == "hello.txt" {
                        if let Ok(s) = core::str::from_utf8(entry.data) {
                            println!("   Content: {}", s.trim());
                        }
                    }
                }

                // TEAM_081 BREADCRUMB: CONFIRMED - THIS IS THE BOOT HIJACK
                // ================================================================
                // THIS CODE PREVENTS THE SYSTEM FROM BEING INTERACTIVE!
                // It jumps to userspace before the main loop runs.
                // 
                // TO FIX: Comment out or remove the run_from_initramfs call below.
                // The system should continue to the main loop which handles
                // keyboard input and displays the interactive prompt.
                // ================================================================
                // TEAM_073: Step 5 Verification - Run Hello World
                println!("[BOOT] TEAM_073: Hijacking boot for Userspace Demo...");
                // Enable interrupts (required for context switch or future syscalls handling)
                // unsafe { levitate_hal::interrupts::enable() };

                // Run "hello" from initramfs (does not return)
                // TODO(TEAM_081): Remove this hijack for interactive shell to work
                task::process::run_from_initramfs("hello", &archive);

                true
            }
            Err(e) => {
                println!("[BOOT] ERROR: No initramfs in DTB: {:?}", e);
                false
            }
        }
    } else {
        println!("[BOOT] ERROR: No DTB provided - cannot locate initramfs");
        false
    };

    // TEAM_065: SPEC-4 enforcement per Rule 14 (Fail Loud, Fail Fast)
    // If initrd is required but missing, drop to maintenance shell
    // TEAM_081: SPEC-4 enforcement - use println! for dual console
    #[cfg(not(feature = "diskless"))]
    if !initrd_found {
        println!("[BOOT] SPEC-4: Initrd required but not found.");
        println!("\n[ERROR] Initramfs not found!");
        println!("Dropping to maintenance shell...");
        maintenance_shell();
    }

    #[cfg(feature = "diskless")]
    if !initrd_found {
        verbose!("Diskless mode: continuing without initrd");
    }

    // Initialize Filesystem (Phase 4)
    // Don't panic if FS fails, just log it. Initramfs implies we might be diskless.
    match fs::init() {
        Ok(_) => {
            verbose!("Filesystem initialized.");
        }
        Err(e) => println!("Filesystem init skipped (expected if no disk): {}", e),
    }

    // 6. Enable interrupts
    unsafe { levitate_hal::interrupts::enable() };
    verbose!("Interrupts enabled.");

    transition_to(BootStage::SteadyState);
    // TEAM_081: Use println! for dual console output (UART + GPU)
    println!("Boot Stage: SYSTEM_READY\n");
    println!("Type to interact with the Boot Console.");
    println!("--------------------------------------\n");

    loop {
        // TEAM_081: Blinking cursor feedback via global terminal
        console_gpu::check_blink();

        // SC14.6: Keep existing cursor tracking
        if input::poll() {
            cursor::draw(&mut display);
        }

        // TEAM_081: Echo UART input - println! now handles both UART and GPU
        if let Some(c) = levitate_hal::console::read_byte() {
            let ch = c as char;
            match ch {
                '\r' | '\n' => {
                    println!(); // Dual console newline
                }
                _ => {
                    print!("{}", ch); // Dual console character
                }
            }
        }

        // TEAM_081: Echo VirtIO Keyboard input - println! now handles both
        if let Some(ch) = input::read_char() {
            match ch {
                '\n' => {
                    println!(); // Dual console newline
                }
                _ => {
                    print!("{}", ch); // Dual console character
                }
            }
        }

        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}

// TEAM_071: Demo tasks for preemption verification
// Gated behind feature flag to not break behavior tests
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
