//! Kernel-side GPU Interface
//!
//! TEAM_114: Wrapper around levitate-gpu crate for kernel use.
//! TEAM_141: Removed duplicate Display type - use los_gpu::Display via as_display()
//! TEAM_331: Added Limine framebuffer fallback for x86_64 when virtio-gpu unavailable
//! TEAM_336: Made generic over transport to support both PCI (x86_64) and MMIO (AArch64)
//!
//! See: `docs/planning/virtio-pci/` for the implementation plan

use los_hal::IrqSafeLock;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::virtio::VirtioHal;

// Re-export from levitate-gpu
pub use los_gpu::GpuError;

// TEAM_337: GPU uses PCI on BOTH architectures (QEMU uses virtio-gpu-pci for all)
type GpuTransport = los_pci::PciTransport;

// TEAM_336: Re-export Display with concrete transport type
pub type Display<'a> = los_gpu::Display<'a, VirtioHal, GpuTransport>;

/// TEAM_336: GPU backend type - either VirtIO or Limine framebuffer
enum GpuBackend {
    VirtIO(los_gpu::Gpu<VirtioHal, GpuTransport>),
    Framebuffer(FramebufferGpu),
}

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

/// TEAM_331: Simple framebuffer-based GPU for Limine boot
struct FramebufferGpu {
    address: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bpp: u8,
    format: crate::boot::PixelFormat,
}

impl FramebufferGpu {
    fn new(fb: &crate::boot::Framebuffer) -> Self {
        Self {
            address: fb.address,
            width: fb.width,
            height: fb.height,
            pitch: fb.pitch,
            bpp: fb.bpp,
            format: fb.format,
        }
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn framebuffer(&mut self) -> &mut [u8] {
        let size = (self.pitch as usize) * (self.height as usize);
        // SAFETY: Limine framebuffer address is valid and mapped by bootloader
        unsafe { core::slice::from_raw_parts_mut(self.address as *mut u8, size) }
    }

    fn flush(&mut self) -> Result<(), GpuError> {
        // Limine framebuffer is directly mapped - no flush needed
        Ok(())
    }
}

/// TEAM_331: DrawTarget wrapper for framebuffer GPU
pub struct FramebufferDisplay<'a> {
    gpu: &'a mut FramebufferGpu,
}

impl<'a> FramebufferDisplay<'a> {
    fn new(gpu: &'a mut FramebufferGpu) -> Self {
        Self { gpu }
    }
}

impl DrawTarget for FramebufferDisplay<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.gpu.width;
        let height = self.gpu.height;
        let pitch = self.gpu.pitch as usize;
        let is_bgr = matches!(self.gpu.format, crate::boot::PixelFormat::Bgr);
        let fb = self.gpu.framebuffer();

        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < width as i32 && point.y >= 0 && point.y < height as i32 {
                let offset = (point.y as usize) * pitch + (point.x as usize) * 4;
                if offset + 3 < fb.len() {
                    if is_bgr {
                        fb[offset] = color.b();
                        fb[offset + 1] = color.g();
                        fb[offset + 2] = color.r();
                    } else {
                        fb[offset] = color.r();
                        fb[offset + 1] = color.g();
                        fb[offset + 2] = color.b();
                    }
                    fb[offset + 3] = 255; // Alpha
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for FramebufferDisplay<'_> {
    fn size(&self) -> Size {
        Size::new(self.gpu.width, self.gpu.height)
    }
}

/// TEAM_331: Unified display wrapper for both VirtIO and Limine framebuffer
/// TEAM_336: Display now uses arch-specific transport via type alias
pub enum UnifiedDisplay<'a> {
    VirtIO(Display<'a>),
    Framebuffer(FramebufferDisplay<'a>),
}

impl DrawTarget for UnifiedDisplay<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        match self {
            UnifiedDisplay::VirtIO(d) => {
                // VirtIO display returns Infallible, so we can unwrap
                d.draw_iter(pixels).map_err(|_| unreachable!())
            }
            UnifiedDisplay::Framebuffer(d) => d.draw_iter(pixels),
        }
    }
}

impl OriginDimensions for UnifiedDisplay<'_> {
    fn size(&self) -> Size {
        match self {
            UnifiedDisplay::VirtIO(d) => d.size(),
            UnifiedDisplay::Framebuffer(d) => d.size(),
        }
    }
}

/// GPU state wrapper supporting both VirtIO and Limine framebuffer
pub struct GpuState {
    backend: GpuBackend,
}

// SAFETY: GPU access is protected by IrqSafeLock
unsafe impl Send for GpuState {}
unsafe impl Sync for GpuState {}

impl GpuState {
    pub fn flush(&mut self) -> Result<(), GpuError> {
        // TEAM_129: Increment flush counter for regression testing
        FLUSH_COUNT.fetch_add(1, Ordering::Relaxed);
        match &mut self.backend {
            GpuBackend::VirtIO(gpu) => gpu.flush(),
            GpuBackend::Framebuffer(fb) => fb.flush(),
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match &self.backend {
            GpuBackend::VirtIO(gpu) => gpu.resolution(),
            GpuBackend::Framebuffer(fb) => fb.resolution(),
        }
    }

    pub fn framebuffer(&mut self) -> &mut [u8] {
        match &mut self.backend {
            GpuBackend::VirtIO(gpu) => gpu.framebuffer(),
            GpuBackend::Framebuffer(fb) => fb.framebuffer(),
        }
    }

    /// TEAM_331: Get a unified display adapter for embedded-graphics DrawTarget
    /// Works for both VirtIO and Limine framebuffer backends
    pub fn as_display(&mut self) -> UnifiedDisplay<'_> {
        match &mut self.backend {
            GpuBackend::VirtIO(gpu) => UnifiedDisplay::VirtIO(Display::new(gpu)),
            GpuBackend::Framebuffer(fb) => UnifiedDisplay::Framebuffer(FramebufferDisplay::new(fb)),
        }
    }
}

