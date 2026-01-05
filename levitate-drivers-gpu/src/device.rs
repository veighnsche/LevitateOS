//! VirtIO GPU Device - Integrated driver with transport
//!
//! TEAM_099: Connects GpuDriver to VirtQueue and MmioTransport.
//!
//! This module provides the complete GPU device that:
//! - Uses MmioTransport for device communication
//! - Uses VirtQueue for command/response handling
//! - Uses GpuDriver for protocol state machine

extern crate alloc;

use core::ptr::NonNull;

use levitate_virtio::{
    BufferDirection, MmioTransport, Transport, VirtQueue, VirtQueueError, VirtioHal,
    status, features,
};

use crate::driver::{DriverTelemetry, GpuDriver};
use crate::protocol::{CtrlHeader, Format, GpuError, RespDisplayInfo};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

/// VirtIO GPU queue indices
const CONTROLQ: u16 = 0;

/// Queue size (must be power of 2)
const QUEUE_SIZE: usize = 16;

/// VirtIO GPU device with full transport integration.
/// TEAM_106: VirtQueue allocated via HAL's dma_alloc for DMA-safe memory.
pub struct VirtioGpu<H: VirtioHal> {
    /// MMIO transport for device registers
    transport: MmioTransport,
    /// Control virtqueue pointer (DMA-allocated)
    /// TEAM_106: Changed from Box to raw pointer for DMA allocation
    control_queue_ptr: NonNull<VirtQueue<QUEUE_SIZE>>,
    /// Control queue physical address (for cleanup)
    control_queue_paddr: u64,
    /// Control queue allocation size in pages
    control_queue_pages: usize,
    /// Protocol state machine driver
    driver: GpuDriver,
    /// Framebuffer physical address
    fb_paddr: u64,
    /// Framebuffer virtual pointer
    fb_ptr: Option<NonNull<u8>>,
    /// Framebuffer size in bytes
    fb_size: usize,
    /// HAL marker
    _hal: core::marker::PhantomData<H>,
}

unsafe impl<H: VirtioHal> Send for VirtioGpu<H> {}
unsafe impl<H: VirtioHal> Sync for VirtioGpu<H> {}

