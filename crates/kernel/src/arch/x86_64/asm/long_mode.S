/* TEAM_415: x86_64 Long Mode Entry Point
 *
 * This file implements:
 * - UoW 2.6: 64-bit entry point
 * - Limine entry path
 * - Higher-half transition
 */

.intel_syntax noprefix

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
    .byte 0x48, 0xcb  /* lretq - use raw encoding to avoid assembler deprecation warning */

.reload_cs:
    /* 3. Set data segments */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* TEAM_358: Enable SSE/FPU for Limine path */
    /* CR0: Clear EM (bit 2), set MP (bit 1), clear TS (bit 3) */
    mov rax, cr0
    and rax, ~(1 << 2)                  /* Clear EM */
    and rax, ~(1 << 3)                  /* Clear TS */
    or rax, (1 << 1)                    /* Set MP */
    mov cr0, rax

    /* CR4: Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10) */
    mov rax, cr4
    or rax, (1 << 9)                    /* OSFXSR */
    or rax, (1 << 10)                   /* OSXMMEXCPT */
    mov cr4, rax

    /* 4. Set our own stack (virtual address) */
    movabs rsp, offset stack_top
    and rsp, -16                        /* TEAM_299: Ensure 16-byte alignment */

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
    and rsp, -16                        /* TEAM_299: Ensure 16-byte alignment */

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
