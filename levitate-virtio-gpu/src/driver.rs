//! VirtIO GPU Driver with State Machine
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module provides the core GPU driver with:
//! - Explicit state machine per Tock patterns
//! - Full command/response visibility
//! - Async-first design per user requirement Q4

use crate::command::{GpuRequest, GpuResponse, HeaderOnlyResponse};
use crate::protocol::{
    CmdGetDisplayInfo, CmdResourceAttachBacking, CmdResourceCreate2d, CmdResourceFlush,
    CmdSetScanout, CmdTransferToHost2d, CtrlHeader, DisplayOne, Format, GpuError,
    MemEntry, Rect, ResourceId, RespDisplayInfo,
};

/// Driver state machine states.
///
/// Per VirtIO GPU initialization sequence from spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    /// Driver not initialized.
    Uninitialized,
    /// Transport configured, querying display info.
    QueryingDisplayInfo,
    /// Display info received, creating framebuffer resource.
    CreatingResource,
    /// Resource created, attaching backing memory.
    AttachingBacking,
    /// Backing attached, setting scanout.
    SettingScanout,
    /// Scanout set, driver is ready for rendering.
    Ready,
    /// Driver encountered an error.
    Failed,
}

impl DriverState {
    /// Check if driver is ready for rendering.
    pub fn is_ready(self) -> bool {
        self == Self::Ready
    }

    /// Check if driver has failed.
    pub fn is_failed(self) -> bool {
        self == Self::Failed
    }
}

/// Telemetry counters for driver operations.
#[derive(Debug, Default, Clone, Copy)]
pub struct DriverTelemetry {
    pub commands_sent: u64,
    pub responses_received: u64,
    pub errors: u64,
    pub flushes: u64,
    pub transfers: u64,
}

/// Configuration for GPU driver.
#[derive(Debug, Clone, Copy)]
pub struct DriverConfig {
    /// Scanout ID to use (usually 0).
    pub scanout_id: u32,
    /// Pixel format for framebuffer.
    pub format: Format,
    /// Resource ID for framebuffer.
    pub framebuffer_resource_id: ResourceId,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            scanout_id: 0,
            format: Format::B8G8R8A8Unorm,
            framebuffer_resource_id: ResourceId::new(0xBABE),
        }
    }
}

/// Display information retrieved from device.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub enabled: bool,
}

impl From<&DisplayOne> for DisplayInfo {
    fn from(d: &DisplayOne) -> Self {
        Self {
            width: d.rect.width,
            height: d.rect.height,
            enabled: d.enabled != 0,
        }
    }
}

/// VirtIO GPU Driver.
///
/// This is the main driver struct that manages the GPU device.
/// Unlike `virtio-drivers`, all state is visible and all commands are logged.
pub struct GpuDriver {
    /// Current driver state.
    state: DriverState,
    /// Driver configuration.
    config: DriverConfig,
    /// Display information from device.
    display_info: DisplayInfo,
    /// Framebuffer dimensions.
    fb_rect: Rect,
    /// Telemetry counters.
    telemetry: DriverTelemetry,
    /// Command buffer for building requests.
    cmd_buffer: [u8; 256],
    /// Response buffer for receiving responses.
    resp_buffer: [u8; 512],
}

impl GpuDriver {
    /// Create a new GPU driver.
    pub const fn new() -> Self {
        Self {
            state: DriverState::Uninitialized,
            config: DriverConfig {
                scanout_id: 0,
                format: Format::B8G8R8A8Unorm,
                framebuffer_resource_id: ResourceId(0xBABE),
            },
            display_info: DisplayInfo {
                width: 0,
                height: 0,
                enabled: false,
            },
            fb_rect: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            telemetry: DriverTelemetry {
                commands_sent: 0,
                responses_received: 0,
                errors: 0,
                flushes: 0,
                transfers: 0,
            },
            cmd_buffer: [0; 256],
            resp_buffer: [0; 512],
        }
    }

