/* TEAM_258: x86_64 Linker Script for Multiboot2 Boot
 *
 * Memory Layout:
 * - Physical: 0x100000 (1MB) - standard x86 kernel load address
 * - Virtual:  0xFFFFFFFF80000000 (higher-half, -2GB)
 *
 * The kernel is loaded at 1MB by the bootloader (GRUB/Multiboot2).
 * Early boot code runs with identity mapping, then transitions to higher-half.
 */

ENTRY(_start)

SECTIONS
{
    /* Multiboot2 header must be in first 32KB of the file */
    . = 0x100000;  /* Physical load address: 1MB */

    /* Multiboot2 header section - must come first */
    .multiboot2 ALIGN(8) : {
        KEEP(*(.multiboot2))
    }

    /* Boot code that runs before paging is set up (identity mapped) */
    .text.boot : {
        KEEP(*(.text.boot))
        KEEP(*(.text.head))
    }

    /* Boot data (page tables, etc.) - identity mapped */
    .data.boot : {
        KEEP(*(.data.boot))
    }

    /* Higher-Half Virtual Address Base for x86_64 */
    _kernel_virt_base = 0xFFFFFFFF80000000;

    /* Main kernel sections at higher-half addresses */
    . = . + _kernel_virt_base;
    _kernel_virt_start = .;

    /* TEAM_269: Added segment boundary symbols for init_kernel_mappings_refined() */
    .text ALIGN(4K) : AT(ADDR(.text) - _kernel_virt_base) {
        __text_start = .;
        *(.text*)
        __text_end = .;
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - _kernel_virt_base) {
        __rodata_start = .;
        *(.rodata*)
        __rodata_end = .;
    }

    .data ALIGN(4K) : AT(ADDR(.data) - _kernel_virt_base) {
        __data_start = .;
        *(.data*)
        __data_end = .;
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - _kernel_virt_base) {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    }

    /* Kernel stack (16KB) */
    . = ALIGN(16);
    stack_bottom = .;
    . = . + 0x4000;  /* 16KB stack */
    stack_top = .;

    /* Kernel heap */
    . = ALIGN(4K);
    __heap_start = .;
    . = _kernel_virt_base + 0x1000000;  /* 16MB higher-half region */
    __heap_end = .;

    _kernel_end = .;

    /* Symbols for boot code */
    __kernel_start = _kernel_virt_start;
    __kernel_phys_start = 0x100000;
    _kernel_size = _kernel_end - _kernel_virt_start;

    /* Discard unwanted sections */
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note*)
    }
}
