//! Pure Rust CPIO archive writer (newc format)
//!
//! TEAM_474: Pure Rust implementation, eliminates `find | cpio` dependency.
//!
//! The newc format uses ASCII headers (110 bytes each) followed by
//! filename and data, with 4-byte alignment.

use std::io::Write;

/// File type constants for mode field (matches POSIX S_IF* values)
const S_IFDIR: u32 = 0o040000; // Directory
const S_IFREG: u32 = 0o100000; // Regular file
const S_IFLNK: u32 = 0o120000; // Symbolic link
const S_IFCHR: u32 = 0o020000; // Character device
const S_IFBLK: u32 = 0o060000; // Block device

/// A single entry in the CPIO archive
pub struct CpioEntry {
    pub path: String,
    pub mode: u32,
    pub data: Vec<u8>,
    pub nlink: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub rdev_major: u32,
    pub rdev_minor: u32,
}

impl CpioEntry {
    fn new(path: String, mode: u32) -> Self {
        Self {
            path,
            mode,
            data: Vec::new(),
            nlink: 1,
            dev_major: 0,
            dev_minor: 0,
            rdev_major: 0,
            rdev_minor: 0,
        }
    }
}

/// CPIO archive builder
pub struct CpioArchive {
    entries: Vec<CpioEntry>,
    next_ino: u32,
}

impl CpioArchive {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_ino: 1,
        }
    }

    /// Add a directory entry
    pub fn add_directory(&mut self, path: &str, mode: u32) {
        let path = normalize_path(path);
        let mut entry = CpioEntry::new(path, S_IFDIR | (mode & 0o7777));
        entry.nlink = 2; // . and ..
        self.entries.push(entry);
    }

    /// Add a regular file
    pub fn add_file(&mut self, path: &str, data: &[u8], mode: u32) {
        let path = normalize_path(path);
        let mut entry = CpioEntry::new(path, S_IFREG | (mode & 0o7777));
        entry.data = data.to_vec();
        self.entries.push(entry);
    }

    /// Add a symbolic link
    pub fn add_symlink(&mut self, path: &str, target: &str) {
        let path = normalize_path(path);
        let mut entry = CpioEntry::new(path, S_IFLNK | 0o777);
        entry.data = target.as_bytes().to_vec();
        self.entries.push(entry);
    }

    /// Add a character device node
    pub fn add_char_device(&mut self, path: &str, mode: u32, major: u32, minor: u32) {
        let path = normalize_path(path);
        let mut entry = CpioEntry::new(path, S_IFCHR | (mode & 0o7777));
        entry.rdev_major = major;
        entry.rdev_minor = minor;
        self.entries.push(entry);
    }

    /// Add a block device node
    pub fn add_block_device(&mut self, path: &str, mode: u32, major: u32, minor: u32) {
        let path = normalize_path(path);
        let mut entry = CpioEntry::new(path, S_IFBLK | (mode & 0o7777));
        entry.rdev_major = major;
        entry.rdev_minor = minor;
        self.entries.push(entry);
    }

    /// Get number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Write the archive to a writer (adds TRAILER automatically)
    pub fn write<W: Write>(&mut self, mut writer: W) -> std::io::Result<u64> {
        let mut total_bytes = 0u64;

        for entry in &self.entries {
            let ino = self.next_ino;
            self.next_ino += 1;

            let namesize = entry.path.len() + 1; // +1 for null terminator
            let filesize = entry.data.len();

            // Write header (110 bytes)
            let header = format_header(
                ino,
                entry.mode,
                entry.nlink,
                filesize as u32,
                namesize as u32,
                entry.dev_major,
                entry.dev_minor,
                entry.rdev_major,
                entry.rdev_minor,
            );
            writer.write_all(header.as_bytes())?;
            total_bytes += header.len() as u64;

            // Write filename + null terminator
            writer.write_all(entry.path.as_bytes())?;
            writer.write_all(&[0])?;
            total_bytes += namesize as u64;

            // Pad header+name to 4-byte boundary
            let header_name_len = 110 + namesize;
            let pad = align_to_4(header_name_len) - header_name_len;
            if pad > 0 {
                writer.write_all(&vec![0u8; pad])?;
                total_bytes += pad as u64;
            }

            // Write file data
            if !entry.data.is_empty() {
                writer.write_all(&entry.data)?;
                total_bytes += entry.data.len() as u64;

                // Pad data to 4-byte boundary
                let data_pad = align_to_4(filesize) - filesize;
                if data_pad > 0 {
                    writer.write_all(&vec![0u8; data_pad])?;
                    total_bytes += data_pad as u64;
                }
            }
        }

        // Write TRAILER entry to mark end of archive
        let trailer = "TRAILER!!!";
        let namesize = trailer.len() + 1;
        let header = format_header(0, 0, 1, 0, namesize as u32, 0, 0, 0, 0);
        writer.write_all(header.as_bytes())?;
        writer.write_all(trailer.as_bytes())?;
        writer.write_all(&[0])?;
        total_bytes += header.len() as u64 + namesize as u64;

        // Pad trailer to 4-byte boundary
        let trailer_len = 110 + namesize;
        let pad = align_to_4(trailer_len) - trailer_len;
        if pad > 0 {
            writer.write_all(&vec![0u8; pad])?;
            total_bytes += pad as u64;
        }

        Ok(total_bytes)
    }
}

