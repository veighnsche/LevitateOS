use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};
use virtio_drivers::{device::gpu::VirtIOGpu, transport::mmio::MmioTransport};
use crate::sync::Spinlock;
use crate::virtio::VirtioHal;

struct GpuState {
    gpu: VirtIOGpu<VirtioHal, MmioTransport>,
    fb_ptr: usize,
    fb_len: usize,
    width: u32,
    height: u32,
}

static GPU: Spinlock<Option<GpuState>> = Spinlock::new(None);

fn puts(s: &str) {
    for c in s.bytes() {
        unsafe { core::ptr::write_volatile(0x09000000 as *mut u8, c); }
    }
    unsafe { core::ptr::write_volatile(0x09000000 as *mut u8, b'\n'); }
}

pub fn init(transport: MmioTransport) {
    puts("gpu::init entry");
    match VirtIOGpu::<VirtioHal, MmioTransport>::new(transport) {
        Ok(mut gpu) => {
            puts("VirtIOGpu::new success");
            match gpu.resolution() {
                Ok((width, height)) => {
                    puts("Resolution ok");
                    // Setup framebuffer
                    // Note: This relies on virtio-drivers keeping the allocation alive
                    // even after the returned slice is dropped.
                    match gpu.setup_framebuffer() {
                        Ok(fb) => {
                           let fb_ptr = fb.as_mut_ptr() as usize;
                           let fb_len = fb.len();
                           puts("Framebuffer setup ok");
                           
                           *GPU.lock() = Some(GpuState {
                               gpu,
                               fb_ptr,
                               fb_len,
                               width,
                               height,
                           });
                           puts("GPU stored with FB");
                        }
                        Err(e) => {
                             puts("Failed to setup framebuffer");
                        }
                    }
                }
                Err(e) => puts("Failed to get resolution"),
            }
        }
        Err(e) => puts("VirtIOGpu::new failed"),
    }
    puts("gpu::init exit");
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
             // Reconstitute framebuffer slice
             let fb = unsafe { core::slice::from_raw_parts_mut(state.fb_ptr as *mut u8, state.fb_len) };
             
             for Pixel(point, color) in pixels {
                 if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
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
