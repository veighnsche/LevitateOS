//! VirtIO GPU Protocol Structures
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module contains all protocol structures per VirtIO 1.1 Section 5.7.
//! All structs are repr(C) for correct memory layout and zerocopy compatibility.

mod commands;
mod formats;

pub use commands::*;
pub use formats::*;

/// Error type for GPU operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuError {
    /// Device returned an error response.
    DeviceError(CtrlType),
    /// Transport error during command.
    TransportError,
    /// Invalid parameter provided.
    InvalidParameter,
    /// Resource not found.
    ResourceNotFound,
    /// Device not initialized.
    NotInitialized,
    /// Timeout waiting for response.
    Timeout,
}

/// Newtype for resource IDs to prevent mixing with other u32 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ResourceId(pub u32);

impl ResourceId {
    /// Create a new resource ID.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Rectangle structure used throughout the GPU protocol.
///
/// Per VirtIO 1.1 Section 5.7.6.8.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Create a new rectangle.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle at origin with given size.
    pub const fn from_size(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, width, height }
    }
}

/// Control header type field values.
///
/// Per VirtIO 1.1 Section 5.7.6.7.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtrlType {
    // 2D Commands
    GetDisplayInfo = 0x0100,
    ResourceCreate2d = 0x0101,
    ResourceUnref = 0x0102,
    SetScanout = 0x0103,
    ResourceFlush = 0x0104,
    TransferToHost2d = 0x0105,
    ResourceAttachBacking = 0x0106,
    ResourceDetachBacking = 0x0107,
    GetCapsetInfo = 0x0108,
    GetCapset = 0x0109,
    GetEdid = 0x010a,

    // Cursor Commands
    UpdateCursor = 0x0300,
    MoveCursor = 0x0301,

    // Success Responses
    OkNodata = 0x1100,
    OkDisplayInfo = 0x1101,
    OkCapsetInfo = 0x1102,
    OkCapset = 0x1103,
    OkEdid = 0x1104,

    // Error Responses
    ErrUnspec = 0x1200,
    ErrOutOfMemory = 0x1201,
    ErrInvalidScanoutId = 0x1202,
    ErrInvalidResourceId = 0x1203,
    ErrInvalidContextId = 0x1204,
    ErrInvalidParameter = 0x1205,
}

impl CtrlType {
    /// Check if this is an error response.
    pub const fn is_error(self) -> bool {
        (self as u32) >= 0x1200
    }

    /// Check if this is a success response.
    pub const fn is_success(self) -> bool {
        let val = self as u32;
        val >= 0x1100 && val < 0x1200
    }

    /// Try to parse from raw u32.
    pub fn from_raw(val: u32) -> Option<Self> {
        match val {
            0x0100 => Some(Self::GetDisplayInfo),
            0x0101 => Some(Self::ResourceCreate2d),
            0x0102 => Some(Self::ResourceUnref),
            0x0103 => Some(Self::SetScanout),
            0x0104 => Some(Self::ResourceFlush),
            0x0105 => Some(Self::TransferToHost2d),
            0x0106 => Some(Self::ResourceAttachBacking),
            0x0107 => Some(Self::ResourceDetachBacking),
            0x0108 => Some(Self::GetCapsetInfo),
            0x0109 => Some(Self::GetCapset),
            0x010a => Some(Self::GetEdid),
            0x0300 => Some(Self::UpdateCursor),
            0x0301 => Some(Self::MoveCursor),
            0x1100 => Some(Self::OkNodata),
            0x1101 => Some(Self::OkDisplayInfo),
            0x1102 => Some(Self::OkCapsetInfo),
            0x1103 => Some(Self::OkCapset),
            0x1104 => Some(Self::OkEdid),
            0x1200 => Some(Self::ErrUnspec),
            0x1201 => Some(Self::ErrOutOfMemory),
            0x1202 => Some(Self::ErrInvalidScanoutId),
            0x1203 => Some(Self::ErrInvalidResourceId),
            0x1204 => Some(Self::ErrInvalidContextId),
            0x1205 => Some(Self::ErrInvalidParameter),
            _ => None,
        }
    }
}

/// Control header for all GPU commands and responses.
///
/// Per VirtIO 1.1 Section 5.7.6.7.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtrlHeader {
    /// Command or response type.
    pub ctrl_type: u32,
    /// Flags (VIRTIO_GPU_FLAG_FENCE = 1).
    pub flags: u32,
    /// Fence ID for synchronization.
    pub fence_id: u64,
    /// Context ID (3D mode only, 0 for 2D).
    pub ctx_id: u32,
    /// Padding.
    pub padding: u32,
}

impl CtrlHeader {
    /// Size of the control header in bytes.
    pub const SIZE: usize = 24;

    /// Create a new command header.
    pub const fn new(ctrl_type: CtrlType) -> Self {
        Self {
            ctrl_type: ctrl_type as u32,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0,
        }
    }

    /// Get the control type.
    pub fn ctrl_type(&self) -> Option<CtrlType> {
        CtrlType::from_raw(self.ctrl_type)
    }

    /// Check if response indicates success.
    pub fn is_ok(&self) -> bool {
        self.ctrl_type().is_some_and(|t| t.is_success())
    }

    /// Check if response indicates error.
    pub fn is_err(&self) -> bool {
        self.ctrl_type().is_some_and(|t| t.is_error())
    }
}

impl Default for CtrlHeader {
    fn default() -> Self {
        Self {
            ctrl_type: 0,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0,
        }
    }
}

/// Maximum number of scanouts supported.
pub const MAX_SCANOUTS: usize = 16;

/// Display information for a single scanout.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayOne {
    pub rect: Rect,
    pub enabled: u32,
    pub flags: u32,
}

/// Response to GET_DISPLAY_INFO command.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RespDisplayInfo {
    pub header: CtrlHeader,
    pub pmodes: [DisplayOne; MAX_SCANOUTS],
}

impl Default for RespDisplayInfo {
    fn default() -> Self {
        Self {
            header: CtrlHeader::default(),
            pmodes: [DisplayOne::default(); MAX_SCANOUTS],
        }
    }
}
