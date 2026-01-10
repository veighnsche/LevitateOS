/* TEAM_415: x86_64 Early Page Tables
 *
 * This file implements:
 * - UoW 2.5: Early identity page tables
 * - Page table initialization (32-bit and 64-bit versions)
 */

.intel_syntax noprefix

/* ============================================================================
 * Early Page Tables (UoW 2.5)
 * Placed in .data.boot so they are identity mapped (low physical addresses)
 * ============================================================================ */
.section .data.boot
.align 4096

/* PML4 - Page Map Level 4 */
.global early_pml4
early_pml4:
    .skip 4096

/* PDPT - Page Directory Pointer Table (for identity map) */
.global early_pdpt
early_pdpt:
    .skip 4096

/* PDPT for higher-half mapping */
.global early_pdpt_high
early_pdpt_high:
    .skip 4096

/* PMO Mapping Page Tables */
.global early_pmo_pdpt
early_pmo_pdpt:
    .skip 4096
.global early_pmo_pd
early_pmo_pd:
    .skip 4096

/* PD - Page Directory (2MB pages) */
.global early_pd
early_pd:
    .skip 4096

/* PD for kernel higher-half (2MB pages) */
.global early_pd_kernel
early_pd_kernel:
    .skip 4096

/* TEAM_278: PD for APIC identity mapping (4th GB) */
.global early_pd_apic
early_pd_apic:
    .skip 4096

/* Stack (16KB) - placed in .data.boot to be accessible during boot */
.align 16
boot_stack_bottom:
    .skip 16384
.global boot_stack_top
boot_stack_top:

/* ============================================================================
 * 32-bit Page table initialization
 * ============================================================================ */
.section .text.boot, "ax"
.code32

.global setup_early_page_tables
setup_early_page_tables:
    /* Clear page table memory (8 pages: early_pml4, early_pdpt, early_pdpt_high, early_pmo_pdpt, early_pmo_pd, early_pd, early_pd_kernel, early_pd_apic) */
    mov edi, offset early_pml4_phys
    xor eax, eax
    mov ecx, 4096 * 8 / 4
    rep stosd

    /* Restore EDI to early_pml4 base */
    mov edi, offset early_pml4_phys

    /* PML4[0] -> PDPT (identity map for low memory) */
    mov eax, offset early_pdpt_phys
    or eax, 0x03                        /* Present + Writable */
    mov dword ptr [edi], eax

    /* PML4[511] -> PDPT_high (higher-half mapping) */
    mov eax, offset early_pdpt_high_phys
    or eax, 0x03
    mov dword ptr [edi + 511 * 8], eax

    /* PML4[256] -> early_pmo_pdpt (PMO Mapping for PHYS_OFFSET) */
    mov eax, offset early_pmo_pdpt_phys
    or eax, 0x03
    mov dword ptr [edi + 256 * 8], eax

    /* PDPT[0] -> PD */
    mov edi, offset early_pdpt_phys
    mov eax, offset early_pd_phys
    or eax, 0x03                        /* Present + Writable */
    mov dword ptr [edi], eax

    /* PDPT_high[510] -> PD_kernel (for -2GB = 0xFFFFFFFF80000000) */
    mov edi, offset early_pdpt_high_phys
    mov eax, offset early_pd_kernel_phys
    or eax, 0x03
    mov dword ptr [edi + 510 * 8], eax

    /* early_pmo_pdpt[0] -> early_pmo_pd */
    mov edi, offset early_pmo_pdpt_phys
    mov eax, offset early_pmo_pd_phys
    or eax, 0x03
    mov dword ptr [edi], eax

    /* TEAM_278: PDPT[3] -> early_pd_apic (for APIC at 0xFEE00000 in 4th GB) */
    mov edi, offset early_pdpt_phys
    mov eax, offset early_pd_apic_phys
    or eax, 0x03
    mov dword ptr [edi + 3 * 8], eax

    /* TEAM_278: Map APIC region (2MB huge page at 0xFEC00000 and 0xFEE00000) */
    /* PD index for 0xFEC00000: (0xFEC00000 >> 21) & 0x1FF = 502 */
    /* PD index for 0xFEE00000: (0xFEE00000 >> 21) & 0x1FF = 503 */
    mov edi, offset early_pd_apic_phys
    mov eax, 0xFEC00083                 /* 0xFEC00000 | Present | Writable | Huge */
    mov dword ptr [edi + 502 * 8], eax
    mov eax, 0xFEE00083                 /* 0xFEE00000 | Present | Writable | Huge */
    mov dword ptr [edi + 503 * 8], eax

    /* PD[0-63] -> 2MB huge pages at physical 0x0..0x7FE00000 (128MB total) */
    /* This covers BIOS, kernel, initramfs, and EARLY_ALLOCATOR */
    mov edi, offset early_pd_phys
    mov eax, 0x00000083                 /* Present + Writable + Huge (2MB) */
    mov ecx, 64                         /* Map 64 huge pages */
