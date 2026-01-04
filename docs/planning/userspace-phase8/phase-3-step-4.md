# Phase 3 / Step 4: ELF Loader

## Goal
Implement ELF64 binary parsing and loading for AArch64.

## Parent Context
- [Phase 3](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md)
- [Phase 2 Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)

## Design Reference
From Phase 2:
- Static linking only (no dynamic loader)
- Parse ELF64 header and program headers
- Map PT_LOAD segments into user address space
- Entry point from ELF header

## Units of Work

### UoW 1: ELF Header Parsing
Parse ELF64 header and validate.

**Tasks:**
1. Create `kernel/src/loader/mod.rs` and `kernel/src/loader/elf.rs`.
2. Define `Elf64Header` struct matching ELF64 format.
3. Implement `Elf64Header::parse(data: &[u8]) -> Result<Self, ElfError>`.
4. Validate:
   - Magic bytes `0x7f ELF`
   - Class = 64-bit (2)
   - Machine = AArch64 (183)
   - Type = Executable (2)

**Exit Criteria:**
- Can parse valid ELF headers.
- Returns error for invalid/unsupported binaries.

### UoW 2: Program Header Parsing
Parse program headers to find loadable segments.

**Tasks:**
1. Define `ProgramHeader` struct.
2. Implement `parse_program_headers(elf_data, elf_header) -> Vec<ProgramHeader>`.
3. Filter for `PT_LOAD` type (1).
4. Extract: `p_vaddr`, `p_offset`, `p_filesz`, `p_memsz`, `p_flags`.

**Exit Criteria:**
- Can identify loadable segments.
- Correct virtual addresses and sizes extracted.

### UoW 3: Segment Loading
Load ELF segments into user address space.

**Tasks:**
1. Implement `load_elf(data: &[u8], user_pgtable: &mut PageTable) -> Result<usize, ElfError>`.
2. For each PT_LOAD segment:
   - Allocate physical pages
   - Copy segment data from ELF
   - Zero-fill `.bss` (p_memsz - p_filesz)
   - Map pages into user address space with correct flags
3. Return entry point address.

**Exit Criteria:**
- ELF segments loaded into memory.
- Entry point returned correctly.
- `.bss` is zeroed.

### UoW 4: Stack Setup
Set up initial user stack.

**Tasks:**
1. Allocate stack pages (e.g., 16 pages = 64KB).
2. Map stack at top of user address space (e.g., `0x7FFF_F000_0000`).
3. Initialize stack with:
   - argc, argv, envp (can be empty initially)
4. Return initial stack pointer.

**Exit Criteria:**
- User stack is allocated and mapped.
- Stack pointer points to valid memory.

## Expected Outputs
- `kernel/src/loader/elf.rs` with ELF parsing.
- ELF loading into user address space.
- Entry point and stack pointer ready for `enter_user_mode`.
