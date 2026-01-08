# TEAM_297: Investigate x86_64 Invalid Opcode Crash

## Summary

**Status: ONGOING - Not Yet Resolved**

**Bug**: Kernel panic `EXCEPTION: INVALID OPCODE` at `RIP=0x100b9` immediately after shell prompt appears.

**Reproducer**: `timeout 25 cargo xtask run term --arch x86_64 --iso`

---

## Symptom Analysis

### What Works
- Kernel boots successfully via Limine
- Init process (PID 1) starts
- Shell (PID 2) spawns
- Shell prints banner: "LevitateOS Shell (lsh) v0.1"
- Shell prints "Type 'help' for commands."
- Shell prints prompt "# "

### What Fails
- **Immediately after prompt prints, crash occurs**
- Exception: `INVALID OPCODE` at `RIP=0x100b9`

### Crash Address Analysis

The crash address `0x100b9` is **inside a 6-byte instruction**:

```
100b6:  ff 15 fc 1e 00 00    call *0x1efc(%rip)   # calls GOT[0x11fb8]
        ^     ^
        |     0x100b9 = 0x1e byte (INVALID: PUSH DS in 64-bit mode!)
        0x100b6 = start of instruction
```

**Key Insight**: The byte at `0x100b9` is `0x1e`, which is `PUSH DS` - **invalid in 64-bit long mode**. This means the CPU is attempting to execute from the middle of an instruction, not from a valid instruction boundary.

---

## Hypotheses Tested

### ❌ Hypothesis 1: GOT/PLT Relocation Issue

**Theory**: The Global Offset Table (GOT) might not be properly populated, causing indirect calls to jump to garbage.

**Investigation**:
```bash
objdump -d ./userspace/target/x86_64-unknown-none/release/shell | grep -A5 "11fb8"
readelf -r ./userspace/target/x86_64-unknown-none/release/shell
```

**Findings**:
- GOT at `0x11fb8` contains `0x108b3` (correct function address for `Stdout::write_str`)
- Binary is `ET_EXEC` (statically linked), no dynamic relocations needed
- GOT is pre-populated at link time

**Outcome**: Ruled out. GOT is correctly populated.

---

### ❌ Hypothesis 2: SyscallFrame Layout Mismatch

**Theory**: The `SyscallFrame` struct might not match the assembly's push/pop order, causing register corruption.

**Investigation**:
- Traced every `push` in `syscall_entry` assembly
- Mapped to struct field offsets byte-by-byte
- Verified pop sequence restores in correct order

**Assembly Push Order** (first push = highest address):
```
sub rsp, 31*8    // Reserve 248 bytes for regs[31]
push 0           // pstate (offset 152)
push [user_rsp]  // sp (offset 144)
push rcx         // pc (offset 136)
push 0           // ttbr0 (offset 128)
push [user_rsp]  // rsp (offset 120)
push r15         // offset 112
... (all regs) ...
push rax         // offset 0
```

**Struct Layout**:
```rust
pub struct SyscallFrame {
    pub rax: u64,    // offset 0
    pub rdi: u64,    // offset 8
    ...
    pub r15: u64,    // offset 112
    pub rsp: u64,    // offset 120
    pub ttbr0: u64,  // offset 128
    pub pc: u64,     // offset 136
    pub sp: u64,     // offset 144
    pub pstate: u64, // offset 152
    pub regs: [u64; 31], // offset 160-408
}
```

**Outcome**: Ruled out. Layout matches exactly.

---

### ❌ Hypothesis 3: RCX (Return Address) Corrupted by syscall_dispatch

**Theory**: The Rust syscall handler might be modifying `frame.rcx`, corrupting the return address.

**Investigation**:
Added debug code to `syscall_handler`:
```rust
let pc_before = frame.rcx;
syscall_dispatch(frame);
if frame.rcx != pc_before {
    println!("[SYSCALL] WARNING: RCX changed!");
}
```

**Findings**:
- No warning message appeared in logs
- `syscall_dispatch` only modifies `frame.rax` (return value)

**Outcome**: Ruled out. RCX not corrupted by handler.

---

### ⚠️ Hypothesis 4: Early Calls Work, Later Calls Fail

**Theory**: Something changes between early function calls (which work) and later ones (which crash).

**Investigation**:
```
Early calls (work):       call *%r14     (register indirect)
Crash call:               call *0x1efc(%rip)  (memory indirect)
```

**Observation**: Both should work the same way - both read the same GOT entry.

**Suspicious**: The early calls load the GOT pointer into R14 once at `0x10018`, then reuse it. But at `0x10060`, R14 is overwritten for a different purpose. The crash call at `0x100b6` must read directly from memory.

**Outcome**: Partially relevant. Timing/state difference exists, but root cause unclear.

---

### 🔍 Hypothesis 5: sysretq Returns to Wrong Address

**Theory**: The `sysretq` instruction is returning to a corrupted RIP (off by 3 bytes).

**Evidence**:
- Expected return: `0x100bc` (instruction after the call)
- Actual RIP: `0x100b9` (3 bytes before)
- This is a **consistent -3 byte offset**

**Possible Causes**:
1. Stack corruption modifying return address
2. Hardware/CPU mode issue with sysretq
3. Page mapping issue causing wrong bytes to be fetched
4. Signal delivery corrupting frame

**Investigation Status**: This is the current leading hypothesis but not yet confirmed.

