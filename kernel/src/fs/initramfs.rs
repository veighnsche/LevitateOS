use core::str;

/// CPIO New ASCII Format Header
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
    pub fn is_valid(&self) -> bool {
        &self.c_magic == b"070701" || &self.c_magic == b"070702"
    }

    pub fn filesize(&self) -> usize {
        parse_hex(&self.c_filesize)
    }

    pub fn namesize(&self) -> usize {
        parse_hex(&self.c_namesize)
    }
}

fn parse_hex(bytes: &[u8]) -> usize {
    let s = str::from_utf8(bytes).unwrap_or("0");
    usize::from_str_radix(s, 16).unwrap_or(0)
}

pub struct CpioArchive {
    data: &'static [u8],
}

impl CpioArchive {
    /// Create a new CPIO archive wrapper around a memory slice.
    pub fn new(data: &'static [u8]) -> Self {
        Self { data }
    }

    pub fn iter(&self) -> CpioIterator {
        CpioIterator {
            data: self.data,
            offset: 0,
        }
    }

    pub fn get_file(&self, path: &str) -> Option<&'static [u8]> {
        for entry in self.iter() {
            if entry.name == path {
                return Some(entry.data);
            }
        }
        None
    }
}

pub struct CpioEntry {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub struct CpioIterator {
    data: &'static [u8],
    offset: usize,
}

impl Iterator for CpioIterator {
    type Item = CpioEntry;

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

        if name == "TRAILER!!!" {
            return None;
        }

        // Calculate Data Offset (4-byte aligned after header + name)
        let mut data_start = name_start + namesize;
        // Align to 4 bytes
        while data_start % 4 != 0 {
            data_start += 1;
        }

        if data_start + filesize > self.data.len() {
            return None;
        }

        let file_data = &self.data[data_start..data_start + filesize];

        // Prepare next offset
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
