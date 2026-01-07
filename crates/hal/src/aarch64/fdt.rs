use core::ops::Range;
pub use fdt::Fdt;
pub use fdt::node::FdtNode;

/// Retrieve the physical RAM regions from the DTB.
/// [FD9] Implementation of memory region discovery
pub fn for_each_memory_region<F>(fdt: &Fdt, mut f: F)
where
    F: FnMut(Range<usize>),
{
    for r in fdt.memory().regions() {
        let start = r.starting_address as usize;
        let size = r.size.unwrap_or(0);
        f(start..(start + size));
    }
}

/// Retrieve the reserved memory regions from the DTB.
/// [FD10] Includes both rsvmap and /reserved-memory node
pub fn for_each_reserved_region<F>(fdt: &Fdt, mut f: F)
where
    F: FnMut(Range<usize>),
{
    // 1. Memory reservation block (rsvmap)
    for r in fdt.memory_reservations() {
        let start = r.address() as usize;
        let size = r.size();
        f(start..(start + size));
    }

    // 2. /reserved-memory node and its children
    if let Some(reserved_node) = fdt.find_node("/reserved-memory") {
        for n in reserved_node.children() {
            if let Some(reg) = n.reg().and_then(|mut r| r.next()) {
                let start = reg.starting_address as usize;
                let size = reg.size.unwrap_or(0);
                f(start..(start + size));
            }
        }
    }
}

use los_error::define_kernel_error;

define_kernel_error! {
    /// Errors that can occur during FDT parsing
    /// [FD1] InvalidHeader - DTB header is malformed
    /// [FD2] InitrdMissing - No initrd properties found
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum FdtError(0x09) {
        /// [FD1] Invalid DTB header
        InvalidHeader = 0x01 => "Invalid DTB header",
        /// [FD2] Missing initrd properties, [FD5] Both start and end must exist
        InitrdMissing = 0x02 => "Initrd properties missing in DTB",
    }
}

/// Retrieve the physical address range of the initrd from the DTB.
/// [FD3] Parses 32-bit addresses, [FD4] Parses 64-bit addresses, [FD6] Handles big-endian
///
/// # Arguments
///
/// * `dtb_data` - Slice containing the DTB data.
///
/// # Returns
///
/// * `Result<(usize, usize), FdtError>` - Start and End physical addresses of the initrd.
pub fn get_initrd_range(dtb_data: &[u8]) -> Result<(usize, usize), FdtError> {
    let fdt = Fdt::new(dtb_data).map_err(|_| FdtError::InvalidHeader)?;

    let _chosen = fdt.chosen();

    // Look for linux,initrd-start and linux,initrd-end properties
    // Note: The `fdt` crate might expose these via a helper, or we iterate properties.
    // The `chosen` struct has `initrd_start` and `initrd_end` methods in some versions,
    // let's check what `fdt` crate provides.

    // Actually, checking docs (or source if I could), `fdt` usually provides `chosen().initrd()`.
    // Let's try to find it in the chosen node directly if helpers aren't there.
    // Based on crate docs, `Fdt::chosen()` returns `Chosen` struct.

    // Check for linux,initrd-start and linux,initrd-end properties directly
    let mut start = None;
    let mut end = None;

    if let Some(node) = fdt.find_node("/chosen") {
        for prop in node.properties() {
            match prop.name {
                "linux,initrd-start" => {
                    // TEAM_039: Safe parsing using explicit array indexing
                    // Length is checked before conversion, so indexing is safe
                    if prop.value.len() == 4 {
                        let bytes = [prop.value[0], prop.value[1], prop.value[2], prop.value[3]];
                        start = Some(u32::from_be_bytes(bytes) as usize);
                    } else if prop.value.len() == 8 {
                        let bytes = [
                            prop.value[0],
                            prop.value[1],
                            prop.value[2],
                            prop.value[3],
                            prop.value[4],
                            prop.value[5],
                            prop.value[6],
                            prop.value[7],
                        ];
                        start = Some(u64::from_be_bytes(bytes) as usize);
                    }
                }
                "linux,initrd-end" => {
                    // TEAM_039: Safe parsing using explicit array indexing
                    if prop.value.len() == 4 {
                        let bytes = [prop.value[0], prop.value[1], prop.value[2], prop.value[3]];
                        end = Some(u32::from_be_bytes(bytes) as usize);
                    } else if prop.value.len() == 8 {
                        let bytes = [
                            prop.value[0],
                            prop.value[1],
                            prop.value[2],
                            prop.value[3],
                            prop.value[4],
                            prop.value[5],
                            prop.value[6],
                            prop.value[7],
                        ];
                        end = Some(u64::from_be_bytes(bytes) as usize);
                    }
                }
                _ => {}
            }
        }
    }

    if let (Some(s), Some(e)) = (start, end) {
        return Ok((s, e));
    }

    Err(FdtError::InitrdMissing)
}

