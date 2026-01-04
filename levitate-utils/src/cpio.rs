//! CPIO Archive Parser
//!
//! TEAM_039: Relocated from kernel/src/fs/initramfs.rs for testability
//!
//! Supports the CPIO New ASCII Format (newc/070701).

use core::str;

/// CPIO New ASCII Format Header
/// [CP1] Accepts "070701" magic, [CP2] Accepts "070702" magic
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpioHeader {
    pub c_magic: [u8; 6],
    pub c_ino: [u8; 8],
    pub c_mode: [u8; 8],
    pub c_uid: [u8; 8],
    pub c_gid: [u8; 8],
    pub c_nlink: [u8; 8],
    pub c_mtime: [u8; 8],
    pub c_filesize: [u8; 8],
    pub c_devmajor: [u8; 8],
    pub c_devminor: [u8; 8],
    pub c_rdevmajor: [u8; 8],
    pub c_rdevminor: [u8; 8],
    pub c_namesize: [u8; 8],
    pub c_check: [u8; 8],
} // 110 bytes

impl CpioHeader {
    /// [CP1] Accepts "070701" magic, [CP2] Accepts "070702" magic, [CP3] Rejects invalid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        &self.c_magic == b"070701" || &self.c_magic == b"070702"
    }

    #[must_use]
    pub fn filesize(&self) -> usize {
        parse_hex(&self.c_filesize)
    }

    #[must_use]
    pub fn namesize(&self) -> usize {
        parse_hex(&self.c_namesize)
    }
}

/// [CP4] Converts hex string to usize, [CP5] Returns 0 for invalid input
#[must_use]
pub fn parse_hex(bytes: &[u8]) -> usize {
    let s = str::from_utf8(bytes).unwrap_or("0"); // [CP5]
    usize::from_str_radix(s, 16).unwrap_or(0) // [CP5]
}

/// CPIO archive wrapper for iterating over entries
pub struct CpioArchive<'a> {
    data: &'a [u8],
}

impl<'a> CpioArchive<'a> {
    /// Create a new CPIO archive wrapper around a memory slice.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// [CP6] Returns entries in order
    #[must_use]
    pub fn iter(&self) -> CpioIterator<'a> {
        CpioIterator {
            data: self.data,
            offset: 0,
        }
    }

    /// [CP8] Finds existing file, [CP9] Returns None for missing file
    #[must_use]
    pub fn get_file(&self, path: &str) -> Option<&'a [u8]> {
        for entry in self.iter() {
            if entry.name == path {
                return Some(entry.data); // [CP8]
            }
        }
        None // [CP9]
    }
}

/// An entry in a CPIO archive
pub struct CpioEntry<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

/// Iterator over CPIO archive entries
pub struct CpioIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for CpioIterator<'a> {
    type Item = CpioEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 110 > self.data.len() {
            return None;
        }

        let header_ptr = unsafe { self.data.as_ptr().add(self.offset) } as *const CpioHeader;
        let header = unsafe { &*header_ptr };

        if !header.is_valid() {
            return None;
        }

        let filesize = header.filesize();
        let namesize = header.namesize();

        if namesize == 0 {
            return None;
        }

        // Parse Name
        let name_start = self.offset + 110;
        if name_start + namesize > self.data.len() {
            return None;
        }

        // CPIO names are null terminated, but namesize includes it.
        // We want a str, so we slice excluding the last byte if it's 0
        let raw_name = &self.data[name_start..name_start + namesize];
        let name_len = if *raw_name.last().unwrap_or(&0) == 0 {
            namesize - 1
        } else {
            namesize
        };
        let name = str::from_utf8(&raw_name[0..name_len]).unwrap_or("<invalid>");

        // [CP7] Iterator stops at TRAILER!!!
        if name == "TRAILER!!!" {
            return None;
        }

        // [CP10] Calculate Data Offset (4-byte aligned after header + name)
        let mut data_start = name_start + namesize;
        // Align to 4 bytes
        while data_start % 4 != 0 {
            data_start += 1;
        }

        if data_start + filesize > self.data.len() {
            return None;
        }

        let file_data = &self.data[data_start..data_start + filesize];

        // Prepare next offset (also 4-byte aligned)
        let mut next_header = data_start + filesize;
        // Align to 4 bytes
        while next_header % 4 != 0 {
            next_header += 1;
        }
        self.offset = next_header;

        Some(CpioEntry {
            name,
            data: file_data,
        })
    }
}

