//! `TEAM_387`: Debug calculator for kernel development math
//!
//! Quick calculations for memory sizes, addresses, allocations, and bit operations.

use anyhow::{bail, Result};
use bytesize::ByteSize;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CalcCommands {
    /// Convert/display memory size (e.g., "9346984" or "8MB")
    Size { value: String },

    /// Analyze address (page offset, alignment, etc.)
    Addr { value: String },

    /// Buddy allocator: find order for allocation size
    Buddy {
        /// Size in bytes (e.g., "9346984" or "8MB")
        size: String,
        /// Page size (default 4096)
        #[arg(long, default_value = "4096")]
        page_size: u64,
    },

    /// Bit operations on a value
    Bits { value: String },

    /// Alignment calculator
    Align {
        /// Value to align
        value: String,
        /// Alignment boundary (default 4096)
        #[arg(long, default_value = "4096")]
        to: u64,
    },

    /// Interactive mode (eval expressions)
    Eval { expr: String },
}

pub fn run(cmd: CalcCommands) -> Result<()> {
    match cmd {
        CalcCommands::Size { value } => size_calc(&value),
        CalcCommands::Addr { value } => addr_calc(&value),
        CalcCommands::Buddy { size, page_size } => buddy_calc(&size, page_size),
        CalcCommands::Bits { value } => bits_calc(&value),
        CalcCommands::Align { value, to } => align_calc(&value, to),
        CalcCommands::Eval { expr } => eval_expr(&expr),
    }
}

/// Parse a size string like "9346984", "8MB", "4KiB", "0x1000"
/// Uses bytesize crate for robust parsing of human-readable sizes.
fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();

    // Handle hex first (bytesize doesn't support hex)
    if s.starts_with("0x") || s.starts_with("0X") {
        return Ok(u64::from_str_radix(&s[2..], 16)?);
    }

    // Try bytesize parsing (handles "8MB", "4KiB", "1.5GB", etc.)
    if let Ok(bs) = s.parse::<ByteSize>() {
        return Ok(bs.as_u64());
    }

    // Fall back to plain number
    Ok(s.parse()?)
}

/// Parse an address (hex or decimal)
fn parse_addr(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        Ok(u64::from_str_radix(&s[2..], 16)?)
    } else {
        Ok(s.parse()?)
    }
}

fn size_calc(value: &str) -> Result<()> {
    let bytes = parse_size(value)?;
    let bs = ByteSize::b(bytes);

    println!(
        "{} = {} (0x{:X}) = {} = {} 4K pages",
        value,
        bytes,
        bytes,
        bs.to_string_as(true),
        bytes.div_ceil(4096)
    );

    Ok(())
}

fn addr_calc(value: &str) -> Result<()> {
    let addr = parse_addr(value)?;

    let page_4k = addr >> 12;
    let off_4k = addr & 0xFFF;
    let pml4 = (addr >> 39) & 0x1FF;
    let pdpt = (addr >> 30) & 0x1FF;
    let pd = (addr >> 21) & 0x1FF;
    let pt = (addr >> 12) & 0x1FF;
    let canonical = if addr & (1 << 47) != 0 {
        (addr >> 48) == 0xFFFF
    } else {
        (addr >> 48) == 0
    };
    let page_aligned = (addr & 0xFFF) == 0;
    let large_page_aligned = (addr & 0x001F_FFFF) == 0;

    println!("0x{addr:016X} ({addr})");
    println!("  4K: page={page_4k} off=0x{off_4k:03X} | PT[{pml4}][{pdpt}][{pd}][{pt}]");
    println!("  canonical={canonical} 4K-aligned={page_aligned} 2M-aligned={large_page_aligned}");

    Ok(())
}

fn buddy_calc(size: &str, page_size: u64) -> Result<()> {
    let bytes = parse_size(size)?;
    let bs = ByteSize::b(bytes);

    let pages_needed = bytes.div_ceil(page_size);
    let order = (pages_needed as f64).log2().ceil() as u32;
    let actual_pages = 1u64 << order;
    let actual_bytes = actual_pages * page_size;
    let waste = actual_bytes - bytes;
    let waste_pct = (waste as f64 / actual_bytes as f64) * 100.0;

    println!(
        "{} ({}) -> order={} (2^{}={} pages, {} actual, {:.0}% waste)",
        bytes,
        bs.to_string_as(true),
        order,
        order,
        actual_pages,
        ByteSize::b(actual_bytes).to_string_as(true),
        waste_pct
    );

    Ok(())
}

