#![no_std]
#![no_main]

extern crate alloc;

use core::arch::global_asm;
use core::panic::PanicInfo;
use core::fmt::{self, Write};
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

mod exceptions;
mod gic;
mod timer;
mod virtio;
mod gpu;
mod input;
mod cursor;
mod sync;

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

struct Spinlock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    fn lock(&self) -> SpinlockGuard<T> {
        while self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        SpinlockGuard { lock: &self.lock, data: unsafe { &mut *self.data.get() } }
    }
}

struct SpinlockGuard<'a, T> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<T> core::ops::Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T { self.data }
}

impl<T> core::ops::DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T { self.data }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

struct SimpleAllocator {
    heap: Spinlock<linked_list_allocator::Heap>,
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: Spinlock::new(linked_list_allocator::Heap::empty()),
};

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            self.heap.lock().allocate_first_fit(layout)
                .ok()
                .map_or(core::ptr::null_mut(), |ptr| ptr.as_ptr())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.heap.lock().deallocate(core::ptr::NonNull::new_unchecked(ptr), layout);
        }
    }
}

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let uart0 = 0x0900_0000 as *mut u8;
        for b in s.bytes() {
            unsafe {
                core::ptr::write_volatile(uart0, b);
            }
        }
        Ok(())
    }
}

pub fn print_hex(val: u64) {
    let mut uart = Uart;
    let _ = uart.write_str("0x");
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xf;
        let c = if nibble < 10 {
            (b'0' + nibble as u8) as char
        } else {
            (b'a' + (nibble - 10) as u8) as char
        };
        let _ = uart.write_str(core::str::from_utf8(&[c as u8]).unwrap());
    }
}

pub fn _print(args: fmt::Arguments) {
    Uart.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
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
        ALLOCATOR.heap.lock().init(heap_start as *mut u8, heap_size);
    }

    println!("\n*** ClaudeOS Rust Kernel ***");
    println!("Heap initialized.");

    // 2. Initialize Core Drivers (Phase 2)
    unsafe { gic::Gic::init(); }
    unsafe { gic::Gic::enable_irq(30); }

    exceptions::init();

    // timer::Timer::enable_interrupt(1);

    // 3. Initialize VirtIO (Phase 3)
    virtio::init();

    // Enable IRQs
    unsafe { core::arch::asm!("msr daifclr, #2"); }
    
    // Verify Graphics
    use embedded_graphics::{
        pixelcolor::Rgb888,
        prelude::*,
        primitives::{Rectangle, PrimitiveStyle},
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
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}