impl<H: VirtioHal> DrawTarget for VirtioGpu<H> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let (width, height) = self.resolution();
        let fb_size = self.fb_size;
        if let Some(ptr) = self.fb_ptr {
            let fb = unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), fb_size) };
            for Pixel(point, color) in pixels {
                if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                    let idx = (point.y as usize * width as usize + point.x as usize) * 4;
                    if idx + 3 < fb.len() {
                        fb[idx] = color.b();
                        fb[idx + 1] = color.g();
                        fb[idx + 2] = color.r();
                        fb[idx + 3] = 255;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<H: VirtioHal> OriginDimensions for VirtioGpu<H> {
    fn size(&self) -> Size {
        let (width, height) = self.resolution();
        Size::new(width, height)
    }
}

impl<H: VirtioHal> VirtioGpu<H> {
    /// Create a new VirtIO GPU device.
    ///
    /// # Safety
    ///
    /// `mmio_base` must be a valid VirtIO MMIO address.
    pub unsafe fn new(mmio_base: usize) -> Result<Self, GpuError> {
        // Create transport
        let mut transport = unsafe { MmioTransport::new(mmio_base) }
            .map_err(|_| GpuError::TransportError)?;

        // Reset device
        transport.reset();

        // Set ACKNOWLEDGE status
        transport.write_status(status::ACKNOWLEDGE);

        // Set DRIVER status
        transport.write_status(status::ACKNOWLEDGE | status::DRIVER);

        // Negotiate features (we only need basic 2D support)
        let device_features = transport.read_device_features();
        let driver_features = device_features & features::VERSION_1;
        transport.write_driver_features(driver_features);

        // Set FEATURES_OK
        transport.write_status(status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK);

        // Verify FEATURES_OK is still set
        if transport.read_status() & status::FEATURES_OK == 0 {
            return Err(GpuError::TransportError);
        }

        // TEAM_106: Allocate control queue via HAL's dma_alloc for DMA-safe memory
        // This ensures 16-byte alignment and DMA-accessible memory per VirtIO spec.
        let queue_size = core::mem::size_of::<VirtQueue<QUEUE_SIZE>>();
        let control_queue_pages = levitate_virtio::pages_for(queue_size);
        let (control_queue_paddr, queue_vptr) = H::dma_alloc(control_queue_pages, BufferDirection::Both);

        // Initialize queue in DMA memory
        let control_queue_ptr = unsafe {
            let ptr = queue_vptr.as_ptr() as *mut VirtQueue<QUEUE_SIZE>;
            ptr.write(VirtQueue::new());
            NonNull::new_unchecked(ptr)
        };

        // Initialize the free list
        unsafe { control_queue_ptr.as_ptr().as_mut().unwrap().init() };

        let max_queue_size = transport.max_queue_size(CONTROLQ);
        if max_queue_size == 0 {
            return Err(GpuError::TransportError);
        }

        // TEAM_106: Get addresses from DMA-allocated queue
        let (desc_vaddr, avail_vaddr, used_vaddr) = unsafe { control_queue_ptr.as_ref() }.addresses();
        let desc_paddr = H::virt_to_phys(desc_vaddr);
        let avail_paddr = H::virt_to_phys(avail_vaddr);
        let used_paddr = H::virt_to_phys(used_vaddr);
        
        transport.queue_set(
            CONTROLQ,
            QUEUE_SIZE as u16,
            desc_paddr,
            avail_paddr,
            used_paddr,
        );

        // Set DRIVER_OK
        transport.write_status(
            status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK | status::DRIVER_OK,
        );

        let driver = GpuDriver::new();

        Ok(Self {
            transport,
            control_queue_ptr,
            control_queue_paddr,
            control_queue_pages,
            driver,
            fb_paddr: 0,
            fb_ptr: None,
            fb_size: 0,
            _hal: core::marker::PhantomData,
        })
    }

    /// TEAM_106: Helper to access control queue safely
    fn control_queue(&mut self) -> &mut VirtQueue<QUEUE_SIZE> {
        // SAFETY: control_queue_ptr is valid for the lifetime of VirtioGpu
        unsafe { self.control_queue_ptr.as_mut() }
    }

    /// Initialize the GPU: get display info, create resource, attach backing, set scanout.
    pub fn init(&mut self) -> Result<(), GpuError> {
        // Step 1: GET_DISPLAY_INFO
        let resp = {
            let cmd = self.driver.build_get_display_info();
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, core::mem::size_of::<RespDisplayInfo>())?
        };
        self.driver.handle_display_info_response(&resp)?;

        let (width, height) = self.driver.resolution();
        if width == 0 || height == 0 {
            return Err(GpuError::InvalidParameter);
        }

        // Step 2: Allocate framebuffer
        let fb_size = Format::default().buffer_size(width, height);
        let pages = levitate_virtio::pages_for(fb_size);
        let (fb_paddr, fb_ptr) = H::dma_alloc(pages, BufferDirection::DriverToDevice);
        self.fb_paddr = fb_paddr;
        self.fb_ptr = Some(fb_ptr);
        self.fb_size = fb_size;

        // Step 3: RESOURCE_CREATE_2D
        let resp = {
            let cmd = self.driver.build_resource_create_2d();
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, CtrlHeader::SIZE)?
        };
        self.driver.handle_resource_create_response(&resp)?;

        // Step 4: RESOURCE_ATTACH_BACKING
        let resp = {
            let cmd = self.driver.build_attach_backing(fb_paddr, fb_size as u32);
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, CtrlHeader::SIZE)?
        };
        self.driver.handle_attach_backing_response(&resp)?;

        // Step 5: SET_SCANOUT
        let resp = {
            let cmd = self.driver.build_set_scanout();
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, CtrlHeader::SIZE)?
        };
        self.driver.handle_set_scanout_response(&resp)?;

        Ok(())
    }

    /// Get the framebuffer as a mutable slice.
    pub fn framebuffer(&mut self) -> Option<&mut [u8]> {
        self.fb_ptr.map(|ptr| {
            // SAFETY: We allocated this memory and own it
            unsafe { core::slice::from_raw_parts_mut(ptr.as_ptr(), self.fb_size) }
        })
    }

    /// Get the display resolution.
    pub fn resolution(&self) -> (u32, u32) {
        self.driver.resolution()
    }

    /// Check if the driver is ready.
    pub fn is_ready(&self) -> bool {
        self.driver.is_ready()
    }

    /// Get driver telemetry.
    pub fn telemetry(&self) -> &DriverTelemetry {
        self.driver.telemetry()
    }

    /// Transfer framebuffer region to host and flush.
    pub fn flush(&mut self) -> Result<(), GpuError> {
        if !self.driver.is_ready() {
            return Err(GpuError::NotInitialized);
        }

        let rect = self.driver.framebuffer_rect();

        // TRANSFER_TO_HOST_2D
        let resp = {
            let cmd = self.driver.build_transfer_to_host(rect);
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, CtrlHeader::SIZE)?
        };
        self.driver.handle_ok_nodata_response(&resp)?;

        // RESOURCE_FLUSH
        let resp = {
            let cmd = self.driver.build_flush(rect);
            let cmd_copy = alloc::vec::Vec::from(cmd);
            self.send_command(&cmd_copy, CtrlHeader::SIZE)?
        };
        self.driver.handle_ok_nodata_response(&resp)?;

        Ok(())
    }

    /// Send a command and wait for response.
    fn send_command(&mut self, cmd: &[u8], resp_size: usize) -> Result<alloc::vec::Vec<u8>, GpuError> {
        extern crate alloc;
        use alloc::vec;

        // Allocate response buffer
        let mut resp = vec![0u8; resp_size];

        // TEAM_106: Add to virtqueue with physical address translation
        let _head = self.control_queue()
            .add_buffer(&[cmd], &mut [resp.as_mut_slice()], H::virt_to_phys)
            .map_err(|e| match e {
                VirtQueueError::QueueFull => GpuError::TransportError,
                _ => GpuError::TransportError,
            })?;

        // Notify device
        self.transport.queue_notify(CONTROLQ);

        // Poll for completion (busy wait)
        // In a real async implementation, we'd use interrupts
        let mut timeout = 1_000_000u32;
        while !self.control_queue().has_used() && timeout > 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }

        if timeout == 0 {
            return Err(GpuError::Timeout);
        }

        // Pop the used buffer
        let _ = self.control_queue().pop_used();

        Ok(resp)
    }
}

impl<H: VirtioHal> Drop for VirtioGpu<H> {
    fn drop(&mut self) {
        // Reset device first
        self.transport.reset();

        // TEAM_106: Deallocate DMA control queue memory
        unsafe {
            H::dma_dealloc(
                self.control_queue_paddr,
                self.control_queue_ptr.cast(),
                self.control_queue_pages,
            );
        }

        // Free framebuffer if allocated
        if let Some(ptr) = self.fb_ptr.take() {
            let pages = levitate_virtio::pages_for(self.fb_size);
            // SAFETY: We allocated this memory in init()
            unsafe { H::dma_dealloc(self.fb_paddr, ptr, pages) };
        }
    }
}
