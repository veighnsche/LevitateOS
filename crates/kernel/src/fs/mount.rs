//! TEAM_201: Mount Table for VFS
//!
//! Provides infrastructure for tracking mounted filesystems at mount points.
//! This replaces the current hardcoded `/tmp` check in tmpfs.
//!
//! Key features:
//! - Mount table with longest-prefix matching
//! - Mount/unmount operations
//! - Thread-safe access via RwLock

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use los_utils::RwLock;

use super::path::{Path, PathBuf};

/// TEAM_201: Mount flags
#[derive(Clone, Copy, Debug, Default)]
pub struct MountFlags {
    /// Read-only mount
    pub readonly: bool,
    /// Don't allow setuid/setgid
    pub nosuid: bool,
    /// Don't allow execution
    pub noexec: bool,
    /// Don't update access times
    pub noatime: bool,
}

impl MountFlags {
    /// TEAM_201: Create default (read-write) mount flags
    pub const fn new() -> Self {
        Self {
            readonly: false,
            nosuid: false,
            noexec: false,
            noatime: false,
        }
    }

    /// TEAM_201: Create read-only mount flags
    pub const fn readonly() -> Self {
        Self {
            readonly: true,
            nosuid: false,
            noexec: false,
            noatime: false,
        }
    }
}

/// TEAM_201: Filesystem type identifier
///
/// Used to identify the type of filesystem for a mount.
/// This will be extended as more filesystems are added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsType {
    /// Temporary in-memory filesystem
    Tmpfs,
    /// Initramfs (CPIO archive, read-only)
    Initramfs,
    /// FAT32 filesystem
    Fat32,
    /// ext4 filesystem (read-only)
    Ext4,
}

impl FsType {
    /// TEAM_201: Get filesystem type name
    pub fn name(&self) -> &'static str {
        match self {
            FsType::Tmpfs => "tmpfs",
            FsType::Initramfs => "initramfs",
            FsType::Fat32 => "fat32",
            FsType::Ext4 => "ext4",
        }
    }
}

impl core::convert::TryFrom<&str> for FsType {
    type Error = MountError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "tmpfs" => Ok(FsType::Tmpfs),
            "initramfs" => Ok(FsType::Initramfs),
            "fat32" => Ok(FsType::Fat32),
            "ext4" => Ok(FsType::Ext4),
            _ => Err(MountError::UnsupportedFsType),
        }
    }
}

/// TEAM_201: A mounted filesystem entry
///
/// Note: The actual superblock/inode references will be added in Phase 13.
/// For now, we just track the mount point and filesystem type.
#[derive(Clone)]
pub struct Mount {
    /// Mount point path (normalized, absolute)
    pub mountpoint: PathBuf,
    /// Filesystem type
    pub fstype: FsType,
    /// Mount flags
    pub flags: MountFlags,
    /// Source device or path (e.g., "/dev/sda1" or "none" for tmpfs)
    pub source: String,
}

impl Mount {
    /// TEAM_201: Create a new mount entry
    pub fn new(mountpoint: PathBuf, fstype: FsType, flags: MountFlags, source: String) -> Self {
        Self {
            mountpoint,
            fstype,
            flags,
            source,
        }
    }
}

/// TEAM_201: The mount table
///
/// Tracks all mounted filesystems. Uses longest-prefix matching to find
/// which filesystem handles a given path.
pub struct MountTable {
    /// List of mounts, sorted by mountpoint length (longest first)
    mounts: Vec<Mount>,
}

impl MountTable {
    /// TEAM_201: Create an empty mount table
    pub const fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    /// TEAM_201: Mount a filesystem at the given path
    ///
    /// Returns an error if something is already mounted at exactly that path.
    pub fn mount(
        &mut self,
        mountpoint: &Path,
        fstype: FsType,
        flags: MountFlags,
        source: &str,
    ) -> Result<(), MountError> {
        // Normalize the mountpoint
        let mountpoint = super::path::normalize(mountpoint);

        // Check if already mounted at this exact path
        for mount in &self.mounts {
            if mount.mountpoint.as_str() == mountpoint.as_str() {
                return Err(MountError::AlreadyMounted);
            }
        }

        // Add the mount
        let mount = Mount::new(mountpoint, fstype, flags, String::from(source));
        self.mounts.push(mount);

        // Sort by mountpoint length (longest first) for longest-prefix matching
        self.mounts.sort_by(|a, b| {
            b.mountpoint
                .as_str()
                .len()
                .cmp(&a.mountpoint.as_str().len())
        });

        Ok(())
    }

