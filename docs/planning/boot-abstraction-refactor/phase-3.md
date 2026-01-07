# Phase 3: Migration — Switch to New Boot Abstraction

## Purpose
Migrate all kernel initialization code to consume `BootInfo` instead of raw bootloader data, and make Limine the primary boot path for x86_64.

---

## Migration Strategy

### UNIX Philosophy: Clean Breaks (Rule 5)

> "Favor clean breaks over compatibility hacks."

We will:
1. **Move callers to new API** - Not add adapters
2. **Let compiler find all sites** - Change type signatures
3. **Fix each call site directly** - No shims

### Migration Order (Dependency-Based)

```
1. Memory subsystem (needs memory_map)
      ↓
2. HAL init (needs framebuffer, ACPI/DTB)
      ↓
3. Scheduler (needs memory for stacks)
      ↓
4. Init process (needs initramfs)
```

---

## Call Site Inventory

### x86_64 Consumers of Boot Info

| Location | Currently Uses | Needs From BootInfo |
|----------|---------------|---------------------|
| `arch/x86_64/mod.rs:kernel_main` | `multiboot_magic`, `multiboot_info` | `boot_info.protocol`, `boot_info.firmware` |
| `memory/mod.rs:init_x86_64` | Multiboot2 memory map | `boot_info.memory_map` |
| `los_hal::x86_64::multiboot2` | Raw multiboot2 tags | Remove - use BootInfo |
| `los_hal::x86_64::init` | Implicit ACPI location | `boot_info.firmware.rsdp` |

### AArch64 Consumers of Boot Info

| Location | Currently Uses | Needs From BootInfo |
|----------|---------------|---------------------|
| `main.rs:rust_main` | `dtb_ptr: usize` | `boot_info.firmware.dtb` |
| `init.rs` | DTB for memory | `boot_info.memory_map` |
| `los_hal::aarch64::dtb` | Raw DTB pointer | `boot_info.firmware.dtb` |

### Shared Consumers

| Location | Currently Uses | Needs From BootInfo |
|----------|---------------|---------------------|
| `init.rs:mount_initramfs` | Hardcoded location | `boot_info.initramfs` |
| `terminal.rs` | Framebuffer from HAL | `boot_info.framebuffer` |

---

## Steps

### Step 1: Migrate Memory Initialization
**Priority**: HIGH (everything depends on memory)

Tasks:
1. Change `memory::init()` to accept `&BootInfo`
2. Replace multiboot2 memory map parsing with `boot_info.memory_map`
3. Replace DTB memory parsing with `boot_info.memory_map`
4. Remove duplicated memory detection logic

**Exit Criteria**:
- `memory::init(&boot_info)` works for both architectures
- Buddy allocator initializes from unified memory map

### Step 2: Migrate HAL Initialization
**Priority**: HIGH (console, interrupts)

Tasks:
1. Pass RSDP to APIC init via `boot_info.firmware`
2. Pass DTB to AArch64 HAL via `boot_info.firmware`
3. Use `boot_info.framebuffer` for early console if available

**Exit Criteria**:
- HAL init uses BootInfo, not raw pointers
- Serial/VGA console works on both architectures

### Step 3: Migrate Initramfs Handling
**Priority**: MEDIUM

Tasks:
1. Use `boot_info.initramfs` instead of hardcoded address
2. Limine provides initramfs location via module request

**Exit Criteria**:
- Initramfs loads from BootInfo on all boot paths

### Step 4: Make Limine Primary for x86_64
**Priority**: HIGH for real hardware

Tasks:
1. Update build system to produce Limine-compatible ELF
2. Create `limine.cfg` for QEMU testing
3. Update `run-term.sh` to use Limine ISO
4. Verify NUC can boot Limine USB

**Exit Criteria**:
- `cargo xtask run` uses Limine by default for x86_64
- Multiboot path still works with `--legacy` flag

### Step 5: Remove Old Entry Points
**Priority**: After Limine is stable

Tasks:
1. Remove `kernel_main(magic, info)` x86_64 signature
2. Remove `rust_main(dtb)` AArch64 signature  
3. Single `kernel_main(&BootInfo)` for all

**Exit Criteria**:
- One entry point signature
- All tests pass

---

## Rollback Plan

If migration causes issues:

1. **Limine breaks**: Fall back to multiboot path (keep both during migration)
2. **BootInfo wrong**: Keep raw pointer access as fallback
3. **AArch64 regresses**: DTB path unchanged until x86_64 proven

Key principle: **Never remove old path until new path is verified**

---

## Test Checkpoints

After each step:
```bash
# x86_64 boot
timeout 5 qemu-system-x86_64 -M q35 -m 1G \
  -kernel target/x86_64-unknown-none/release/levitate-kernel \
  -nographic -serial mon:stdio -no-reboot

# AArch64 boot  
timeout 5 qemu-system-aarch64 -M virt -m 1G \
  -kernel kernel64_rust.bin \
  -nographic -serial mon:stdio -no-reboot

# Behavior tests
cargo xtask test behavior
```