fn bits_calc(value: &str) -> Result<()> {
    let num = parse_addr(value)?;

    let set_bits: Vec<u32> = (0..64).filter(|i| (num >> i) & 1 == 1).collect();
    let is_pow2 = num != 0 && num.is_power_of_two();

    println!("{num} = 0x{num:X} = 0b{num:b}");
    println!(
        "  bits set: {} | leading 0s: {} | trailing 0s: {}{}",
        set_bits.len(),
        num.leading_zeros(),
        if num == 0 { 64 } else { num.trailing_zeros() },
        if is_pow2 {
            format!(" | 2^{}", num.trailing_zeros())
        } else {
            String::new()
        }
    );

    if !set_bits.is_empty() && set_bits.len() <= 16 {
        let positions: String = set_bits
            .iter()
            .rev()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        println!("  positions: [{positions}]");
    }

    Ok(())
}

fn align_calc(value: &str, to: u64) -> Result<()> {
    let num = parse_addr(value)?;

    if to == 0 || (to & (to - 1)) != 0 {
        bail!("Alignment must be a power of 2, got {to}");
    }

    let down = num & !(to - 1);
    let up = (num + to - 1) & !(to - 1);
    let off = num - down;

    println!(
        "0x{:X} align({}) -> down=0x{:X} up=0x{:X} off={} aligned={}",
        num,
        to,
        down,
        up,
        off,
        num == down
    );

    Ok(())
}

fn eval_expr(expr: &str) -> Result<()> {
    // Simple expression evaluator for common operations
    let expr = expr.trim();

    // Handle simple operations
    if let Some((a, b)) = expr.split_once('+') {
        let a = parse_size(a.trim())?;
        let b = parse_size(b.trim())?;
        let result = a.wrapping_add(b);
        println!("  {a} + {b} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('-') {
        let a = parse_size(a.trim())?;
        let b = parse_size(b.trim())?;
        let result = a.wrapping_sub(b);
        println!("  {a} - {b} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('*') {
        let a = parse_size(a.trim())?;
        let b = parse_size(b.trim())?;
        let result = a.wrapping_mul(b);
        println!("  {a} * {b} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('/') {
        let a = parse_size(a.trim())?;
        let b = parse_size(b.trim())?;
        if b == 0 {
            bail!("Division by zero");
        }
        let result = a / b;
        let remainder = a % b;
        println!("  {a} / {b} = {result} remainder {remainder} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once("<<") {
        let a = parse_size(a.trim())?;
        let b: u32 = b.trim().parse()?;
        let result = a << b;
        println!("  {a} << {b} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once(">>") {
        let a = parse_size(a.trim())?;
        let b: u32 = b.trim().parse()?;
        let result = a >> b;
        println!("  {a} >> {b} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('&') {
        let a = parse_addr(a.trim())?;
        let b = parse_addr(b.trim())?;
        let result = a & b;
        println!("  0x{a:X} & 0x{b:X} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('|') {
        let a = parse_addr(a.trim())?;
        let b = parse_addr(b.trim())?;
        let result = a | b;
        println!("  0x{a:X} | 0x{b:X} = {result} (0x{result:X})");
        return Ok(());
    }
    if let Some((a, b)) = expr.split_once('^') {
        let a = parse_addr(a.trim())?;
        let b = parse_addr(b.trim())?;
        let result = a ^ b;
        println!("  0x{a:X} ^ 0x{b:X} = {result} (0x{result:X})");
        return Ok(());
    }
    if expr.starts_with('~') {
        let a = parse_addr(expr[1..].trim())?;
        let result = !a;
        println!("  ~0x{a:X} = {result} (0x{result:X})");
        return Ok(());
    }

    // Just parse and display
    let val = parse_size(expr)?;
    println!("  {expr} = {val} (0x{val:X})");

    Ok(())
}
