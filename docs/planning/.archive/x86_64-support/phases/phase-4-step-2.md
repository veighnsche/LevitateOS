# Phase 4 — Step 2: Initramfs and VFS Integration

## Parent
[Phase 4: Integration and Testing](phase-4.md)

## Goal
Ensure the initramfs can be loaded and parsed on x86_64, with proper VFS integration.

## Prerequisites
- Kernel boots on x86_64
- MMU and basic console output working

---

## UoW 2.1: Parse Multiboot2 Module Tags

**Goal**: Extract initramfs location from Multiboot2 info structure.

**File**: `kernel/src/arch/x86_64/multiboot2.rs` (new)

**Tasks**:
1. Create `multiboot2.rs`
2. Define Multiboot2 tag structures:
   - Basic tag header (type, size)
   - Module tag (type 3): mod_start, mod_end, cmdline
   - Memory map tag (type 6)
   - End tag (type 0)
3. Implement `parse_multiboot2(info_addr: usize)`:
   - Iterate through tags
   - Find module tag containing initramfs
   - Return (start_phys, end_phys) tuple
4. Handle case where no module is present

**Exit Criteria**:
- Can find initramfs physical address range
- Works with QEMU `-initrd` option

**Verification**:
- Print initramfs start/end addresses at boot

---

## UoW 2.2: Map Initramfs into Virtual Memory

**Goal**: Create virtual mapping for initramfs data.

**File**: `kernel/src/arch/x86_64/init.rs` (new or modify)

**Tasks**:
1. Get initramfs physical range from Multiboot2 tags
2. Calculate size and page count
3. Map pages into kernel virtual address space (read-only)
4. Return virtual address slice `&[u8]`
5. Ensure mapping persists (not temporary early boot mapping)

**Exit Criteria**:
- Initramfs data accessible via virtual pointer
- Can read first bytes (CPIO magic)

**Verification**:
- Print first 6 bytes: should be "070701" for newc format

---

## UoW 2.3: Verify CPIO Parser Compatibility

**Goal**: Confirm existing CPIO parser works on x86_64.

**File**: `kernel/src/fs/initramfs/cpio.rs` (verify)

**Tasks**:
1. Review CPIO parser for endianness assumptions
2. x86_64 is little-endian (same as AArch64) — should be fine
3. Check alignment assumptions (CPIO newc is ASCII-based)
4. Write a test that parses a known CPIO archive
5. Fix any issues found

**Exit Criteria**:
- CPIO parser extracts files correctly on x86_64
- File list matches what was packed

**Verification**:
- Parse test initramfs, list file names

---

## UoW 2.4: Wire Initramfs to VFS on x86_64

**Goal**: Mount initramfs as root filesystem on x86_64 boot.

**File**: `kernel/src/arch/x86_64/init.rs`

**Tasks**:
1. Add x86_64-specific initialization sequence
2. Call `initramfs::mount(&initramfs_data)` 
3. Verify VFS mounts at "/"
4. Ensure `init` binary can be opened from VFS

**Exit Criteria**:
- Root filesystem is mounted
- `/init` path resolves to init binary

**Verification**:
- Open `/init` via VFS, print file size

---

## Progress Tracking
- [ ] UoW 2.1: Multiboot2 Parsing
- [ ] UoW 2.2: Initramfs Mapping
- [ ] UoW 2.3: CPIO Verification
- [ ] UoW 2.4: VFS Integration

## Dependencies Graph
```
UoW 2.1 ──→ UoW 2.2 ──→ UoW 2.3 ──→ UoW 2.4
```