impl Default for CpioArchive {
    fn default() -> Self {
        Self::new()
    }
}

/// Format newc header (110 bytes ASCII)
///
/// Format: "070701" magic + 13 fields, each 8 hex chars
fn format_header(
    ino: u32,
    mode: u32,
    nlink: u32,
    filesize: u32,
    namesize: u32,
    dev_major: u32,
    dev_minor: u32,
    rdev_major: u32,
    rdev_minor: u32,
) -> String {
    format!(
        "070701\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}\
         {:08X}",
        ino,       // c_ino
        mode,      // c_mode
        0,         // c_uid (root)
        0,         // c_gid (root)
        nlink,     // c_nlink
        0,         // c_mtime
        filesize,  // c_filesize
        dev_major, // c_devmajor
        dev_minor, // c_devminor
        rdev_major, // c_rdevmajor
        rdev_minor, // c_rdevminor
        namesize,  // c_namesize
        0,         // c_check (always 0 for newc)
    )
}

/// Normalize path: ensure no leading slash, handle "." specially
fn normalize_path(path: &str) -> String {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        ".".to_string()
    } else {
        path.to_string()
    }
}

/// Align value up to 4-byte boundary
fn align_to_4(n: usize) -> usize {
    (n + 3) & !3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_format() {
        let header = format_header(1, 0o100755, 1, 100, 5, 0, 0, 0, 0);
        assert_eq!(header.len(), 110);
        assert!(header.starts_with("070701"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/bin/ls"), "bin/ls");
        assert_eq!(normalize_path("bin/ls"), "bin/ls");
        assert_eq!(normalize_path("/"), ".");
        assert_eq!(normalize_path(""), ".");
    }

    #[test]
    fn test_align_to_4() {
        assert_eq!(align_to_4(0), 0);
        assert_eq!(align_to_4(1), 4);
        assert_eq!(align_to_4(2), 4);
        assert_eq!(align_to_4(3), 4);
        assert_eq!(align_to_4(4), 4);
        assert_eq!(align_to_4(5), 8);
    }

    #[test]
    fn test_basic_archive() {
        let mut archive = CpioArchive::new();
        archive.add_directory("bin", 0o755);
        archive.add_file("bin/hello", b"Hello, World!", 0o755);
        archive.add_symlink("bin/hi", "hello");

        let mut output = Vec::new();
        let bytes = archive.write(&mut output).unwrap();
        assert!(bytes > 0);
        assert!(output.starts_with(b"070701"));
    }
}
