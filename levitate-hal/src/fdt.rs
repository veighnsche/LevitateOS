use fdt::Fdt;

/// Errors that can occur during FDT parsing
/// [FD1] InvalidHeader - DTB header is malformed
/// [FD2] InitrdMissing - No initrd properties found
#[derive(Debug)]
pub enum FdtError {
    /// [FD1] Invalid DTB header
    InvalidHeader,
    /// [FD2] Missing initrd properties, [FD5] Both start and end must exist
    InitrdMissing,
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