// TEAM_122: Use IrqSafeLock to prevent deadlocks between input::poll and ISR prints
pub static GPU: IrqSafeLock<Option<GpuState>> = IrqSafeLock::new(None);

// TEAM_129: Flush counter for regression testing - ensures GPU flush is actually called
static FLUSH_COUNT: AtomicU32 = AtomicU32::new(0);

/// Get the number of times GPU flush has been called (for testing)
pub fn flush_count() -> u32 {
    FLUSH_COUNT.load(Ordering::Relaxed)
}

/// TEAM_129: Check if framebuffer contains any non-black pixels (for regression testing)
/// Returns (total_pixels, non_black_count) to verify terminal actually rendered content
pub fn framebuffer_has_content() -> Option<(usize, usize)> {
    let mut guard = GPU.lock();
    if let Some(gpu_state) = guard.as_mut() {
        let fb = gpu_state.framebuffer();
        let total_pixels = fb.len() / 4; // BGRA format
        let mut non_black = 0usize;
        
        // Sample every 100th pixel for performance (still catches any rendering)
        for i in (0..fb.len()).step_by(400) { // 400 = 100 pixels * 4 bytes
            // Check if R, G, or B is non-zero (index+2=R, index+1=G, index=B)
            if i + 2 < fb.len() && (fb[i] != 0 || fb[i + 1] != 0 || fb[i + 2] != 0) {
                non_black += 1;
            }
        }
        Some((total_pixels, non_black * 100)) // Scale back up
    } else {
        None
    }
}

/// TEAM_320: Print GPU display status to serial console for host debugging
/// Outputs a clear indicator whether the screen is black or has content
pub fn debug_display_status() {
    use los_hal::serial_println;
    
    let guard = GPU.lock();
    if guard.is_none() {
        serial_println!("╔══════════════════════════════════════════════════════════╗");
        serial_println!("║  [GPU DEBUG] STATUS: NO GPU INITIALIZED                  ║");
        serial_println!("╚══════════════════════════════════════════════════════════╝");
        return;
    }
    drop(guard);
    
    match framebuffer_has_content() {
        Some((total, non_black)) => {
            let percentage = if total > 0 { (non_black * 100) / total } else { 0 };
            serial_println!("╔══════════════════════════════════════════════════════════╗");
            if non_black == 0 {
                serial_println!("║  [GPU DEBUG] DISPLAY STATUS: BLACK SCREEN ❌             ║");
                serial_println!("║  Framebuffer is entirely black - no content rendered     ║");
            } else {
                serial_println!("║  [GPU DEBUG] DISPLAY STATUS: HAS CONTENT ✅              ║");
                serial_println!("║  Non-black pixels: {} (~{}% of screen)            ", non_black, percentage);
            }
            serial_println!("║  Flush count: {}                                          ", flush_count());
            serial_println!("╚══════════════════════════════════════════════════════════╝");
        }
        None => {
            serial_println!("╔══════════════════════════════════════════════════════════╗");
            serial_println!("║  [GPU DEBUG] STATUS: FRAMEBUFFER NOT AVAILABLE           ║");
            serial_println!("╚══════════════════════════════════════════════════════════╝");
        }
    }
}

/// TEAM_336: Initialize GPU with transport (now accepts Option<GpuTransport>)
/// For x86_64: Pass PciTransport from PCI scan
/// For AArch64: Pass MmioTransport from MMIO scan
pub fn init(transport: Option<GpuTransport>) {
    log::info!("[GPU] Initializing...");

    // Try VirtIO GPU if transport is provided
    if let Some(transport) = transport {
        log::info!("[GPU] Attempting VirtIO GPU initialization...");
        match los_gpu::Gpu::new(transport) {
            Ok(gpu) => {
                log::info!("[GPU] Initialized via VirtIO transport");
                *GPU.lock() = Some(GpuState { backend: GpuBackend::VirtIO(gpu) });
                return;
            }
            Err(e) => {
                log::error!("[GPU] Failed to create VirtIO GPU: {:?}", e);
            }
        }
    } else {
        log::info!("[GPU] No VirtIO transport provided");
    }

    // TEAM_331: Fall back to Limine framebuffer if available
    if let Some(boot_info) = crate::boot::boot_info() {
        if let Some(ref fb) = boot_info.framebuffer {
            log::info!("[GPU] Using Limine framebuffer fallback: {}x{}", fb.width, fb.height);
            let fb_gpu = FramebufferGpu::new(fb);
            
            // Clear framebuffer to black
            let mut state = GpuState { backend: GpuBackend::Framebuffer(fb_gpu) };
            let framebuffer = state.framebuffer();
            for byte in framebuffer.iter_mut() {
                *byte = 0;
            }
            
            *GPU.lock() = Some(state);
            return;
        }
    }

    log::warn!("[GPU] No GPU available (no VirtIO GPU and no Limine framebuffer)");
}

pub fn get_resolution() -> Option<(u32, u32)> {
    GPU.lock().as_ref().map(|s| s.dimensions())
}

// TEAM_141: Removed duplicate Display type - use los_gpu::Display via GpuState::as_display()
