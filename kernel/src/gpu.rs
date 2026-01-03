use crate::virtio::VirtioHal;
// No imports needed
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use levitate_utils::Spinlock;
use virtio_drivers::{device::gpu::VirtIOGpu, transport::mmio::MmioTransport};

pub struct GpuState {
    gpu: VirtIOGpu<VirtioHal, MmioTransport>,
    fb_ptr: usize,
    fb_len: usize,
    width: u32,
    height: u32,
}

pub static GPU: Spinlock<Option<GpuState>> = Spinlock::new(None);

pub fn init(transport: MmioTransport) {
    match VirtIOGpu::<VirtioHal, MmioTransport>::new(transport) {
        Ok(mut gpu) => {
            match gpu.resolution() {
                Ok((width, height)) => {
                    // Setup framebuffer
                    match gpu.setup_framebuffer() {
                        Ok(fb) => {
                            let fb_ptr = fb.as_mut_ptr() as usize;
                            let fb_len = fb.len();

                            *GPU.lock() = Some(GpuState {
                                gpu,
                                fb_ptr,
                                fb_len,
                                width,
                                height,
                            });
                        }
                        Err(_e) => {
                            crate::println!("GPU: Failed to setup framebuffer");  // Error always prints
                        }
                    }
                }
                Err(_e) => crate::println!("GPU: Failed to get resolution"),  // Error always prints
            }
        }
        Err(_e) => crate::println!("GPU: VirtIOGpu::new failed"),  // Error always prints
    }
}

impl GpuState {
    pub fn framebuffer(&mut self) -> &mut [u8] {
        // SAFETY: The framebuffer memory is allocated via DMA and kept alive by the VirtIOGpu instance
        // which lives as long as this GpuState. The pointer and length were obtained validity from setup_framebuffer.
        unsafe { core::slice::from_raw_parts_mut(self.fb_ptr as *mut u8, self.fb_len) }
    }

    /// TEAM_030: Get screen dimensions for input coordinate scaling
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct Display;

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            let width = state.width;
            let height = state.height;
            let fb = state.framebuffer();

            for Pixel(point, color) in pixels {
                if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32
                {
                    let idx = (point.y as usize * width as usize + point.x as usize) * 4;
                    if idx + 3 < fb.len() {
                        fb[idx] = color.r();
                        fb[idx + 1] = color.g();
                        fb[idx + 2] = color.b();
                        fb[idx + 3] = 255; // Alpha
                    }
                }
            }
            // Flush
            state.gpu.flush().ok();
        }
        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        let guard = GPU.lock();
        if let Some(state) = guard.as_ref() {
            Size::new(state.width, state.height)
        } else {
            Size::new(0, 0)
        }
    }
}