// ============================================================================
// Unit Tests - TEAM_039
// ============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    /// Tests: [CP1] "070701" magic, [CP2] "070702" magic
    #[test]
    fn test_cpio_header_valid_magic() {
        // [CP1] Valid newc format
        let mut header = CpioHeader {
            c_magic: *b"070701",
            c_ino: [0; 8],
            c_mode: [0; 8],
            c_uid: [0; 8],
            c_gid: [0; 8],
            c_nlink: [0; 8],
            c_mtime: [0; 8],
            c_filesize: [0; 8],
            c_devmajor: [0; 8],
            c_devminor: [0; 8],
            c_rdevmajor: [0; 8],
            c_rdevminor: [0; 8],
            c_namesize: [0; 8],
            c_check: [0; 8],
        };
        assert!(header.is_valid()); // [CP1]

        // [CP2] Valid newc CRC format
        header.c_magic = *b"070702";
        assert!(header.is_valid()); // [CP2]
    }

    /// Tests: [CP3] Invalid magic rejected
    #[test]
    fn test_cpio_header_invalid_magic() {
        let header = CpioHeader {
            c_magic: *b"BADMAG",
            c_ino: [0; 8],
            c_mode: [0; 8],
            c_uid: [0; 8],
            c_gid: [0; 8],
            c_nlink: [0; 8],
            c_mtime: [0; 8],
            c_filesize: [0; 8],
            c_devmajor: [0; 8],
            c_devminor: [0; 8],
            c_rdevmajor: [0; 8],
            c_rdevminor: [0; 8],
            c_namesize: [0; 8],
            c_check: [0; 8],
        };
        assert!(!header.is_valid()); // [CP3]
    }

    /// Tests: [CP4] Hex string parsing
    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex(b"00000000"), 0); // [CP4]
        assert_eq!(parse_hex(b"0000000A"), 10); // [CP4]
        assert_eq!(parse_hex(b"000000FF"), 255); // [CP4]
        assert_eq!(parse_hex(b"00001000"), 4096); // [CP4]
    }

    /// Tests: [CP5] Invalid hex returns 0
    #[test]
    fn test_parse_hex_invalid() {
        assert_eq!(parse_hex(b"ZZZZZZZZ"), 0); // [CP5] invalid chars
        assert_eq!(parse_hex(&[0xFF; 8]), 0); // [CP5] non-UTF8
    }

    // Helper to create a minimal CPIO entry
    // Header (110 bytes) + name (aligned to 4) + data (aligned to 4)
    fn make_cpio_entry(name: &str, data: &[u8]) -> Vec<u8> {
        let namesize = name.len() + 1; // include null terminator
        let filesize = data.len();

        // Build header
        let mut entry = Vec::new();
        entry.extend_from_slice(b"070701"); // magic
        entry.extend_from_slice(b"00000001"); // ino
        entry.extend_from_slice(b"000081A4"); // mode (regular file)
        entry.extend_from_slice(b"00000000"); // uid
        entry.extend_from_slice(b"00000000"); // gid
        entry.extend_from_slice(b"00000001"); // nlink
        entry.extend_from_slice(b"00000000"); // mtime
        entry.extend_from_slice(format!("{:08X}", filesize).as_bytes()); // filesize
        entry.extend_from_slice(b"00000000"); // devmajor
        entry.extend_from_slice(b"00000000"); // devminor
        entry.extend_from_slice(b"00000000"); // rdevmajor
        entry.extend_from_slice(b"00000000"); // rdevminor
        entry.extend_from_slice(format!("{:08X}", namesize).as_bytes()); // namesize
        entry.extend_from_slice(b"00000000"); // check

        // Add name with null terminator
        entry.extend_from_slice(name.as_bytes());
        entry.push(0); // null terminator

        // Pad to 4-byte alignment
        while entry.len() % 4 != 0 {
            entry.push(0);
        }

        // Add data
        entry.extend_from_slice(data);

        // Pad data to 4-byte alignment
        while entry.len() % 4 != 0 {
            entry.push(0);
        }

        entry
    }

    fn make_trailer() -> Vec<u8> {
        make_cpio_entry("TRAILER!!!", &[])
    }

    /// Tests: [CP6] CpioArchive::iter returns entries in order
    #[test]
    fn test_cpio_iter_order() {
        let mut archive_data = Vec::new();
        archive_data.extend(make_cpio_entry("first.txt", b"first"));
        archive_data.extend(make_cpio_entry("second.txt", b"second"));
        archive_data.extend(make_cpio_entry("third.txt", b"third"));
        archive_data.extend(make_trailer());

        let archive = CpioArchive::new(&archive_data);
        let entries: Vec<_> = archive.iter().collect();

        assert_eq!(entries.len(), 3); // [CP6]
        assert_eq!(entries[0].name, "first.txt"); // [CP6] order preserved
        assert_eq!(entries[1].name, "second.txt");
        assert_eq!(entries[2].name, "third.txt");
    }

    /// Tests: [CP7] Iterator stops at TRAILER!!!
    #[test]
    fn test_cpio_iter_trailer() {
        let mut archive_data = Vec::new();
        archive_data.extend(make_cpio_entry("file.txt", b"content"));
        archive_data.extend(make_trailer());
        // Add garbage after trailer - should NOT be read
        archive_data.extend(make_cpio_entry("hidden.txt", b"hidden"));

        let archive = CpioArchive::new(&archive_data);
        let entries: Vec<_> = archive.iter().collect();

        assert_eq!(entries.len(), 1); // [CP7] stops at trailer
        assert_eq!(entries[0].name, "file.txt");
    }

    /// Tests: [CP8] CpioArchive::get_file finds existing file
    #[test]
    fn test_cpio_get_file_found() {
        let mut archive_data = Vec::new();
        archive_data.extend(make_cpio_entry("hello.txt", b"Hello World"));
        archive_data.extend(make_cpio_entry("other.txt", b"Other"));
        archive_data.extend(make_trailer());

        let archive = CpioArchive::new(&archive_data);
        let result = archive.get_file("hello.txt");

        assert!(result.is_some()); // [CP8]
        assert_eq!(result.unwrap(), b"Hello World");
    }

    /// Tests: [CP9] CpioArchive::get_file returns None for missing file
    #[test]
    fn test_cpio_get_file_missing() {
        let mut archive_data = Vec::new();
        archive_data.extend(make_cpio_entry("exists.txt", b"data"));
        archive_data.extend(make_trailer());

        let archive = CpioArchive::new(&archive_data);
        let result = archive.get_file("missing.txt");

        assert!(result.is_none()); // [CP9]
    }

    /// Tests: [CP10] 4-byte alignment is applied after header+name
    #[test]
    fn test_cpio_alignment() {
        // Create entry with name that requires padding
        // "ab" = 2 chars + 1 null = 3 bytes, header = 110, total = 113
        // Needs 3 bytes padding to reach 116 (divisible by 4)
        let mut archive_data = Vec::new();
        archive_data.extend(make_cpio_entry("ab", b"XY")); // 2-char name
        archive_data.extend(make_trailer());

        let archive = CpioArchive::new(&archive_data);
        let entries: Vec<_> = archive.iter().collect();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "ab");
        assert_eq!(entries[0].data, b"XY"); // [CP10] data correctly located after alignment
    }
}
