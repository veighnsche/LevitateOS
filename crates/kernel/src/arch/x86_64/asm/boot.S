/* TEAM_415: x86_64 Boot Assembly - Multiboot Headers and 32-bit Entry
 *
 * This file implements:
 * - UoW 2.2: Multiboot1 header (for QEMU -kernel)
 * - UoW 2.2: Multiboot2 header (for GRUB2)
 * - UoW 2.4: Protected-to-long mode transition
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

    /* Save multiboot magic and info pointer first to avoid clobbering */
    mov edi, eax                        /* Magic in EDI */
    mov esi, ebx                        /* Multiboot info pointer in ESI */

    /* TEAM_308: Diagnostic 'S' - 32-bit Entry (Start) */
    mov dx, 0x3f8
    mov al, 'S'
    out dx, al

    /* Check for multiboot2 magic */
    cmp edi, 0x36D76289
    je .magic_ok

    /* Check for multiboot1 magic (QEMU -kernel) */
    cmp edi, 0x2BADB002
    je .magic_ok

    /* Unknown magic - hang */
    jmp .hang

.magic_ok:
    /* TEAM_308: Diagnostic 'M' - Magic OK */
    mov dx, 0x3f8
    mov al, 'M'
    out dx, al

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

    /* TEAM_308: Diagnostic 'G' - GDT Loaded */
    mov dx, 0x3f8
    mov al, 'G'
    out dx, al

    /* Setup page tables */
    call setup_early_page_tables

    /* TEAM_308: Diagnostic 'P' - Page Tables Setup */
    mov dx, 0x3f8
    mov al, 'P'
    out dx, al

    /* TEAM_278: Restore multiboot args after page table setup */
    pop edi                             /* Restore magic */
    pop esi                             /* Restore info ptr */

    /* TEAM_358: Enable SSE/FPU in CR0 */
    /* Clear CR0.EM (bit 2) - no FPU emulation */
    /* Set CR0.MP (bit 1) - monitor coprocessor */
    mov eax, cr0
    and eax, 0xFFFFFFFB                 /* Clear bit 2 (EM) */
    or eax, 0x02                        /* Set bit 1 (MP) */
    mov cr0, eax

    /* Enable PAE and SSE in CR4 */
    mov eax, cr4
    or eax, 0x620                       /* Bit 5: PAE, Bit 9: OSFXSR, Bit 10: OSXMMEXCPT */
    mov cr4, eax

    /* Set PML4 address in CR3 */
    mov eax, offset early_pml4_phys
    mov cr3, eax

    /* TEAM_308: Diagnostic 'C' - CR3 Loaded */
    mov dx, 0x3f8
    mov al, 'C'
    out dx, al

    /* Enable long mode in EFER MSR */
    mov ecx, 0xC0000080                 /* EFER MSR */
    rdmsr
    or eax, 0x100                       /* Bit 8: LME (Long Mode Enable) */
    wrmsr

    /* TEAM_308: Diagnostic 'L' - LME Enabled */
    mov dx, 0x3f8
    mov al, 'L'
    out dx, al

    /* Enable paging and protected mode in CR0 */
    mov eax, cr0
    or eax, 0x80000001                  /* Bit 31: PG, Bit 0: PE */
    /* TEAM_308: Prepare far jump target on stack: [esp] = offset, [esp+4] = selector */
    mov dword ptr [esp], offset long_mode_start_phys
    mov dword ptr [esp + 4], 0x08       /* Seg selector */
    mov cr0, eax

    /* TEAM_308: Diagnostic 'E' - PG Enabled */
    mov dx, 0x3f8
    mov al, 'E'
    out dx, al

    /* Far jump to 64-bit code segment */
    ljmp [esp]

.hang:
    hlt
    jmp .hang
