extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicU64, Ordering};
use los_utils::Mutex;

use crate::fs::mode;
use crate::fs::vfs::error::{VfsError, VfsResult};
use crate::fs::vfs::inode::Inode;
use crate::fs::vfs::ops::{DirEntry, InodeOps, SetAttr};
use crate::fs::vfs::superblock::{StatFs, Superblock};

pub use los_utils::cpio::{CpioArchive, CpioEntry, CpioEntryType};

/// TEAM_205: Initramfs Inode Operations (Shared for all types)
pub struct InitramfsInodeOps;
static INITRAMFS_OPS: InitramfsInodeOps = InitramfsInodeOps;

impl InodeOps for InitramfsInodeOps {
    fn lookup(&self, inode: &Inode, name: &str) -> VfsResult<Arc<Inode>> {
        let path = inode.private::<String>().ok_or(VfsError::InternalError)?;
        let full_path = if path.is_empty() || path == "/" {
            name.to_string()
        } else {
            alloc::format!("{}/{}", path.trim_end_matches('/'), name)
        };

        let sb = inode.sb.upgrade().ok_or(VfsError::InternalError)?;
        let initramfs_sb = sb
            .as_any()
            .downcast_ref::<InitramfsSuperblock>()
            .ok_or(VfsError::InternalError)?;

        // Find entry in CPIO
        for entry in initramfs_sb.archive.iter() {
            if entry.name == full_path {
                return Ok(initramfs_sb.make_inode(entry, inode.sb.clone()));
            }
        }

        // Special case: check if it's a directory (CPIO might not have explicit dir entries for all paths)
        if initramfs_sb.archive.is_directory(&full_path) {
            // Create a virtual directory inode
            return Ok(initramfs_sb.make_virtual_dir_inode(&full_path, inode.sb.clone()));
        }

        Err(VfsError::NotFound)
    }

    fn read(&self, inode: &Inode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let path = inode.private::<String>().ok_or(VfsError::InternalError)?;
        let sb = inode.sb.upgrade().ok_or(VfsError::InternalError)?;
        let initramfs_sb = sb
            .as_any()
            .downcast_ref::<InitramfsSuperblock>()
            .ok_or(VfsError::InternalError)?;

        let data = initramfs_sb
            .archive
            .get_file(&path)
            .ok_or(VfsError::NotFound)?;
        let offset = offset as usize;
        if offset >= data.len() {
            return Ok(0);
        }

        let available = data.len() - offset;
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&data[offset..offset + to_read]);
        Ok(to_read)
    }

    fn readdir(&self, inode: &Inode, offset: usize) -> VfsResult<Option<DirEntry>> {
        let path = inode.private::<String>().ok_or(VfsError::InternalError)?;
        let sb = inode.sb.upgrade().ok_or(VfsError::InternalError)?;
        let initramfs_sb = sb
            .as_any()
            .downcast_ref::<InitramfsSuperblock>()
            .ok_or(VfsError::InternalError)?;

        // offset 0 = ., 1 = ..
        if offset == 0 {
            return Ok(Some(DirEntry {
                ino: inode.ino,
                name: ".".to_string(),
                file_type: mode::S_IFDIR,
            }));
        }
        if offset == 1 {
            return Ok(Some(DirEntry {
                ino: 1, // Assume root is 1
                name: "..".to_string(),
                file_type: mode::S_IFDIR,
            }));
        }

        // List directory entries from CPIO
        let mut idx = 2usize;
        for entry in initramfs_sb.archive.list_directory(&path) {
            if idx == offset {
                return Ok(Some(DirEntry {
                    ino: entry.ino,
                    name: entry
                        .name
                        .split('/')
                        .last()
                        .unwrap_or(entry.name)
                        .to_string(),
                    file_type: match entry.entry_type {
                        CpioEntryType::File => mode::S_IFREG,
                        CpioEntryType::Directory => mode::S_IFDIR,
                        CpioEntryType::Symlink => mode::S_IFLNK,
                        _ => mode::S_IFREG,
                    },
                }));
            }
            idx += 1;
        }

        Ok(None)
    }

    fn readlink(&self, inode: &Inode) -> VfsResult<String> {
        let path = inode.private::<String>().ok_or(VfsError::InternalError)?;
        let sb = inode.sb.upgrade().ok_or(VfsError::InternalError)?;
        let initramfs_sb = sb
            .as_any()
            .downcast_ref::<InitramfsSuperblock>()
            .ok_or(VfsError::InternalError)?;

        // In CPIO, symlink target is stored in data
        let target_data = initramfs_sb
            .archive
            .get_file(&path)
            .ok_or(VfsError::NotASymlink)?;
        core::str::from_utf8(target_data)
            .map(|s| s.to_string())
            .map_err(|_| VfsError::IoError)
    }

    fn setattr(&self, _inode: &Inode, _attr: &SetAttr) -> VfsResult<()> {
        Err(VfsError::ReadOnlyFs)
    }
}