    /// TEAM_201: Unmount a filesystem at the given path
    pub fn umount(&mut self, mountpoint: &Path) -> Result<(), MountError> {
        let mountpoint = super::path::normalize(mountpoint);
        let mountpoint_str = mountpoint.as_str();

        let idx = self
            .mounts
            .iter()
            .position(|m| m.mountpoint.as_str() == mountpoint_str);

        match idx {
            Some(i) => {
                self.mounts.remove(i);
                Ok(())
            }
            None => Err(MountError::NotMounted),
        }
    }

    /// TEAM_201: Find the mount that handles the given path
    ///
    /// Uses longest-prefix matching: returns the mount with the longest
    /// mountpoint that is a prefix of the given path.
    /// Returns (mount, relative_path_str) where relative_path_str is the path within the mount.
    pub fn lookup<'a>(&'a self, path: &'a str) -> Option<(&'a Mount, &'a str)> {
        for mount in &self.mounts {
            let mountpoint_str = mount.mountpoint.as_str();

            // Check if mountpoint is a prefix of path
            if path.starts_with(mountpoint_str) {
                // Make sure it's a proper path prefix (ends at / or end of string)
                let remaining = &path[mountpoint_str.len()..];
                if remaining.is_empty() || remaining.starts_with('/') {
                    // Return the relative path within the mount
                    let relative = remaining.trim_start_matches('/');
                    return Some((mount, relative));
                }
            }
        }

        None
    }

    /// TEAM_201: Check if a path is under a specific mount
    pub fn is_mounted_at(&self, path: &str, mountpoint: &Path) -> bool {
        match self.lookup(path) {
            Some((mount, _)) => mount.mountpoint.as_str() == mountpoint.as_str(),
            None => false,
        }
    }

    /// TEAM_201: Get all mounts
    pub fn iter(&self) -> impl Iterator<Item = &Mount> {
        self.mounts.iter()
    }

    /// TEAM_201: Get mount count
    pub fn len(&self) -> usize {
        self.mounts.len()
    }

    /// TEAM_201: Check if mount table is empty
    pub fn is_empty(&self) -> bool {
        self.mounts.is_empty()
    }
}

impl Default for MountTable {
    fn default() -> Self {
        Self::new()
    }
}

/// TEAM_201: Mount error type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountError {
    /// Something is already mounted at this path
    AlreadyMounted,
    /// Nothing is mounted at this path
    NotMounted,
    /// Invalid mount point
    InvalidMountpoint,
    /// Filesystem type not supported
    UnsupportedFsType,
    /// Permission denied
    PermissionDenied,
}

// ============================================================================
// Global mount table
// ============================================================================

/// TEAM_201: Global mount table
///
/// This will be initialized during boot with the root filesystem.
static MOUNT_TABLE: RwLock<MountTable> = RwLock::new(MountTable::new());

/// TEAM_201: Get a read lock on the global mount table
pub fn mounts() -> los_utils::RwLockReadGuard<'static, MountTable> {
    MOUNT_TABLE.read()
}

/// TEAM_201: Get a write lock on the global mount table
pub fn mounts_mut() -> los_utils::RwLockWriteGuard<'static, MountTable> {
    MOUNT_TABLE.write()
}

/// TEAM_201: Mount a filesystem (convenience function)
pub fn mount(
    mountpoint: &Path,
    fstype: FsType,
    flags: MountFlags,
    source: &str,
) -> Result<(), MountError> {
    mounts_mut().mount(mountpoint, fstype, flags, source)
}

/// TEAM_201: Unmount a filesystem (convenience function)
pub fn umount(mountpoint: &Path) -> Result<(), MountError> {
    mounts_mut().umount(mountpoint)
}

/// TEAM_201: Initialize default mounts
///
/// Called during kernel boot to set up initial mount points.
pub fn init() {
    let mut table = mounts_mut();

    // Mount tmpfs at /tmp
    let _ = table.mount(Path::new("/tmp"), FsType::Tmpfs, MountFlags::new(), "none");

    // Mount initramfs at root
    let _ = table.mount(
        Path::new("/"),
        FsType::Initramfs,
        MountFlags::readonly(),
        "initramfs",
    );
}