---

### 🔍 Hypothesis 6: Page Table Mapping Issue

**Theory**: The userspace page tables might be corrupted or incorrectly set, causing the CPU to fetch wrong bytes.

**ELF Segments**:
```
LOAD 0x10000 size 0x1a9f R-X (code)
LOAD 0x11aa0 size 0x500  R-- (rodata)
LOAD 0x11fa0 size 0x78   RW- (got)
LOAD 0x12018 size 0x100  RW- (data/bss)
```

**Page Overlaps**:
- Page `0x10000`: Only .text
- Page `0x11000`: .text tail + .rodata + .got (SHARED PAGE!)
- Page `0x12000`: .got tail + .data/.bss

**Concern**: Multiple segments share page `0x11000`. The ELF loader handles this by:
1. First segment allocates the page
2. Later segments check if already mapped, upgrade permissions if needed

**Outcome**: Need to verify page contents match binary.

---

## Debugging Tools Available

### GDB Support
```bash
cargo xtask run gdb --arch x86_64 --wait
# In another terminal:
gdb -ex "target remote :1234" target/x86_64-unknown-none/release/levitate-kernel
```

### Serial Logging
```bash
# Run with serial to file
timeout 25 qemu-system-x86_64 ... -serial file:/tmp/levitate_serial.log
```

### Disassembly
```bash
objdump -d ./userspace/target/x86_64-unknown-none/release/shell | less
readelf -l ./userspace/target/x86_64-unknown-none/release/shell
```

---

## Key Files

| File | Purpose |
|------|---------|
| `kernel/src/arch/x86_64/syscall.rs` | Syscall entry/exit assembly |
| `kernel/src/arch/x86_64/mod.rs` | SyscallFrame struct definition |
| `kernel/src/arch/x86_64/task.rs` | `enter_user_mode` via sysretq |
| `kernel/src/loader/elf.rs` | ELF loading and page mapping |
| `crates/hal/src/x86_64/mmu.rs` | Page flags definitions |
| `userspace/libsyscall/src/arch/x86_64.rs` | Userspace syscall stubs |

---

## Known Architectural Gaps

### Per-CPU State Management (Critical for SMP)

The current implementation uses **global static variables** for per-CPU state:
```rust
pub static mut CURRENT_KERNEL_STACK: usize = 0;
pub static mut USER_RSP_SCRATCH: usize = 0;
```

**Problem**: On multicore systems, this would cause race conditions.

**Solution**: Implement `swapgs` and `IA32_KERNEL_GS_BASE` MSR for thread-safe per-CPU data.

**Reference**: See `docs/x86_64_per_cpu_abstraction.md`

---

## Next Steps for Future Teams

1. **Use GDB** to set breakpoint at `sysretq` and inspect RCX value
   ```gdb
   break *0xffffffff80010100  # Near sysretq in syscall_entry
   ```

2. **Verify page contents** by dumping memory in GDB:
   ```gdb
   x/10i 0x100b0  # Should match objdump output
   ```

3. **Add more breadcrumbs** before/after sysretq:
   ```asm
   "mov [rip + {debug_rcx}], rcx",  // Save RCX before sysretq
   "sysretq",
   ```

4. **Check if crash happens on FIRST syscall return** or later

5. **Investigate signal handling** - signal delivery modifies frame PC

### 🔍 Hypothesis 7: ELF Loader Corruption (Strong)

**Theory**: The GOT entry for `write` at `0x11fb8` is being corrupted during loading.
**Evidence**:
- Indirect calls via register (`call *%r14`) work early in `_start`.
- Indirect calls via memory (`call *GOT`) fail later.
- Crash at `0x100b9` suggests execution jumped there. There is no valid jump to `0x100b9` in code.
- GOT at `0x11fb8` should contain `0x108b3`. If it contained `0x100b9`, it would explain the crash perfectly.

**Instrumentation Added**:
- Added log warning in `kernel/src/loader/elf.rs` whenever `0x11fb8` is written.
- Added log in `syscall_handler` to print `rcx` when `nr=1` (write).

---

## Breadcrumbs Added

```rust
// kernel/src/arch/x86_64/syscall.rs
// TEAM_297 BREADCRUMB: INVESTIGATING - Debug syscall entry/exit to trace RCX corruption
```

**Status**: Debug output was added but did NOT appear in logs. This suggests either:
- syscall_handler is not being reached, or
- los_hal::println! doesn't work during syscall context, or  
- The crash happens before/during the first write syscall

---

## Timeline

| Time | Action |
|------|--------|
| Session Start | Registered as TEAM_297, reviewed TEAM_296 findings |
| Early | Analyzed crash address, identified it's mid-instruction |
| Middle | Traced SyscallFrame layout, verified matches assembly |
| Middle | Added debug output to syscall_handler |
| Late | Discovered debug output NOT appearing in logs |
| Current | Documenting findings for future teams |

---

## TL;DR for Future Teams

1. **The crash is at 0x100b9**, which is byte 3 of a 6-byte CALL instruction
2. **RIP is off by 3 bytes** from expected return address
3. **GOT is correctly populated** - not a relocation issue
4. **SyscallFrame layout is correct** - not a struct mismatch
5. **Debug output doesn't appear** - something prevents println during syscall
6. **Use GDB** - that's your best debugging tool now
7. **Focus on sysretq** - the return path is likely the culprit
