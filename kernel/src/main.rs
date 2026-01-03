#![no_std]
#![no_main]

extern crate alloc;

// use core::alloc::Layout;
use core::arch::global_asm;
use core::panic::PanicInfo;

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

.section ".text.boot", "ax"
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

    ldr     x0, =0x48000000
    mov     sp, x0

    ldr     x0, =__bss_start
    ldr     x1, =__bss_end
    mov     x2, #0
bss_loop:
    cmp     x0, x1
    b.ge    bss_done
    str     x2, [x0], #8
    b       bss_loop
bss_done:

    bl      kmain

halt:
    wfe
    b       halt

.section ".data"
.global _end
_end:
"#
);

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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

    // 2. Initialize Core Drivers
    exceptions::init();
    gic::API.init();
    gic::API.enable_irq(27); // Virtual Timer
    gic::API.enable_irq(33); // UART

    println!("Core drivers initialized.");
    levitate_hal::console::init();

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
