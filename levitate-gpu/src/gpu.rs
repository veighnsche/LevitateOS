//! VirtIO GPU Library
//! TEAM_092: Extracted from kernel/src/gpu.rs

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use levitate_hal::{StaticMmioTransport, VirtioHal};
use virtio_drivers::device::gpu::VirtIOGpu;

#[derive(Debug, Clone, Copy)]
pub enum GpuError {
    NotInitialized,
    FlushFailed,
}

pub struct GpuState {
    pub gpu: VirtIOGpu<VirtioHal, StaticMmioTransport>,
    fb_ptr: usize,
    fb_len: usize,
    pub width: u32,
    pub height: u32,
    // Telemetry hooks
    pub total_flushes: u64,
    pub failed_flushes: u64,
}

impl GpuState {
    pub fn new(transport: StaticMmioTransport) -> Result<Self, &'static str> {
        let mut gpu = VirtIOGpu::<VirtioHal, StaticMmioTransport>::new(transport)
            .map_err(|_| "VirtIOGpu::new failed")?;

        let (width, height) = gpu.resolution().map_err(|_| "Failed to get resolution")?;

        let mut state = Self {
            gpu,
            fb_ptr: 0,
            fb_len: 0,
            width,
            height,
            total_flushes: 0,
            failed_flushes: 0,
        };

        // SETUP FB after move
        let fb = state
            .gpu
            .setup_framebuffer()
            .map_err(|_| "Failed to setup framebuffer")?;
        state.fb_ptr = fb.as_mut_ptr() as usize;
        state.fb_len = fb.len();

        Ok(state)
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.fb_ptr as *mut u8, self.fb_len) }
    }

    pub fn flush(&mut self) {
        self.total_flushes += 1;
        if let Err(_e) = self.gpu.flush() {
            self.failed_flushes += 1;
        }
    }

    pub fn heartbeat(&self) {
        use core::fmt::Write;
        if let Some(mut uart) = levitate_hal::console::WRITER.try_lock() {
            let _ = writeln!(
                uart,
                "[GPU-HB] {}x{} | flushes: {} | errors: {}",
                self.width, self.height, self.total_flushes, self.failed_flushes
            );
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct Display<'a> {
    pub state: &'a mut GpuState,
}

impl<'a> Display<'a> {
    pub fn new(state: &'a mut GpuState) -> Self {
        Self { state }
    }
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.state.width;
        let height = self.state.height;
        let fb = self.state.framebuffer();

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
        Ok(())
    }
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        Size::new(self.state.width, self.state.height)
    }
}
