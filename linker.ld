ENTRY(_head)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 127M
}

SECTIONS
{
    . = 0x40080000;

    .text.head : {
        KEEP(*(.text.head))
    } > RAM

    .text : {
        KEEP(*(.text.boot))
        . = ALIGN(0x800);
        *(.text.vectors)
        *(.text*)
    } > RAM

    .rodata : {
        *(.rodata*)
    } > RAM

    .data : {
        *(.data*)
    } > RAM

    .bss : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > RAM

    . = ALIGN(16);
    . = . + 0x10000;
    stack_top = .;

    . = ALIGN(16);
    __heap_start = .;
    __heap_end = 0x41F00000;

    _kernel_size = . - 0x40080000;
}
