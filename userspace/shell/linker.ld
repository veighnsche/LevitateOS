ENTRY(_head)

SECTIONS
{
    /* Physical Load Address */
    . = 0x40080000;
    _kernel_phys_start = .;
    __executable_start = .;

    /* Boot code that runs before MMU or in identity map */
    .text.head : {
        KEEP(*(.text.head))
    }

    /* Higher-Half Virtual Address Base */
    _kernel_virt_base = 0xFFFF800000000000;
    . += _kernel_virt_base;
    _kernel_virt_start = .;

    .text : AT(ADDR(.text) - _kernel_virt_base) {
        KEEP(*(.text.boot))
        . = ALIGN(0x800);
        __vectors_start = .;
        *(.text.vectors)
        *(.text*)
    }

    .rodata : AT(ADDR(.rodata) - _kernel_virt_base) {
        *(.rodata*)
    }

    .data : AT(ADDR(.data) - _kernel_virt_base) {
        *(.data*)
    }

    .bss : AT(ADDR(.bss) - _kernel_virt_base) {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(16);
    . = . + 0x10000;
    stack_top = .;

    . = ALIGN(16);
    __heap_start = .;
    . = _kernel_virt_base + 0x41F00000;
    __heap_end = .;
    _kernel_end = .;

    _kernel_size = . - _kernel_virt_base - 0x40080000;
}
