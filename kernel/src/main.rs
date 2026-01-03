#![no_std]
#![no_main]

extern crate alloc;

// use core::alloc::Layout;
use core::arch::global_asm;
use core::panic::PanicInfo;

mod block;
mod cursor;
mod exceptions;
mod gpu;
mod input;
mod virtio;

use levitate_hal::gic;
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
    .quad   0x0
    .quad   _end - _head
    .quad   0x0A
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\x64"
    .long   0

_start:
    msr     daifset, #0xf
    mrs     x0, mpidr_el1
    and     x0, x0, #0xFF
    cbz     x0, primary_cpu

secondary_halt:
    wfe
    b       secondary_halt

primary_cpu:
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

    /* L1_high[1] -> 0x40000000 (1GB Block matching 0x40000000 physical) */
    ldr     x0, =boot_pt_l1_high
    sub     x0, x0, x4
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

// TEAM_015: IRQ handler functions registered via gic::register_handler
fn timer_irq_handler() {
    // Reload timer for next interrupt (1 second)
    let freq = timer::API.read_frequency();
    timer::API.set_timeout(freq);
}

fn uart_irq_handler() {
    levitate_hal::console::handle_interrupt();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // 1. Initialize heap
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

    println!("\n*** ClaudeOS Rust Kernel ***");
    println!("Heap initialized.");

    // 1.5 Initialize MMU (TEAM_020)
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

        // Map Devices (Identity mapped for now)
        mmu::identity_map_range_optimized(root, 0x0900_0000, 0x0900_1000, mmu::PageFlags::DEVICE)
            .unwrap(); // UART
        mmu::identity_map_range_optimized(root, 0x0800_0000, 0x0802_0000, mmu::PageFlags::DEVICE)
            .unwrap(); // GIC
        mmu::identity_map_range_optimized(root, 0x0a00_0000, 0x0a10_0000, mmu::PageFlags::DEVICE)
            .unwrap(); // VirtIO

        // Enable MMU with both TTBR0 and TTBR1
        mmu::tlb_flush_all();
        let root_phys = mmu::virt_to_phys(root as *const _ as usize);
        unsafe {
            // During transition, we can use the same root if it has
            // both bottom-half and top-half entries.
            mmu::enable_mmu(root_phys, root_phys);
        }
        println!("MMU re-initialized (Higher-Half + Identity).");
    }

    // 2. Initialize Core Drivers
    exceptions::init();
    println!("Exceptions initialized.");
    levitate_hal::console::init();
    gic::API.init();

    // TEAM_015: Register IRQ handlers using typed IrqId
    gic::register_handler(gic::IrqId::VirtualTimer, timer_irq_handler);
    gic::register_handler(gic::IrqId::Uart, uart_irq_handler);
    gic::API.enable_irq(gic::IrqId::VirtualTimer.irq_number());
    gic::API.enable_irq(gic::IrqId::Uart.irq_number());

    println!("Core drivers initialized.");

    // 3. Initialize Timer (Phase 2)
    println!("Initializing Timer...");
    let freq = timer::API.read_frequency();
    print!("Timer frequency (hex): ");
    levitate_hal::console::print_hex(freq);
    println!("");
    timer::API.set_timeout(freq);
    timer::API.enable();
    println!("Timer initialized.");

    // 4. Initialize VirtIO (Phase 3)
    virtio::init();

    // 5. Enable interrupts
    unsafe {
        core::arch::asm!("msr daifclr, #2");
    }
    println!("Interrupts enabled.");

    // Verify Graphics
    use embedded_graphics::{
        pixelcolor::Rgb888,
        prelude::*,
        primitives::{PrimitiveStyle, Rectangle},
    };

    println!("Drawing test pattern...");
    let mut display = gpu::Display;
    if display.size().width > 0 {
        // Draw blue background
        let _ = Rectangle::new(Point::new(0, 0), display.size())
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(&mut display);

        // Draw red rectangle
        let _ = Rectangle::new(Point::new(100, 100), Size::new(200, 200))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(255, 0, 0)))
            .draw(&mut display);
        println!("Drawing complete.");
    } else {
        println!("Display not ready.");
    }

    loop {
        if input::poll() {
            cursor::draw(&mut display);
        }

        // Echo UART input
        if let Some(c) = levitate_hal::console::read_byte() {
            print!("{}", c as char);
        }

        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}
