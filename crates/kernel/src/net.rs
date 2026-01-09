//! VirtIO Network Device Driver
//!
//! TEAM_057: VirtIO Net implementation for Phase 6
//!
//! ## Behaviors
//! - [NET1] init() detects and initializes virtio-net device
//! - [NET2] init() reads MAC address from device config
//! - [NET3] mac_address() returns device MAC when initialized
//! - [NET4] mac_address() returns None when not initialized
//! - [NET5] can_send() returns true when TX queue has space
//! - [NET6] can_send() returns false when not initialized
//! - [NET7] can_recv() returns true when RX packet available
//! - [NET8] can_recv() returns false when not initialized
//! - [NET9] send() transmits packet when device ready
//! - [NET10] send() returns NotInitialized when device missing
//! - [NET11] send() returns DeviceBusy when queue full
//! - [NET12] receive() returns packet data when available
//! - [NET13] receive() returns None when no packet
//! - [NET14] receive() recycles RX buffer after read

extern crate alloc;

use crate::virtio::{StaticMmioTransport, VirtioHal};
use los_utils::Mutex;
use virtio_drivers::device::net::VirtIONet;

const QUEUE_SIZE: usize = 16;
const RX_BUFFER_LEN: usize = 2048;

static NET_DEVICE: Mutex<Option<VirtIONet<VirtioHal, StaticMmioTransport, QUEUE_SIZE>>> =
    Mutex::new(None);

use los_error::define_kernel_error;

define_kernel_error! {
    /// Network driver error types
    /// TEAM_152: Added error codes (0x07xx) per unified error system plan.
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum NetError(0x07) {
        /// Device not initialized
        NotInitialized = 0x01 => "Network device not initialized",
        /// TX queue is full
        DeviceBusy = 0x02 => "TX queue full",
        /// Transmission failed
        SendFailed = 0x03 => "Transmission failed",
    }
}

/// [NET1] Initialize network device from VirtIO transport
/// [NET2] Reads MAC address from device configuration
pub fn init(transport: StaticMmioTransport) {
    crate::verbose!("Initializing Network device...");
    match VirtIONet::<VirtioHal, StaticMmioTransport, QUEUE_SIZE>::new(transport, RX_BUFFER_LEN) {
        Ok(net) => {
            // [NET2] Read MAC address from device config
            #[allow(unused_variables)]
            let mac = net.mac_address();
            log::info!(
                "VirtIO Net: MAC={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0],
                mac[1],
                mac[2],
                mac[3],
                mac[4],
                mac[5]
            );
            // [NET1] Store initialized device
            *NET_DEVICE.lock() = Some(net);
        }
        Err(e) => log::error!("Failed to init VirtIO Net: {:?}", e),
    }
}
