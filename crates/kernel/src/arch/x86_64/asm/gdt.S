/* TEAM_415: x86_64 GDT64 - Global Descriptor Table for 64-bit mode
 *
 * This file implements:
 * - UoW 2.3: GDT64 for long mode
 */

.intel_syntax noprefix

/* ============================================================================
 * GDT64 - Global Descriptor Table for 64-bit mode (UoW 2.3)
 * Placed in .data.boot so it's identity mapped
 * ============================================================================ */
.section .data.boot
.align 16

.global GDT64
GDT64:
    /* Null descriptor (entry 0) */
    .quad 0

    /* Code segment descriptor (entry 1, selector 0x08) */
    /* Base=0, Limit=0, Type=Code, L=1 (64-bit), P=1 */
    .quad 0x00AF9A000000FFFF

    /* Data segment descriptor (entry 2, selector 0x10) */
    /* Base=0, Limit=0, Type=Data, P=1 */
    .quad 0x00CF92000000FFFF

    /* TEAM_293: User segments for Ring 3 userspace execution */
    /* User Data segment (entry 3, selector 0x18) - DPL=3 */
    .quad 0x00CFF2000000FFFF

    /* User Code segment (entry 4, selector 0x20) - DPL=3, L=1 (64-bit) */
    .quad 0x00AFFA000000FFFF
gdt64_end:

.global GDT64_Pointer
GDT64_Pointer:
    .word gdt64_end - GDT64 - 1         /* Limit */
    .quad GDT64                         /* Base address (64-bit) */

.global GDT64_Pointer_32
GDT64_Pointer_32:
    .word gdt64_end - GDT64 - 1
    .long 0                             /* Placeholder for physical base */