/// TEAM_205: Initramfs Superblock
pub struct InitramfsSuperblock {
    pub archive: CpioArchive<'static>,
    root_inode: Mutex<Option<Arc<Inode>>>,
    next_ino: AtomicU64,
}

impl InitramfsSuperblock {
    pub fn new(data: &'static [u8]) -> Self {
        Self {
            archive: CpioArchive::new(data),
            root_inode: Mutex::new(None),
            next_ino: AtomicU64::new(1000), // Start above CPIO inos
        }
    }

    pub fn make_inode(&self, entry: CpioEntry<'static>, sb: Weak<dyn Superblock>) -> Arc<Inode> {
        let mode = match entry.entry_type {
            CpioEntryType::File => mode::S_IFREG | 0o444,
            CpioEntryType::Directory => mode::S_IFDIR | 0o555,
            CpioEntryType::Symlink => mode::S_IFLNK | 0o777,
            _ => mode::S_IFREG | 0o444,
        };

        Arc::new(Inode::new(
            entry.ino,
            0,
            mode,
            &INITRAMFS_OPS,
            sb,
            Box::new(entry.name.to_string()),
        ))
    }

    pub fn make_virtual_dir_inode(&self, path: &str, sb: Weak<dyn Superblock>) -> Arc<Inode> {
        Arc::new(Inode::new(
            self.next_ino.fetch_add(1, Ordering::SeqCst),
            0,
            mode::S_IFDIR | 0o555,
            &INITRAMFS_OPS,
            sb,
            Box::new(path.to_string()),
        ))
    }
}

impl Superblock for InitramfsSuperblock {
    fn root(&self) -> Arc<Inode> {
        self.root_inode.lock().as_ref().unwrap().clone()
    }

    fn statfs(&self) -> VfsResult<StatFs> {
        Ok(StatFs {
            f_type: 0x01234567, // Random
            f_bsize: 4096,
            f_blocks: (self.archive.iter().count() as u64),
            f_bfree: 0,
            f_bavail: 0,
            f_files: (self.archive.iter().count() as u64),
            f_ffree: 0,
            f_namelen: 255,
            ..Default::default()
        })
    }

    fn fs_type(&self) -> &'static str {
        "initramfs"
    }

    fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

/// TEAM_205: Initialize initramfs as VFS root
pub fn init_vfs(data: &'static [u8]) -> Arc<InitramfsSuperblock> {
    let sb = Arc::new(InitramfsSuperblock::new(data));

    // Create root inode
    let root = Arc::new(Inode::new(
        1,
        0,
        mode::S_IFDIR | 0o555,
        &INITRAMFS_OPS,
        Arc::downgrade(&(Arc::clone(&sb) as Arc<dyn Superblock>)),
        Box::new("".to_string()),
    ));

    *sb.root_inode.lock() = Some(root);
    sb
}
