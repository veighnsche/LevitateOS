use fdt::Fdt;

/// Errors that can occur during FDT parsing
#[derive(Debug)]
pub enum FdtError {
    InvalidHeader,
    InitrdMissing,
}

/// Retrieve the physical address range of the initrd from the DTB.
///
/// # Arguments
///
/// * `dtb_ptr` - Virtual pointer to the simple slice containing the DTB.
///
/// # Returns
///
/// * `Option<(usize, usize)>` - Start and End physical addresses of the initrd.
pub fn get_initrd_range(dtb_data: &[u8]) -> Result<(usize, usize), FdtError> {
    let fdt = Fdt::new(dtb_data).map_err(|_| FdtError::InvalidHeader)?;

    let chosen = fdt.chosen();

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
                    // Can be 32-bit or 64-bit depending on #address-cells.
                    // fdt crate usually handles endianness in `as_usize()`.
                    // Actually `prop.value` is &[u8]. Fdt crate hasn't got `as_usize`.
                    // But let's check if the property wrapper has helpers.
                    // Checking 0.1.5 source/docs... it seems `Property` has `as_usize`?
                    // No, let's use manual parsing which is safer.
                    if prop.value.len() == 4 {
                        start = Some(u32::from_be_bytes(prop.value.try_into().unwrap()) as usize);
                    } else if prop.value.len() == 8 {
                        start = Some(u64::from_be_bytes(prop.value.try_into().unwrap()) as usize);
                    }
                }
                "linux,initrd-end" => {
                    if prop.value.len() == 4 {
                        end = Some(u32::from_be_bytes(prop.value.try_into().unwrap()) as usize);
                    } else if prop.value.len() == 8 {
                        end = Some(u64::from_be_bytes(prop.value.try_into().unwrap()) as usize);
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
