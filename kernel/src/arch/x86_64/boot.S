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
 * Multiboot1 Header (for QEMU -kernel direct loading)
 * QEMU's -kernel option requires multiboot1, not multiboot2
 * Must be in first 8KB, 4-byte aligned
 * Using AOUT_KLUDGE to provide explicit load addresses for 64-bit ELF
 * ============================================================================ */
.section .multiboot1, "a"
.align 4

.global multiboot1_header_start
multiboot1_header_start:
MULTIBOOT_MAGIC     = 0x1BADB002
MULTIBOOT_PAGE_ALIGN = (1 << 0)
MULTIBOOT_MEMORY_INFO = (1 << 1)
MULTIBOOT_AOUT_KLUDGE = (1 << 16)  /* We provide load addresses */
MULTIBOOT_FLAGS     = MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO | MULTIBOOT_AOUT_KLUDGE
MULTIBOOT_CHECKSUM  = -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)
    .long MULTIBOOT_MAGIC
    .long MULTIBOOT_FLAGS
    .long MULTIBOOT_CHECKSUM
    /* AOUT_KLUDGE address fields */
    .long multiboot1_header_start_phys  /* header_addr: where this header is */
    .long 0x200000                      /* load_addr: where to load (2MB) */
    .long 0                             /* load_end_addr: 0 = load entire file */
    .long 0                             /* bss_end_addr: 0 = no BSS */
    .long _start_phys                   /* entry_addr: entry point */
multiboot1_header_end:

/* ============================================================================
 * Multiboot2 Header (for GRUB2)
 * Must be in first 32KB, 8-byte aligned
 * ============================================================================ */
.section .multiboot2, "a"
.align 8

.global multiboot2_header_start
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

    /* Entry address tag */
    .align 8
    .short 3                            /* Type: entry address */
    .short 0                            /* Flags */
    .long 12                            /* Size */
    .long _start_phys                   /* Entry address */

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

    /* Save multiboot magic and info pointer */
    mov edi, eax                        /* Magic in EDI */
    mov esi, ebx                        /* Multiboot info pointer in ESI */

    /* Check for multiboot2 magic */
    cmp eax, 0x36D76289
    je .magic_ok

    /* Check for multiboot1 magic (QEMU -kernel) */
    cmp eax, 0x2BADB002
    je .magic_ok

    /* Unknown magic - hang */
    jmp .hang

.magic_ok:
    /* Set up stack (in low memory .data.boot) */
    mov esp, offset boot_stack_top_phys

    /* TEAM_278: Save multiboot args to stack BEFORE setup_early_page_tables (clobbers EDI) */
    push esi                            /* Save info ptr */
    push edi                            /* Save magic */

    /* Load GDT64 */
    /* We need to fill the physical base address at runtime since we can't 
     * use a 64-bit relocation in a 32-bit field. */
    mov eax, offset GDT64_phys
    mov dword ptr [GDT64_Pointer_32_phys + 2], eax
    lgdt [GDT64_Pointer_32_phys]

    /* Setup page tables */
    call setup_early_page_tables

    /* TEAM_278: Restore multiboot args after page table setup */
    pop edi                             /* Restore magic */
    pop esi                             /* Restore info ptr */

    /* Enable PAE (Physical Address Extension) in CR4 */
    mov eax, cr4
    or eax, 0x20                        /* Bit 5: PAE */
    mov cr4, eax

    /* Set PML4 address in CR3 */
    mov eax, offset early_pml4_phys
    mov cr3, eax

    /* Enable long mode in EFER MSR */
    mov ecx, 0xC0000080                 /* EFER MSR */
    rdmsr
    or eax, 0x100                       /* Bit 8: LME (Long Mode Enable) */
    wrmsr

    /* Enable paging and protected mode in CR0 */
    mov eax, cr0
    or eax, 0x80000001                  /* Bit 31: PG, Bit 0: PE */
    mov dword ptr [esp], 0x08           /* Seg selector */
    mov dword ptr [esp + 4], offset long_mode_start_phys
    mov cr0, eax

    /* Far jump to 64-bit code segment */
    ljmp [esp]

