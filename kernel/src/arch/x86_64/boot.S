/* TEAM_258: x86_64 Boot Assembly
 *
 * This file implements:
 * - UoW 2.2: Multiboot2 header
 * - UoW 2.3: GDT64 for long mode
 * - UoW 2.4: Protected-to-long mode transition
 * - UoW 2.5: Early identity page tables
 * - UoW 2.6: 64-bit entry point
 */

.intel_syntax noprefix

/* ============================================================================
 * Multiboot2 Header (UoW 2.2)
 * Must be in first 32KB, 8-byte aligned
 * ============================================================================ */
.section .multiboot2, "a"
.align 8

multiboot2_header_start:
    .long 0xE85250D6                    /* Magic number */
    .long 0                             /* Architecture: 0 = i386 (protected mode) */
    .long multiboot2_header_end - multiboot2_header_start  /* Header length */
    .long -(0xE85250D6 + 0 + (multiboot2_header_end - multiboot2_header_start))  /* Checksum */

    /* Framebuffer tag (optional, request text mode) */
    .align 8
    .word 5                             /* Type: framebuffer */
    .word 0                             /* Flags */
    .long 20                            /* Size */
    .long 80                            /* Width (columns) */
    .long 25                            /* Height (rows) */
    .long 0                             /* Depth (0 = text mode) */

    /* End tag */
    .align 8
    .word 0                             /* Type: end */
    .word 0                             /* Flags */
    .long 8                             /* Size */
multiboot2_header_end:

/* ============================================================================
 * 32-bit Boot Code (UoW 2.4)
 * ============================================================================ */
.section .text.boot, "ax"
.code32
.global _start

_start:
    /* Disable interrupts */
    cli

    /* Save Multiboot2 magic and info pointer */
    mov edi, eax                        /* Magic in EDI */
    mov esi, ebx                        /* Multiboot2 info pointer in ESI */

    /* Check Multiboot2 magic */
    cmp eax, 0x36D76289
    jne .hang

    /* Set up stack (in low memory .data.boot) */
    mov esp, offset stack_top

    /* Load GDT64 */
    lgdt [GDT64.Pointer]

    /* Setup page tables */
    call setup_early_page_tables

    /* Enable PAE (Physical Address Extension) in CR4 */
    mov eax, cr4
    or eax, 0x20                        /* Bit 5: PAE */
    mov cr4, eax

    /* Set PML4 address in CR3 */
    mov eax, offset early_pml4
    mov cr3, eax

    /* Enable long mode in EFER MSR */
    mov ecx, 0xC0000080                 /* EFER MSR */
    rdmsr
    or eax, 0x100                       /* Bit 8: LME (Long Mode Enable) */
    wrmsr

    /* Enable paging and protected mode in CR0 */
    mov eax, cr0
    or eax, 0x80000001                  /* Bit 31: PG, Bit 0: PE */
    mov cr0, eax

    /* Far jump to 64-bit code segment */
    ljmp 0x08, offset long_mode_start

.hang:
    hlt
    jmp .hang

/* ============================================================================
 * 64-bit Entry Point (UoW 2.6)
 * ============================================================================ */
.section .text.boot, "ax"
.code64

long_mode_start:
    /* Reset segment registers to data segment */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Set up 64-bit stack */
    mov rsp, offset stack_top

    /* Zero out BSS */
    /* Use immediate load for absolute symbols */
    mov rdi, offset __bss_start
    mov rcx, offset __bss_end
    sub rcx, rdi
    shr rcx, 3                          /* Divide by 8 (qwords) */
    xor rax, rax
    rep stosq

    /* Prepare arguments for kernel_main */
    /* RDI = Multiboot2 magic (from EDI) */
    /* RSI = Multiboot2 info pointer (from ESI) */
    /* Zero extend to 64-bit */
    mov edi, edi
    mov esi, esi

    /* Call Rust kernel_main */
    call kernel_main

    /* If kernel_main returns, halt */
.halt:
    cli
    hlt
    jmp .halt

/* ============================================================================
 * GDT64 - Global Descriptor Table for 64-bit mode (UoW 2.3)
 * Placed in .data.boot so it's identity mapped
 * ============================================================================ */
.section .data.boot
.align 16

GDT64:
    /* Null descriptor (entry 0) */
    .quad 0

    /* Code segment descriptor (entry 1, selector 0x08) */
    /* Base=0, Limit=0, Type=Code, L=1 (64-bit), P=1 */
    .quad 0x00AF9A000000FFFF

    /* Data segment descriptor (entry 2, selector 0x10) */
    /* Base=0, Limit=0, Type=Data, P=1 */
    .quad 0x00CF92000000FFFF
gdt64_end:

GDT64.Pointer:
    .word gdt64_end - GDT64 - 1         /* Limit */
    .quad GDT64                         /* Base address */

/* ============================================================================
 * Early Page Tables (UoW 2.5)
 * Placed in .data.boot so they are identity mapped (low physical addresses)
 * ============================================================================ */
.align 4096

/* PML4 - Page Map Level 4 */
early_pml4:
    .skip 4096

/* PDPT - Page Directory Pointer Table (for identity map) */
early_pdpt:
    .skip 4096

/* PDPT for higher-half mapping */
early_pdpt_high:
    .skip 4096

/* PD - Page Directory (2MB pages) */
early_pd:
    .skip 4096

/* Stack (16KB) - placed in .data.boot to be accessible during boot */
.align 16
stack_bottom:
    .skip 16384
stack_top:

/* ============================================================================
 * Page table initialization
 * ============================================================================ */
.section .text.boot, "ax"
.code32

setup_early_page_tables:
    /* Clear page table memory */
    mov edi, offset early_pml4
    xor eax, eax
    mov ecx, 4096 * 4 / 4
    rep stosd

    /* Restore EDI to early_pml4 base */
    mov edi, offset early_pml4

    /* PML4[0] -> PDPT (identity map for low memory) */
    mov eax, offset early_pdpt
    or eax, 0x03                        /* Present + Writable */
    mov dword ptr [edi], eax

    /* PML4[511] -> PDPT_high (higher-half mapping) */
    mov eax, offset early_pdpt_high
    or eax, 0x03
    mov dword ptr [edi + 511 * 8], eax

    /* PDPT[0] -> PD */
    mov edi, offset early_pdpt
    mov eax, offset early_pd
    or eax, 0x03
    mov dword ptr [edi], eax

    /* PDPT_high[510] -> PD (for -2GB = 0xFFFFFFFF80000000) */
    mov edi, offset early_pdpt_high
    mov eax, offset early_pd
    or eax, 0x03
    mov dword ptr [edi + 510 * 8], eax

    /* PD[0] -> 2MB huge page at physical 0x0 */
    mov edi, offset early_pd
    mov eax, 0x00000083                 /* Present + Writable + Huge (2MB) */
    mov dword ptr [edi], eax

    /* PD[1] -> 2MB huge page at physical 0x200000 */
    mov eax, 0x00200083
    mov dword ptr [edi + 8], eax

    ret
