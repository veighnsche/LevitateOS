use alloc::vec::Vec;
use virtio_drivers::{device::input::VirtIOInput, transport::mmio::MmioTransport};
use crate::virtio::VirtioHal;
use crate::sync::Spinlock;

static INPUT_DEVICES: Spinlock<Vec<VirtIOInput<VirtioHal, MmioTransport>>> = Spinlock::new(Vec::new());

pub fn init(transport: MmioTransport) {
    crate::println!("Initializing VirtIO Input device...");
    match VirtIOInput::<VirtioHal, MmioTransport>::new(transport) {
        Ok(input) => {
            crate::println!("VirtIO Input initialized successfully.");
            INPUT_DEVICES.lock().push(input);
        }
        Err(e) => crate::println!("Failed to init VirtIO Input: {:?}", e),
    }
}

pub fn poll() -> bool {
    let mut dirty = false;
    let mut devices = INPUT_DEVICES.lock();
    for input in devices.iter_mut() {
        if let Some(event) = input.pop_pending_event() {
             match event.event_type {
                 3 => { // EV_ABS
                     match event.code {
                         0 => { // ABS_X
                             let x = (event.value as i32 * 1024) / 32768;
                             crate::cursor::set_x(x);
                             dirty = true;
                         }
                         1 => { // ABS_Y
                             let y = (event.value as i32 * 768) / 32768;
                             crate::cursor::set_y(y);
                             dirty = true;
                         }
                         _ => {}
                     }
                 }
                 _ => {}
             }
        }
        input.ack_interrupt();
    }
    dirty
}
