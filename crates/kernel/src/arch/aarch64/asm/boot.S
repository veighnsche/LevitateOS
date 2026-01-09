.section ".text.head", "ax"
.global _head
.global _start

_head:
    b       _start
    .long   0
    .quad   0x80000          /* text_offset: kernel expects RAM_BASE + 0x80000 */
    .quad   _kernel_size     /* image_size: calculated by linker script */
    .quad   0x0A             /* flags: LE, 4K pages */
    .quad   0
    .quad   0
    .quad   0
    .ascii  "ARM\x64"
    .long   0

_start:
    msr     daifset, #0xf
    mrs     x1, mpidr_el1
    and     x1, x1, #0xFF
    cbz     x1, primary_cpu

secondary_halt:
    wfe
    b       secondary_halt

primary_cpu:
    /* Save x0-x3 to callee-saved registers x19-x22 */
    mov     x19, x0
    mov     x20, x1
    mov     x21, x2
    mov     x22, x3

    /* Enable FP/SIMD */
    mov     x0, #0x300000
    msr     cpacr_el1, x0
    isb

    /* Zero BSS (using physical addresses during boot) */
    /* Note: In higher half, symbols like __bss_start are high VAs. */
    /* We need to convert them to physical for early boot. */
    ldr     x0, =__bss_start
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1          /* x0 = __bss_start_phys */
    ldr     x2, =__bss_end
    sub     x2, x2, x1          /* x2 = __bss_end_phys */
    mov     x3, #0
bss_loop:
    cmp     x0, x2
    b.ge    bss_done
    str     x3, [x0], #8
    b       bss_loop
bss_done:

    /* Save preserved registers to global variable BOOT_REGS */
    ldr     x0, =BOOT_REGS
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1          /* x0 = physical address of BOOT_REGS */
    str     x19, [x0]           /* x0 */
    str     x20, [x0, #8]       /* x1 */
    str     x21, [x0, #16]      /* x2 */
    str     x22, [x0, #24]      /* x3 */

    /* Save DTB to BOOT_DTB_ADDR for compatibility */
    ldr     x0, =BOOT_DTB_ADDR
    ldr     x1, =_kernel_virt_base
    sub     x0, x0, x1
    str     x19, [x0]

    /* Setup Early Page Tables */
    /* L0_low[0] -> L1_low (ID map for first 1GB) */
    ldr     x4, =_kernel_virt_base
    
    ldr     x0, =boot_pt_l0_low
    sub     x0, x0, x4          /* x0 = boot_pt_l0_low_phys */
    ldr     x1, =boot_pt_l1_low
    sub     x1, x1, x4          /* x1 = boot_pt_l1_low_phys */
    orr     x1, x1, #0x3        /* Table + Valid */
    str     x1, [x0]

    /* L1_low[0] -> 0x00000000 (1GB Device Block) */
    ldr     x0, =boot_pt_l1_low
    sub     x0, x0, x4
    mov     x1, #0x00000000
    add     x1, x1, #0x405      /* Block + AF + Attr1 (Device) */
    str     x1, [x0]
    
    /* L1_low[1] -> 0x40000000 (1GB Normal Block for RAM) */
    mov     x1, #0x40000000
    add     x1, x1, #0x401      /* Block + AF + Attr0 (Normal) */
    str     x1, [x0, #8]        /* Index 1 = 1GB */

    /* L0_high[256] -> L1_high (Higher-half base 0xFFFF8000...) */
    ldr     x0, =boot_pt_l0_high
    sub     x0, x0, x4
    ldr     x1, =boot_pt_l1_high
    sub     x1, x1, x4
    orr     x1, x1, #0x3
    str     x1, [x0, #256*8]

    /* TEAM_078: L1_high[0] -> 0x00000000 (1GB Device Block for UART, GIC, VirtIO) */
    /* This enables device access via TTBR1 high VA before kmain() */
    ldr     x0, =boot_pt_l1_high
    sub     x0, x0, x4
    mov     x1, #0x00000000
    add     x1, x1, #0x405      /* Block + AF + Attr1 (Device) */
    str     x1, [x0]            /* Index 0 for 0x00000000 */

    /* L1_high[1] -> 0x40000000 (1GB Block matching 0x40000000 physical) */
    mov     x1, #0x40000000
    add     x1, x1, #0x401      /* Block + AF + Attr0 (Normal) */
    str     x1, [x0, #8]        /* Index 1 for 0x40000000 */

    /* Configure MMU Registers */
    /* MAIR_EL1: Attr0=0xFF (Normal), Attr1=0x04 (Device) */
    ldr     x0, =0x00000000000004FF
    msr     mair_el1, x0

    /* TCR_EL1: T0SZ=16, T1SZ=16, TG0=4K, TG1=4K, IPS=48bit, SH0/SH1=Inner, Cacheable */
    ldr     x0, =0x00000005b5103510
    msr     tcr_el1, x0
    isb

    /* Load TTBR0 and TTBR1 */
    ldr     x0, =boot_pt_l0_low
    sub     x0, x0, x4
    msr     ttbr0_el1, x0
    ldr     x0, =boot_pt_l0_high
    sub     x0, x0, x4
    msr     ttbr1_el1, x0
    isb

    /* Enable MMU */
    mrs     x0, sctlr_el1
    orr     x0, x0, #0x1        /* M (MMU) */
    orr     x0, x0, #0x4        /* C (D-Cache) */
    orr     x0, x0, #0x1000     /* I (I-Cache) */
    msr     sctlr_el1, x0
    isb

    /* Jump to High VA kmain */
    ldr     x0, =stack_top
    mov     sp, x0
    ldr     x0, =kmain
    br      x0

.section ".text", "ax"

.section ".data.boot_pt", "aw"
.align 12
boot_pt_l0_low:  .space 4096
boot_pt_l1_low:  .space 4096
boot_pt_l0_high: .space 4096
boot_pt_l1_high: .space 4096

.section ".data"
.global _end
_end:
