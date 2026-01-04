//! Hex Formatting Utilities
//!
//! TEAM_039: Relocated from levitate-hal/src/console.rs for testability
//!
//! Pure functions for converting values to hexadecimal strings.

/// [C4] Nibble 0-9 maps to '0'-'9', [C5] Nibble 10-15 maps to 'a'-'f'
#[inline]
#[must_use]
pub fn nibble_to_hex(nibble: u8) -> char {
    if nibble < 10 {
        (b'0' + nibble) as char // [C4]
    } else {
        (b'a' + (nibble - 10)) as char // [C5]
    }
}

/// [C1] Converts 0 correctly, [C2] Converts max u64, [C3] Handles mixed nibbles
#[must_use]
pub fn format_hex(val: u64, buf: &mut [u8; 18]) -> &str {
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nibble = ((val >> ((15 - i) * 4)) & 0xf) as u8;
        buf[2 + i] = nibble_to_hex(nibble) as u8; // [C1][C2][C3]
    }
    // SAFETY: buf contains only ASCII hex chars ('0'-'9', 'a'-'f', 'x'), always valid UTF-8
    unsafe { core::str::from_utf8_unchecked(&buf[..]) }
}

// ============================================================================
// Unit Tests - TEAM_039: Relocated from levitate-hal/src/console.rs
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // C4: Nibble 0-9 maps to '0'-'9'
    #[test]
    fn test_nibble_to_hex_digits() {
        assert_eq!(nibble_to_hex(0), '0');
        assert_eq!(nibble_to_hex(1), '1');
        assert_eq!(nibble_to_hex(5), '5');
        assert_eq!(nibble_to_hex(9), '9');
    }

    // C5: Nibble 10-15 maps to 'a'-'f'
    #[test]
    fn test_nibble_to_hex_letters() {
        assert_eq!(nibble_to_hex(10), 'a');
        assert_eq!(nibble_to_hex(11), 'b');
        assert_eq!(nibble_to_hex(12), 'c');
        assert_eq!(nibble_to_hex(13), 'd');
        assert_eq!(nibble_to_hex(14), 'e');
        assert_eq!(nibble_to_hex(15), 'f');
    }

    // C1: format_hex converts 0 to "0x0000000000000000"
    #[test]
    fn test_format_hex_zero() {
        let mut buf = [0u8; 18];
        let result = format_hex(0, &mut buf);
        assert_eq!(result, "0x0000000000000000");
    }

    // C2: format_hex converts max u64 correctly
    #[test]
    fn test_format_hex_max() {
        let mut buf = [0u8; 18];
        let result = format_hex(u64::MAX, &mut buf);
        assert_eq!(result, "0xffffffffffffffff");
    }

    // C3: format_hex handles mixed nibble values
    #[test]
    fn test_format_hex_mixed() {
        let mut buf = [0u8; 18];
        let result = format_hex(0x0123456789abcdef, &mut buf);
        assert_eq!(result, "0x0123456789abcdef");
    }
}
