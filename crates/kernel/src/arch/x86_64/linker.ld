/* TEAM_258: x86_64 Linker Script for Multiboot2 Boot
 *
 * Memory Layout:
 * - Physical: 0x100000 (1MB) - standard x86 kernel load address
 * - Virtual:  0xFFFFFFFF80000000 (higher-half, -2GB)
 *
 * The kernel is loaded at 1MB by the bootloader (GRUB/Multiboot2).
 * Early boot code runs with identity mapping, then transitions to higher-half.
 */

ENTRY(_start64)

PHDRS
{
    boot_text PT_LOAD FLAGS(5); /* R-X */
    boot_data PT_LOAD FLAGS(6); /* RW- */
    text      PT_LOAD FLAGS(5); /* R-X */
    rodata    PT_LOAD FLAGS(4); /* R-- */
    data      PT_LOAD FLAGS(6); /* RW- */
}

SECTIONS
{
    /* Higher-Half Virtual Address Base for x86_64 */
    _kernel_virt_base = 0xFFFFFFFF80000000;
    . = _kernel_virt_base;

    _kernel_virt_start = .;

    /* Physical Load Offset (2MB) to avoid overwriting BIOS/low memory */
    _phys_offset = 0x200000;

    /* Boot text section - must be early in the file for Multiboot */
    .boot_text ALIGN(4K) : AT(ADDR(.boot_text) - _kernel_virt_base + _phys_offset) {
        __boot_text_start = .;
        KEEP(*(.multiboot1))
        KEEP(*(.multiboot2))
        KEEP(*(.text.boot))
        __boot_text_end = .;
    } :boot_text

    /* Boot data section - must be writable */
    .boot_data ALIGN(4K) : AT(ADDR(.boot_data) - _kernel_virt_base + _phys_offset) {
        __boot_data_start = .;
        KEEP(*(.requests)) /* Limine requests - MUST be writable for responses */
        KEEP(*(.data.boot))
        __boot_data_end = .;
    } :boot_data

    /* Physical address aliases for 32-bit boot code */
    multiboot1_header_start_phys = ABSOLUTE(multiboot1_header_start - _kernel_virt_base + _phys_offset);
    multiboot2_header_start_phys = ABSOLUTE(multiboot2_header_start - _kernel_virt_base + _phys_offset);
    _start_phys = ABSOLUTE(_start - _kernel_virt_base + _phys_offset);
    early_pml4_phys = ABSOLUTE(early_pml4 - _kernel_virt_base + _phys_offset);
    early_pdpt_phys = ABSOLUTE(early_pdpt - _kernel_virt_base + _phys_offset);
    early_pdpt_high_phys = ABSOLUTE(early_pdpt_high - _kernel_virt_base + _phys_offset);
    early_pmo_pdpt_phys = ABSOLUTE(early_pmo_pdpt - _kernel_virt_base + _phys_offset);
    early_pmo_pd_phys = ABSOLUTE(early_pmo_pd - _kernel_virt_base + _phys_offset);
    early_pd_phys = ABSOLUTE(early_pd - _kernel_virt_base + _phys_offset);
    early_pd_kernel_phys = ABSOLUTE(early_pd_kernel - _kernel_virt_base + _phys_offset);
    early_pd_apic_phys = ABSOLUTE(early_pd_apic - _kernel_virt_base + _phys_offset);
    GDT64_phys = ABSOLUTE(GDT64 - _kernel_virt_base + _phys_offset);
    GDT64_Pointer_phys = ABSOLUTE(GDT64_Pointer - _kernel_virt_base + _phys_offset);
    GDT64_Pointer_32_phys = ABSOLUTE(GDT64_Pointer_32 - _kernel_virt_base + _phys_offset);
    boot_stack_top_phys = ABSOLUTE(boot_stack_top - _kernel_virt_base + _phys_offset);
    long_mode_start_phys = ABSOLUTE(long_mode_start - _kernel_virt_base + _phys_offset);
    setup_early_page_tables_phys = ABSOLUTE(setup_early_page_tables - _kernel_virt_base + _phys_offset);
    setup_early_page_tables_64_phys = ABSOLUTE(setup_early_page_tables_64 - _kernel_virt_base + _phys_offset);

    .text ALIGN(4K) : AT(ADDR(.text) - _kernel_virt_base + _phys_offset) {
        __text_start = .;
        *(.text*)
        __text_end = .;
    } :text

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - _kernel_virt_base + _phys_offset) {
        __rodata_start = .;
        *(.rodata*)
        __rodata_end = .;
    } :rodata

    .data ALIGN(4K) : AT(ADDR(.data) - _kernel_virt_base + _phys_offset) {
        __data_start = .;
        *(.data*)
        __data_end = .;
    } :data

    .bss ALIGN(4K) : AT(ADDR(.bss) - _kernel_virt_base + _phys_offset) {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(16);
        stack_bottom = .;
        . = . + 0x4000;  /* 16KB stack */
        stack_top = .;
        __bss_end = .;
    } :data

    /* Kernel heap - must be part of a segment to be mapped by Limine */
    .heap ALIGN(4K) : AT(ADDR(.heap) - _kernel_virt_base + _phys_offset) {
        __heap_start = .;
        . = . + 0x1000000; /* 16MB */
        __heap_end = .;
        BYTE(0) /* Force section to have size and be included in segment MemSiz */
    } :data

    _kernel_end = .;

    /* Symbols for boot code compatibility */
    __kernel_start = _kernel_virt_start;
    __kernel_phys_start = _phys_offset;
    _kernel_size = _kernel_end - _kernel_virt_start;

    /* Discard unwanted sections */
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note*)
    }
}