    /// Get the current driver state.
    pub fn state(&self) -> DriverState {
        self.state
    }

    /// Get the display info.
    pub fn display_info(&self) -> &DisplayInfo {
        &self.display_info
    }

    /// Get the framebuffer rectangle.
    pub fn framebuffer_rect(&self) -> Rect {
        self.fb_rect
    }

    /// Get telemetry counters.
    pub fn telemetry(&self) -> &DriverTelemetry {
        &self.telemetry
    }

    /// Get the resolution as (width, height).
    pub fn resolution(&self) -> (u32, u32) {
        (self.display_info.width, self.display_info.height)
    }

    /// Check if driver is ready.
    pub fn is_ready(&self) -> bool {
        self.state.is_ready()
    }

    /// Build a GET_DISPLAY_INFO command.
    pub fn build_get_display_info(&mut self) -> &[u8] {
        let cmd = CmdGetDisplayInfo::new();
        let bytes = cmd.as_bytes();
        let len = bytes.len();
        self.cmd_buffer[..len].copy_from_slice(bytes);
        self.telemetry.commands_sent += 1;
        &self.cmd_buffer[..len]
    }

    /// Parse GET_DISPLAY_INFO response and update state.
    pub fn handle_display_info_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        if response.len() < core::mem::size_of::<RespDisplayInfo>() {
            return Err(GpuError::TransportError);
        }

        // Parse response header
        let header = unsafe { &*(response.as_ptr().cast::<CtrlHeader>()) };
        if header.is_err() {
            self.state = DriverState::Failed;
            return Err(GpuError::DeviceError(
                header.ctrl_type().unwrap_or(crate::protocol::CtrlType::ErrUnspec),
            ));
        }

        // Parse display info
        let resp = unsafe { &*(response.as_ptr().cast::<RespDisplayInfo>()) };
        let scanout = &resp.pmodes[self.config.scanout_id as usize];
        
        self.display_info = DisplayInfo::from(scanout);
        self.fb_rect = Rect::from_size(self.display_info.width, self.display_info.height);
        self.state = DriverState::CreatingResource;
        self.telemetry.responses_received += 1;

