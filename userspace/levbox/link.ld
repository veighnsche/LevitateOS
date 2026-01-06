/* TEAM_182: Linker script for coreutils binaries */
/* Places code at standard user address 0x10000 (64KB) */

ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    .text : {
        *(.text._start)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .init_array : {
        __init_array_start = .;
        *(.init_array .init_array.*)
        __init_array_end = .;
    }

    .fini_array : {
        __fini_array_start = .;
        *(.fini_array .fini_array.*)
        __fini_array_end = .;
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    _end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.note*)
        *(.comment)
    }
}
