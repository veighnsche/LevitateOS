# TEAM_470: Feature - PT_INTERP Dynamic Linking Support

## Objective
Enable LevitateOS to run dynamically-linked ELF binaries by implementing PT_INTERP program header support. When an ELF has a PT_INTERP segment, the kernel loads the specified interpreter (e.g., `/lib/ld-musl-x86_64.so.1`) and jumps to its entry point.

## Status: COMPLETE (Kernel Implementation)

The PT_INTERP kernel-side implementation is complete and verified working. The dynamic linker starts executing but needs file-backed mmap support (future work).

## Progress Log

### Session 1 (2026-01-13)
- Created implementation plan
- Implemented PT_INTERP constant in elf.rs
- Added find_interp() method to parse PT_INTERP segment
- Added load_at() method to load ELF at specified base address
- Added resolve_interpreter() helper in process.rs
- Modified prepare_exec_image() for interpreter support
- Modified spawn_from_elf() for interpreter support
- Verified static binaries still work
- Modified xtask/src/build/initramfs.rs to include:
  - /lib directory for dynamic linker
  - musl dynamic linker (/lib/ld-musl-x86_64.so.1)
  - Extra test binaries from xtask/initrd_resources/bin/
- Created test dynamic binary (hello_dynamic) with musl-gcc
- **Final test result**: PT_INTERP works correctly!

### Test Output
```
[EXEC] Dynamic binary, interpreter: /lib/ld-musl-x86_64.so.1
[EXEC] Dynamic: interp_entry=0x7f000006cbfa main_entry=0x400340 AT_BASE=0x7f0000000000
[MMAP] Only MAP_ANONYMOUS supported, got flags=0x2
Error loading shared library ld-musl-x86_64.so.1: Invalid argument (needed by /bin/hello_dynamic)
```

The kernel correctly:
1. Detected PT_INTERP and found interpreter path
2. Loaded interpreter at INTERP_BASE (0x7f0000000000)
3. Set up auxv with AT_BASE=interpreter, AT_ENTRY=main program
4. Jumped to interpreter's entry point

The dynamic linker STARTED EXECUTING but failed because mmap doesn't support file-backed mappings yet.

## Key Design Decisions

1. **Interpreter base address**: 0x7f0000000000 (high address to avoid conflicts with main program)
2. **Both binaries loaded**: Kernel loads both main program AND interpreter into address space
3. **Auxv changes for dynamic linking**:
   - AT_BASE = interpreter's load base (not main program's)
   - AT_ENTRY = main program's entry point (interpreter needs to know where to jump)
   - AT_PHDR/AT_PHNUM = main program's headers (interpreter needs to read them)
4. **Entry point**: Jump to interpreter's entry, not main program's

## Files Modified

### Kernel Changes
- `crates/kernel/levitate/src/loader/elf.rs`:
  - Added `PT_INTERP` constant (value 3)
  - Added `find_interp()` method - parses PT_INTERP segment, returns interpreter path
  - Added `load_at()` method - loads ELF at specified base address
- `crates/kernel/levitate/src/process.rs`:
  - Added `resolve_interpreter()` helper - loads interpreter from initramfs
  - Modified `prepare_exec_image()` - handles interpreter loading and auxv setup
  - Modified `spawn_from_elf()` - similar changes for that code path

### Build System Changes
- `xtask/src/build/initramfs.rs`:
  - Added /lib directory creation
  - Added musl dynamic linker copy from /lib/ld-musl-x86_64.so.1
  - Added extra test binary copy mechanism from xtask/initrd_resources/bin/

### Test Files
- `xtask/initrd_resources/bin/hello_dynamic` - Test dynamically-linked binary

## Verification

- [x] Static binaries still work unchanged
- [x] PT_INTERP detection works correctly
- [x] Interpreter loads at correct address (0x7f0000000000)
- [x] Auxv set up correctly (AT_BASE, AT_ENTRY)
- [x] Entry point jumps to interpreter
- [ ] Dynamic binary fully executes (blocked on file-backed mmap)

## Related Teams
- TEAM_354 - PIE binary support
- TEAM_444 - musl libc migration
- TEAM_469 - procfs/sysfs (for /proc/self/maps debugging)

## Blockers for Full Dynamic Linking

The musl dynamic linker needs `mmap` with file-backed mappings (MAP_PRIVATE without MAP_ANONYMOUS, flags=0x2). Current kernel only supports MAP_ANONYMOUS.

### Required for full dynamic linking:
1. File-backed mmap support (MAP_PRIVATE with fd)
2. This allows the dynamic linker to map shared libraries into memory

## Handoff Notes

**PT_INTERP implementation is complete and working.** The kernel correctly:
- Detects PT_INTERP in ELF binaries
- Loads the interpreter at a high fixed base address
- Sets up auxv correctly for the dynamic linker
- Jumps to the interpreter's entry point

The dynamic linker starts executing but immediately hits the mmap limitation. To enable full dynamic binary support, a future team needs to implement file-backed mmap support.

### Quick verification command:
```bash
cargo xtask vm exec "/bin/hello_dynamic"
```

### Expected current output (PT_INTERP working):
```
[EXEC] Dynamic binary, interpreter: /lib/ld-musl-x86_64.so.1
[EXEC] Dynamic: interp_entry=0x7f000006cbfa main_entry=0x400340 AT_BASE=0x7f0000000000
```