        Ok(())
    }

    /// Build a RESOURCE_CREATE_2D command.
    pub fn build_resource_create_2d(&mut self) -> &[u8] {
        let cmd = CmdResourceCreate2d::new(
            self.config.framebuffer_resource_id,
            self.config.format,
            self.display_info.width,
            self.display_info.height,
        );
        let bytes = cmd.as_bytes();
        let len = bytes.len();
        self.cmd_buffer[..len].copy_from_slice(bytes);
        self.telemetry.commands_sent += 1;
        &self.cmd_buffer[..len]
    }

    /// Handle RESOURCE_CREATE_2D response.
    pub fn handle_resource_create_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        self.check_ok_nodata_response(response)?;
        self.state = DriverState::AttachingBacking;
        Ok(())
    }

    /// Build a RESOURCE_ATTACH_BACKING command.
    ///
    /// `backing_addr` is the physical address of the framebuffer memory.
    /// `backing_len` is the size in bytes.
    pub fn build_attach_backing(&mut self, backing_addr: u64, backing_len: u32) -> &[u8] {
        // Build command header
        let cmd = CmdResourceAttachBacking::new(self.config.framebuffer_resource_id, 1);
        let cmd_bytes = unsafe {
            core::slice::from_raw_parts(
                (&cmd as *const CmdResourceAttachBacking).cast::<u8>(),
                core::mem::size_of::<CmdResourceAttachBacking>(),
            )
        };
        let cmd_len = cmd_bytes.len();
        self.cmd_buffer[..cmd_len].copy_from_slice(cmd_bytes);

        // Build memory entry
        let entry = MemEntry::new(backing_addr, backing_len);
        let entry_bytes = unsafe {
            core::slice::from_raw_parts(
                (&entry as *const MemEntry).cast::<u8>(),
                core::mem::size_of::<MemEntry>(),
            )
        };
        let entry_len = entry_bytes.len();
        self.cmd_buffer[cmd_len..cmd_len + entry_len].copy_from_slice(entry_bytes);

        self.telemetry.commands_sent += 1;
        &self.cmd_buffer[..cmd_len + entry_len]
    }

    /// Handle RESOURCE_ATTACH_BACKING response.
    pub fn handle_attach_backing_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        self.check_ok_nodata_response(response)?;
        self.state = DriverState::SettingScanout;
        Ok(())
    }

    /// Build a SET_SCANOUT command.
    pub fn build_set_scanout(&mut self) -> &[u8] {
        let cmd = CmdSetScanout::new(
            self.config.scanout_id,
            self.config.framebuffer_resource_id,
            self.fb_rect,
        );
        let bytes = cmd.as_bytes();
        let len = bytes.len();
        self.cmd_buffer[..len].copy_from_slice(bytes);
        self.telemetry.commands_sent += 1;
        &self.cmd_buffer[..len]
    }

    /// Handle SET_SCANOUT response.
    pub fn handle_set_scanout_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        self.check_ok_nodata_response(response)?;
        self.state = DriverState::Ready;
        Ok(())
    }

    /// Build a TRANSFER_TO_HOST_2D command.
    pub fn build_transfer_to_host(&mut self, rect: Rect) -> &[u8] {
        let cmd = CmdTransferToHost2d::new(self.config.framebuffer_resource_id, rect, 0);
        let bytes = cmd.as_bytes();
        let len = bytes.len();
        self.cmd_buffer[..len].copy_from_slice(bytes);
        self.telemetry.commands_sent += 1;
        self.telemetry.transfers += 1;
        &self.cmd_buffer[..len]
    }

    /// Build a RESOURCE_FLUSH command.
    pub fn build_flush(&mut self, rect: Rect) -> &[u8] {
        let cmd = CmdResourceFlush::new(self.config.framebuffer_resource_id, rect);
        let bytes = cmd.as_bytes();
        let len = bytes.len();
        self.cmd_buffer[..len].copy_from_slice(bytes);
        self.telemetry.commands_sent += 1;
        self.telemetry.flushes += 1;
        &self.cmd_buffer[..len]
    }

    /// Handle a generic OK_NODATA response.
    pub fn handle_ok_nodata_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        self.check_ok_nodata_response(response)
    }

    /// Get a mutable reference to the response buffer.
    pub fn response_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.resp_buffer
    }

    /// Check an OK_NODATA response.
    fn check_ok_nodata_response(&mut self, response: &[u8]) -> Result<(), GpuError> {
        if response.len() < CtrlHeader::SIZE {
            self.telemetry.errors += 1;
            return Err(GpuError::TransportError);
        }

        let header = unsafe { &*(response.as_ptr().cast::<CtrlHeader>()) };
        self.telemetry.responses_received += 1;

        if header.is_err() {
            self.telemetry.errors += 1;
            return Err(GpuError::DeviceError(
                header.ctrl_type().unwrap_or(crate::protocol::CtrlType::ErrUnspec),
            ));
        }

        Ok(())
    }
}

impl Default for GpuDriver {
    fn default() -> Self {
        Self::new()
    }
}

// Implement GpuRequest for command structs
impl GpuRequest for CmdGetDisplayInfo {
    type Response = RespDisplayInfo;

    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}

impl GpuRequest for CmdResourceCreate2d {
    type Response = HeaderOnlyResponse;

    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}

impl GpuRequest for CmdSetScanout {
    type Response = HeaderOnlyResponse;

    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}

impl GpuRequest for CmdResourceFlush {
    type Response = HeaderOnlyResponse;

    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}

impl GpuRequest for CmdTransferToHost2d {
    type Response = HeaderOnlyResponse;

    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}

// Implement GpuResponse for response structs
impl GpuResponse for RespDisplayInfo {
    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}
