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

use alloc::vec::Vec;
use crate::virtio::{StaticMmioTransport, VirtioHal};
use levitate_utils::Spinlock;
use virtio_drivers::device::net::VirtIONet;

const QUEUE_SIZE: usize = 16;
const RX_BUFFER_LEN: usize = 2048;

static NET_DEVICE: Spinlock<Option<VirtIONet<VirtioHal, StaticMmioTransport, QUEUE_SIZE>>> =
    Spinlock::new(None);

/// Network driver error types
#[derive(Debug)]
pub enum NetError {
    /// Device not initialized
    NotInitialized,
    /// TX queue is full
    DeviceBusy,
    /// Transmission failed
    SendFailed,
}

/// [NET1] Initialize network device from VirtIO transport
/// [NET2] Reads MAC address from device configuration
pub fn init(transport: StaticMmioTransport) {
    crate::verbose!("Initializing Network device...");
    match VirtIONet::<VirtioHal, StaticMmioTransport, QUEUE_SIZE>::new(transport, RX_BUFFER_LEN) {
        Ok(net) => {
            // [NET2] Read MAC address from device config
            let mac = net.mac_address();
            crate::verbose!(
                "VirtIO Net: MAC={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );
            // [NET1] Store initialized device
            *NET_DEVICE.lock() = Some(net);
        }
        Err(e) => crate::println!("Failed to init VirtIO Net: {:?}", e),
    }
}

/// [NET3] Returns MAC address when initialized
/// [NET4] Returns None when not initialized
pub fn mac_address() -> Option<[u8; 6]> {
    NET_DEVICE.lock().as_ref().map(|net| net.mac_address()) // [NET3], [NET4]
}

/// [NET5] Returns true when TX queue has space
/// [NET6] Returns false when not initialized
pub fn can_send() -> bool {
    NET_DEVICE
        .lock()
        .as_ref()
        .map_or(false, |net| net.can_send()) // [NET5], [NET6]
}

/// [NET7] Returns true when RX packet available
/// [NET8] Returns false when not initialized
pub fn can_recv() -> bool {
    NET_DEVICE
        .lock()
        .as_ref()
        .map_or(false, |net| net.can_recv()) // [NET7], [NET8]
}

/// [NET9] Transmits packet when device ready
/// [NET10] Returns NotInitialized when device missing
/// [NET11] Returns DeviceBusy when queue full
pub fn send(data: &[u8]) -> Result<(), NetError> {
    let mut dev = NET_DEVICE.lock();
    let net = dev.as_mut().ok_or(NetError::NotInitialized)?; // [NET10]

    if !net.can_send() {
        return Err(NetError::DeviceBusy); // [NET11]
    }

    // [NET9] Create TX buffer and send
    let mut tx_buf = net.new_tx_buffer(data.len());
    tx_buf.packet_mut().copy_from_slice(data);

    net.send(tx_buf).map_err(|_| NetError::SendFailed)
}

/// [NET12] Returns packet data when available
/// [NET13] Returns None when no packet
/// [NET14] Recycles RX buffer after read
pub fn receive() -> Option<Vec<u8>> {
    let mut dev = NET_DEVICE.lock();
    let net = dev.as_mut()?; // [NET13] implicit None if not initialized

    if !net.can_recv() {
        return None; // [NET13]
    }

    match net.receive() {
        Ok(rx_buf) => {
            // [NET12] Copy packet data
            let data = rx_buf.packet().to_vec();
            // [NET14] Recycle buffer for reuse
            let _ = net.recycle_rx_buffer(rx_buf);
            Some(data)
        }
        Err(_) => None,
    }
}