.hang:
    hlt
    jmp .hang

/* ============================================================================
 * 64-bit Entry Point (UoW 2.6)
 * ============================================================================ */
.section .text.boot, "ax"
.code64

.global _start64
_start64:
    /* TEAM_284: Direct serial output 'L' for 'Limine Entered' */
    mov dx, 0x3f8
    mov al, 'L'
    out dx, al

    /* 1. Load our own GDT (virtual address) */
    movabs rax, offset GDT64_Pointer
    lgdt [rax]

    /* 2. Reload CS and jump to known state */
    mov rax, 0x08
    push rax
    movabs rax, offset .reload_cs
    push rax
    lretq

.reload_cs:
    /* 3. Set data segments */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* 4. Set our own stack (virtual address) */
    movabs rsp, offset stack_top

    /* 5. Direct serial output 'H' for 'Higher Half' */
    mov dx, 0x3f8
    mov al, 'H'
    out dx, al

    /* 6. Setup page tables using virtual addresses */
    /* Limine has mapped us, so we can write to our tables via virtual pointers */
    movabs rax, offset setup_early_page_tables_64
    call rax

    /* 7. Load our own CR3 (must be physical) */
    /* TEAM_285: Reloading CR3 here in the Limine path is causing a hang.
     * Limine has already provided us with valid page tables and a higher-half direct map.
     * We will stay on Limine's page tables until we reach Rust, then we will
     * switch to our own tables once we've properly initialized them. */
    /*
    movabs rax, offset early_pml4_phys
    mov cr3, rax
    */

    /* 8. Direct serial output 'C' for 'CR3 load skipped (safe)' */
    mov dx, 0x3f8
    mov al, 'C'
    out dx, al

    /* 9. Jump to common initialization */
    xor rdi, rdi                        /* magic = 0 for Limine */
    xor rsi, rsi                        /* info = 0 for now (Limine doesn't use this) */
    
    movabs rax, offset .call_rust
    jmp rax

.call_rust:
    /* 10. Call Rust kernel_main */
    movabs rax, offset kernel_main
    call rax

    /* If kernel_main returns, halt */
    jmp .halt

.global long_mode_start
long_mode_start:
    /* TEAM_284: This is the common 64-bit entry point for Multiboot paths.
     * We are already in 64-bit mode, identity mapped. */
    
    /* 1. Transition to higher-half address space */
    movabs rax, offset .higher_half
    jmp rax

.higher_half:
    /* TEAM_284: Direct serial output 'H' for 'Higher Half' */
    mov dx, 0x3f8
    mov al, 'H'
    out dx, al

    /* Reset segment registers to data segment */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Set up 64-bit stack in higher-half */
    movabs rsp, offset stack_top

    /* TEAM_278: Save multiboot args before BSS zeroing (rep stosq clobbers RDI) */
    mov r12d, edi                       /* Save magic to R12 (callee-saved) */
    mov r13d, esi                       /* Save info ptr to R13 (callee-saved) */

    /* Zero out BSS */
    movabs rdi, offset __bss_start
    movabs rcx, offset __bss_end
    sub rcx, rdi
    shr rcx, 3                          /* Divide by 8 (qwords) */
    xor rax, rax
    rep stosq

    /* Prepare arguments for kernel_main */
    mov edi, r12d                       /* Restore magic from R12 */
    mov esi, r13d                       /* Restore info ptr from R13 */

    /* Call Rust kernel_main */
    movabs rax, offset kernel_main
    call rax

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

/* ============================================================================
 * Early Page Tables (UoW 2.5)
 * Placed in .data.boot so they are identity mapped (low physical addresses)
 * ============================================================================ */
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
 * Page table initialization
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

/* 64-bit version for Limine path */
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