.map_loop:
    mov dword ptr [edi], eax
    add eax, 0x200000                   /* Next 2MB */
    add edi, 8                          /* Next entry */
    loop .map_loop

    /* PD_kernel[0-511] -> 2MB huge pages at physical 2MB..1026MB */
    /* This maps 0xFFFFFFFF80000000 to physical 2MB+ (maps entire first 1GB) */
    mov edi, offset early_pd_kernel_phys
    mov eax, 0x00200083                 /* 2MB | Present | Writable | Huge */
    mov ecx, 512
.map_kernel_loop:
    mov dword ptr [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .map_kernel_loop

    /* early_pmo_pd[0-511] -> 2MB huge pages at physical 0x0..0x3FE00000 (1GB total) */
    mov edi, offset early_pmo_pd_phys
    mov eax, 0x00000083                 /* Present + Writable + Huge (2MB) */
    mov ecx, 512                        /* Map 1GB total in 2MB chunks */
.map_pmo_loop:
    mov dword ptr [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .map_pmo_loop

    ret

/* ============================================================================
 * 64-bit Page table initialization (for Limine path)
 * ============================================================================ */
.section .text.boot, "ax"
.code64

.global setup_early_page_tables_64
setup_early_page_tables_64:
    push rbp
    mov rbp, rsp
    
    /* '1' - Before writable check */
    mov dx, 0x3f8
    mov al, '1'
    out dx, al

    /* Check if early_pml4 is writable */
    movabs rdi, offset early_pml4
    mov byte ptr [rdi], 0

    /* '2' - Before clear */
    mov al, '2'
    out dx, al

    /* Rewrite the clear loop for 64-bit using virtual addresses */
    movabs rdi, offset early_pml4
    xor rax, rax
    mov rcx, 4096 * 8 / 8
    rep stosq

    /* '3' - After clear */
    mov al, '3'
    out dx, al

    /* PML4 setup */
    movabs rdi, offset early_pml4
    
    movabs rax, offset early_pdpt_phys
    or rax, 0x03
    mov [rdi], rax

    movabs rax, offset early_pdpt_high_phys
    or rax, 0x03
    mov [rdi + 511 * 8], rax

    movabs rax, offset early_pmo_pdpt_phys
    or rax, 0x03
    mov [rdi + 256 * 8], rax

    /* '4' - After PML4 */
    mov al, '4'
    out dx, al

    /* PDPT setup */
    movabs rdi, offset early_pdpt
    movabs rax, offset early_pd_phys
    or rax, 0x03
    mov [rdi], rax

    movabs rdi, offset early_pdpt_high
    movabs rax, offset early_pd_kernel_phys
    or rax, 0x03
    mov [rdi + 510 * 8], rax

    /* '5' - After PDPT */
    mov al, '5'
    out dx, al

    /* PMO PDPT setup */
    movabs rdi, offset early_pmo_pdpt
    movabs rax, offset early_pmo_pd_phys
    or rax, 0x03
    mov [rdi], rax

    /* APIC setup */
    movabs rdi, offset early_pdpt
    movabs rax, offset early_pd_apic_phys
    or rax, 0x03
    mov [rdi + 3 * 8], rax

    movabs rdi, offset early_pd_apic
    movabs rax, 0xFEC00083
    mov [rdi + 502 * 8], rax
    movabs rax, 0xFEE00083
    mov [rdi + 503 * 8], rax

    /* '6' - After APIC */
    mov al, '6'
    out dx, al

    /* PD setup (BIOS/Low memory) - Map 128MB */
    movabs rdi, offset early_pd
    mov rax, 0x00000083
    mov rcx, 64
.map_loop64:
    mov [rdi], rax
    add rax, 0x200000
    add rdi, 8
    dec rcx
    jnz .map_loop64

    /* '7' - After PD identity */
    mov al, '7'
    out dx, al

    /* PD_kernel setup - map 1GB starting from physical 2MB */
    movabs rdi, offset early_pd_kernel
    movabs rax, 0x00200083
    mov rcx, 512
.map_kernel_loop64:
    mov [rdi], rax
    add rax, 0x200000
    add rdi, 8
    dec rcx
    jnz .map_kernel_loop64

    /* '8' - After PD kernel */
    mov al, '8'
    out dx, al

    /* PMO PD setup */
    movabs rdi, offset early_pmo_pd
    movabs rax, 0x00000083
    mov rcx, 512
.map_pmo_loop64:
    mov [rdi], rax
    add rax, 0x200000
    add rdi, 8
    dec rcx
    jnz .map_pmo_loop64

    /* '9' - Done */
    mov al, '9'
    out dx, al

    leave
    ret