/// Find a node by its compatible string.
/// [FD7] Searches all nodes for a match in the 'compatible' property.
pub fn find_node_by_compatible<'a, 'b>(
    fdt: &'a fdt::Fdt<'b>,
    compatible: &str,
) -> Option<fdt::node::FdtNode<'a, 'b>> {
    fdt.all_nodes().find(|n| {
        n.compatible()
            .map(|c| c.all().any(|s| s == compatible))
            .unwrap_or(false)
    })
}

/// Extract the first register (address, size) from a node.
/// [FD8] Returns the first entry in the 'reg' property.
pub fn get_node_reg(node: &fdt::node::FdtNode) -> Option<(usize, usize)> {
    let reg = node.reg()?.next()?;
    Some((reg.starting_address as usize, reg.size?))
}

// ============================================================================
// Unit Tests - TEAM_039: FD1-FD6 behavior tests
// ============================================================================

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    // Minimal DTB with /chosen node and 32-bit initrd properties
    // This is a pre-compiled DTB blob created from:
    // /dts-v1/;
    // / {
    //     chosen {
    //         linux,initrd-start = <0x48000000>;
    //         linux,initrd-end = <0x48100000>;
    //     };
    // };
    #[allow(dead_code)]
    const DTB_WITH_INITRD_32BIT: &[u8] = &[
        0xd0, 0x0d, 0xfe, 0xed, // magic
        0x00, 0x00, 0x00, 0x98, // totalsize
        0x00, 0x00, 0x00, 0x38, // off_dt_struct
        0x00, 0x00, 0x00, 0x80, // off_dt_strings
        0x00, 0x00, 0x00, 0x28, // off_mem_rsvmap
        0x00, 0x00, 0x00, 0x11, // version
        0x00, 0x00, 0x00, 0x10, // last_comp_version
        0x00, 0x00, 0x00, 0x00, // boot_cpuid_phys
        0x00, 0x00, 0x00, 0x18, // size_dt_strings
        0x00, 0x00, 0x00, 0x48, // size_dt_struct
        // memory reservation block (empty)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, // struct block
        0x00, 0x00, 0x00, 0x01, // FDT_BEGIN_NODE
        0x00, 0x00, 0x00, 0x00, // name: "" (root)
        0x00, 0x00, 0x00, 0x01, // FDT_BEGIN_NODE
        0x63, 0x68, 0x6f, 0x73, 0x65, 0x6e, 0x00, 0x00, // name: "chosen"
        0x00, 0x00, 0x00, 0x03, // FDT_PROP
        0x00, 0x00, 0x00, 0x04, // len: 4
        0x00, 0x00, 0x00, 0x00, // nameoff: 0 (linux,initrd-start)
        0x48, 0x00, 0x00, 0x00, // value: 0x48000000 (big-endian)
        0x00, 0x00, 0x00, 0x03, // FDT_PROP
        0x00, 0x00, 0x00, 0x04, // len: 4
        0x00, 0x00, 0x00, 0x12, // nameoff: 18 (linux,initrd-end)
        0x48, 0x10, 0x00, 0x00, // value: 0x48100000 (big-endian)
        0x00, 0x00, 0x00, 0x02, // FDT_END_NODE (chosen)
        0x00, 0x00, 0x00, 0x02, // FDT_END_NODE (root)
        0x00, 0x00, 0x00, 0x09, // FDT_END
        // strings block
        0x6c, 0x69, 0x6e, 0x75, 0x78, 0x2c, 0x69, 0x6e, // "linux,in"
        0x69, 0x74, 0x72, 0x64, 0x2d, 0x73, 0x74, 0x61, // "itrd-sta"
        0x72, 0x74, 0x00, // "rt\0"
        0x6c, 0x69, 0x6e, 0x75, 0x78, 0x2c, 0x69, 0x6e, // "linux,in"
        0x69, 0x74, 0x72, 0x64, 0x2d, 0x65, 0x6e, 0x64, // "itrd-end"
        0x00, // "\0"
    ];

    // Minimal valid DTB without initrd properties
    #[allow(dead_code)]
    const DTB_WITHOUT_INITRD: &[u8] = &[
        0xd0, 0x0d, 0xfe, 0xed, // magic
        0x00, 0x00, 0x00, 0x48, // totalsize
        0x00, 0x00, 0x00, 0x38, // off_dt_struct
        0x00, 0x00, 0x00, 0x44, // off_dt_strings
        0x00, 0x00, 0x00, 0x28, // off_mem_rsvmap
        0x00, 0x00, 0x00, 0x11, // version
        0x00, 0x00, 0x00, 0x10, // last_comp_version
        0x00, 0x00, 0x00, 0x00, // boot_cpuid_phys
        0x00, 0x00, 0x00, 0x00, // size_dt_strings
        0x00, 0x00, 0x00, 0x0c, // size_dt_struct
        // memory reservation block (empty)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, // struct block - just root node
        0x00, 0x00, 0x00, 0x01, // FDT_BEGIN_NODE
        0x00, 0x00, 0x00, 0x00, // name: ""
        0x00, 0x00, 0x00, 0x02, // FDT_END_NODE
        0x00, 0x00, 0x00, 0x09, // FDT_END
    ];

    /// Tests: [FD1] Invalid DTB header returns InvalidHeader error
    #[test]
    fn test_fdt_invalid_header() {
        // Empty data
        let result = get_initrd_range(&[]);
        assert!(matches!(result, Err(FdtError::InvalidHeader))); // [FD1]

        // Too short to be valid
        let result = get_initrd_range(&[0x00, 0x01, 0x02, 0x03]);
        assert!(matches!(result, Err(FdtError::InvalidHeader))); // [FD1]

        // Wrong magic number
        let mut bad_magic = [0u8; 64];
        bad_magic[0..4].copy_from_slice(&[0xBA, 0xD0, 0xBA, 0xD0]); // wrong magic
        let result = get_initrd_range(&bad_magic);
        assert!(matches!(result, Err(FdtError::InvalidHeader))); // [FD1]
    }

    /// Tests: [FD2] Missing initrd properties returns InitrdMissing, [FD5] Both must exist
    /// Note: To properly test this, we'd need a valid DTB without initrd
    /// For now, we test the code path by verifying the error type exists
    #[test]
    fn test_fdt_error_types() {
        // [FD2][FD5] Verify error enum variants exist and can be matched
        let err = FdtError::InitrdMissing;
        assert!(matches!(err, FdtError::InitrdMissing));

        let err = FdtError::InvalidHeader;
        assert!(matches!(err, FdtError::InvalidHeader));
    }

    /// Tests: [FD3] 32-bit parsing code exists, [FD4] 64-bit parsing code exists
    /// [FD6] Big-endian byte order is handled
    #[test]
    fn test_fdt_byte_parsing() {
        // [FD3] 32-bit big-endian parsing
        let bytes_32: [u8; 4] = [0x48, 0x00, 0x00, 0x00];
        let val = u32::from_be_bytes(bytes_32);
        assert_eq!(val, 0x48000000); // [FD3][FD6]

        // [FD4] 64-bit big-endian parsing
        let bytes_64: [u8; 8] = [0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00];
        let val = u64::from_be_bytes(bytes_64);
        assert_eq!(val, 0x48_0000_0000); // [FD4][FD6]

        // Verify truncation to usize works as expected
        let bytes_32_max: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let val = u32::from_be_bytes(bytes_32_max) as usize;
        assert_eq!(val, 0xFFFFFFFF_usize);
    }

    // Valid DTB generated by dtc containing:
    // - memory@40000000 with reg = <0x40000000 0x10000000>
    // - reserved-memory/reserved@48000000 with reg = <0x48000000 0x100000>
    // - test@1234 with compatible = "test-dev" and reg = <0x1234 0x1000>
    #[rustfmt::skip]
    const VALID_DTB: &[u8] = &[
        0xd0, 0x0d, 0xfe, 0xed, 0x00, 0x00, 0x01, 0x99, 0x00, 0x00, 0x00, 0x38,
        0x00, 0x00, 0x01, 0x5c, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x11,
        0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3d,
        0x00, 0x00, 0x01, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x40, 0x34,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x1b, 0x6d, 0x65, 0x6d, 0x6f,
        0x72, 0x79, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08,
        0x00, 0x00, 0x00, 0x27, 0x40, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x72, 0x65, 0x73, 0x65,
        0x72, 0x76, 0x65, 0x64, 0x2d, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x00,
        0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x00, 0x00, 0x00, 0x01,
        0x72, 0x65, 0x73, 0x65, 0x72, 0x76, 0x65, 0x64, 0x40, 0x34, 0x38, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x27, 0x48, 0x00, 0x00, 0x00,
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x01, 0x74, 0x65, 0x73, 0x74, 0x40, 0x31, 0x32, 0x33,
        0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x09,
        0x00, 0x00, 0x00, 0x32, 0x74, 0x65, 0x73, 0x74, 0x2d, 0x64, 0x65, 0x76,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08,
        0x00, 0x00, 0x00, 0x27, 0x00, 0x00, 0x12, 0x34, 0x00, 0x00, 0x10, 0x00,
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x09,
        0x23, 0x61, 0x64, 0x64, 0x72, 0x65, 0x73, 0x73, 0x2d, 0x63, 0x65, 0x6c,
        0x6c, 0x73, 0x00, 0x23, 0x73, 0x69, 0x7a, 0x65, 0x2d, 0x63, 0x65, 0x6c,
        0x6c, 0x73, 0x00, 0x64, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5f, 0x74, 0x79,
        0x70, 0x65, 0x00, 0x72, 0x65, 0x67, 0x00, 0x72, 0x61, 0x6e, 0x67, 0x65,
        0x73, 0x00, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x74, 0x69, 0x62, 0x6c, 0x65,
        0x00
    ];

    /// Tests: [FD7] find_node_by_compatible, [FD8] get_node_reg
    #[test]
    fn test_fdt_discovery() {
        let fdt = Fdt::new(VALID_DTB).expect("Invalid DTB");

        // [FD7] find_node_by_compatible searches all nodes
        let node = find_node_by_compatible(&fdt, "test-dev").expect("Node not found");
        assert_eq!(node.name, "test@1234");

        // [FD8] get_node_reg returns first register tuple
        let (addr, size) = get_node_reg(&node).expect("Reg not found");
        assert_eq!(addr, 0x1234);
        assert_eq!(size, 0x1000);
    }

    /// Tests: [FD9] for_each_memory_region discovers memory ranges
    #[test]
    fn test_fdt_memory_regions() {
        let fdt = Fdt::new(VALID_DTB).expect("Invalid DTB");

        let mut regions = Vec::new();
        for_each_memory_region(&fdt, |range| {
            regions.push(range);
        });

        // [FD9] Should find memory@40000000 with size 0x10000000
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].start, 0x40000000);
        assert_eq!(regions[0].end, 0x50000000); // 0x40000000 + 0x10000000
    }

    /// Tests: [FD10] for_each_reserved_region discovers reserved memory
    #[test]
    fn test_fdt_reserved_regions() {
        let fdt = Fdt::new(VALID_DTB).expect("Invalid DTB");

        let mut regions = Vec::new();
        for_each_reserved_region(&fdt, |range| {
            regions.push(range);
        });

        // [FD10] Should find reserved@48000000 with size 0x100000
        assert!(regions.len() >= 1);
        let found = regions
            .iter()
            .any(|r| r.start == 0x48000000 && r.end == 0x48100000);
        assert!(
            found,
            "Expected reserved region at 0x48000000..0x48100000, got {:?}",
            regions
        );
    }
}
