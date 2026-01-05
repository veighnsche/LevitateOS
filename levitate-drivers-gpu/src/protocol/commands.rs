//! VirtIO GPU Command Structures
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! Command structures for VirtIO GPU 2D operations.

use super::{CtrlHeader, CtrlType, Format, Rect, ResourceId};

/// RESOURCE_CREATE_2D command.
///
/// Creates a 2D resource on the host.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdResourceCreate2d {
    pub header: CtrlHeader,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}

impl CmdResourceCreate2d {
    /// Create a new RESOURCE_CREATE_2D command.
    pub fn new(resource_id: ResourceId, format: Format, width: u32, height: u32) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::ResourceCreate2d),
            resource_id: resource_id.raw(),
            format: format as u32,
            width,
            height,
        }
    }
}

/// RESOURCE_UNREF command.
///
/// Destroys a resource on the host.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdResourceUnref {
    pub header: CtrlHeader,
    pub resource_id: u32,
    pub padding: u32,
}

impl CmdResourceUnref {
    /// Create a new RESOURCE_UNREF command.
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::ResourceUnref),
            resource_id: resource_id.raw(),
            padding: 0,
        }
    }
}

/// SET_SCANOUT command.
///
/// Links a resource to a display scanout.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdSetScanout {
    pub header: CtrlHeader,
    pub rect: Rect,
    pub scanout_id: u32,
    pub resource_id: u32,
}

impl CmdSetScanout {
    /// Create a new SET_SCANOUT command.
    pub fn new(scanout_id: u32, resource_id: ResourceId, rect: Rect) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::SetScanout),
            rect,
            scanout_id,
            resource_id: resource_id.raw(),
        }
    }

    /// Create a command to disable a scanout.
    pub fn disable(scanout_id: u32) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::SetScanout),
            rect: Rect::default(),
            scanout_id,
            resource_id: 0,
        }
    }
}

/// RESOURCE_FLUSH command.
///
/// Flushes a resource to the display.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdResourceFlush {
    pub header: CtrlHeader,
    pub rect: Rect,
    pub resource_id: u32,
    pub padding: u32,
}

impl CmdResourceFlush {
    /// Create a new RESOURCE_FLUSH command.
    pub fn new(resource_id: ResourceId, rect: Rect) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::ResourceFlush),
            rect,
            resource_id: resource_id.raw(),
            padding: 0,
        }
    }
}

/// TRANSFER_TO_HOST_2D command.
///
/// Transfers data from guest memory to host resource.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdTransferToHost2d {
    pub header: CtrlHeader,
    pub rect: Rect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}

impl CmdTransferToHost2d {
    /// Create a new TRANSFER_TO_HOST_2D command.
    pub fn new(resource_id: ResourceId, rect: Rect, offset: u64) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::TransferToHost2d),
            rect,
            offset,
            resource_id: resource_id.raw(),
            padding: 0,
        }
    }
}

/// RESOURCE_ATTACH_BACKING command header.
///
/// Attaches guest memory pages to a resource.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdResourceAttachBacking {
    pub header: CtrlHeader,
    pub resource_id: u32,
    pub nr_entries: u32,
}

impl CmdResourceAttachBacking {
    /// Create a new RESOURCE_ATTACH_BACKING command header.
    pub fn new(resource_id: ResourceId, nr_entries: u32) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::ResourceAttachBacking),
            resource_id: resource_id.raw(),
            nr_entries,
        }
    }
}

/// Memory entry for RESOURCE_ATTACH_BACKING.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}

impl MemEntry {
    /// Create a new memory entry.
    pub const fn new(addr: u64, length: u32) -> Self {
        Self {
            addr,
            length,
            padding: 0,
        }
    }
}

/// RESOURCE_DETACH_BACKING command.
///
/// Detaches guest memory from a resource.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdResourceDetachBacking {
    pub header: CtrlHeader,
    pub resource_id: u32,
    pub padding: u32,
}

impl CmdResourceDetachBacking {
    /// Create a new RESOURCE_DETACH_BACKING command.
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::ResourceDetachBacking),
            resource_id: resource_id.raw(),
            padding: 0,
        }
    }
}

/// GET_DISPLAY_INFO command.
///
/// Retrieves display configuration from the host.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdGetDisplayInfo {
    pub header: CtrlHeader,
}

impl CmdGetDisplayInfo {
    /// Create a new GET_DISPLAY_INFO command.
    pub fn new() -> Self {
        Self {
            header: CtrlHeader::new(CtrlType::GetDisplayInfo),
        }
    }
}

impl Default for CmdGetDisplayInfo {
    fn default() -> Self {
        Self::new()
    }
}
