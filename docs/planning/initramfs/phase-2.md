# Initramfs - Phase 2: Design

## 1. Proposed Solution
The kernel will store the Devicetree Blob (DTB) address passed by the bootloader. A dedicated module will parse the DTB to find the location and size of the initramfs. A CPIO parser will then allow the kernel to access files within the initramfs.

### 1.1 DTB Preservation
- Modify `kernel/src/main.rs` assembly to save `x0` into a `static mut BOOT_DTB_ADDR: usize`.
- This happens before `x0` is overwritten.

### 1.2 DTB Parsing
- Use the `fdt` crate to parse the DTB.
- The `fdt` module will be located in `levitate-hal` or as a new utility in `kernel`.
- Key discovery: `linux,initrd-start` and `linux,initrd-end` in the `/chosen` node.

### 1.3 CPIO Parsing
- Implement a minimal CPIO New ASCII format parser ($newc$).
- The parser will provide an iterator over files in the initramfs.
- Files will be represented as `(name, data_ptr, size)`.

### 1.4 Virtual File System (VFS) Integration
- Initially, the initramfs will be accessible via a `fs::initramfs` module.
- It will eventually be mounted into a Global VFS (future phase).

## 2. API Design

### 2.1 Boot Integration
```rust
// In kernel/src/main.rs
static mut BOOT_DTB_ADDR: usize = 0;

// Accessor
pub fn get_dtb_phys() -> Option<usize> {
    let addr = unsafe { BOOT_DTB_ADDR };
    if addr == 0 { None } else { Some(addr) }
}
```

### 2.2 Initramfs Parser
```rust
pub struct Initramfs {
    data: &'static [u8],
}

impl Initramfs {
    pub fn new(start: usize, end: usize) -> Self;
    pub fn find_file(&self, path: &str) -> Option<&'static [u8]>;
    pub fn list_files(&self) -> impl Iterator<Item = &str>;
}
```

## 3. Data Model Changes
- No persistent data model changes.
- In-memory representation of initramfs structure (likely just the raw buffer and offset pointers).

## 4. Behavioral Decisions
- **Read-Only**: The initramfs is strictly read-only.
- **Memory Residency**: The initramfs remains in memory at its boot location. No copying to a new buffer unless necessary.
- **Fault Tolerance**: If no initramfs is found or if parsing fails, the kernel should log a warning but continue booting (it can still try to mount VirtIO-Blk).

## 5. Open Questions
1. **Should we copy the DTB?** QEMU might overwrite the DTB location later if we are not careful (though unlikely if it's outside our heap).
2. **Dynamic Mapping?** Should we use the MMU to create a specific mapping for the initrd, or rely on the initial 1GB block mapping?
   - *Recommendation*: Rely on the 1GB block for simplicity now, but add a explicit mapping in `mmu::init` if it's found outside that range.
3. **Crate for CPIO?** Should we use `cpio-reader` or write our own? New ASCII format is very simple to parse.
   - *Recommendation*: Write a minimal custom parser to keep dependencies low and tailored for our needs.

## 6. Design Alternatives
- **Built-in initrd**: Bundle the initrd into the kernel binary using `include_bytes!`.
  - **Pros**: Even simpler, no DTB required.
  - **Cons**: Less flexible (requires recompiling kernel to change initrd), larger kernel binary.
  - **Decision**: Support QEMU/Bootloader passed initrd as it's the standard way for most OSes.
