
.section ".text.boot", "ax"
.global _head
_head:
    // Disable interrupts
    msr     daifset, #0xf

    // Setup stack
    ldr     x0, =0x40080000
    mov     sp, x0

    // Setup MAIR
    // Attr0: Normal WB-WA-RA (0xFF)
    // Attr1: Device nGnRE (0x04)
    ldr     x0, =0x00000000000004FF
    msr     mair_el1, x0

    // Setup TCR
    // IPS=5 (48-bit PA), TG0=0 (4KB), T0SZ=16 (48-bit VA for TTBR0)
    // EPD1=1 (disable TTBR1)
    mov     x0, #16
    orr     x0, x0, #(1 << 23) // EPD1
    mov     x1, #5
    lsl     x1, x1, #32
    orr     x0, x0, x1
    msr     tcr_el1, x0
    isb

    // Setup Page Tables
    // L0 @ 0x40100000
    // L1 @ 0x40101000
    // L2 @ 0x40102000
    // L1_high @ 0x40103000
    
    ldr     x0, =0x40100000
    mov     x1, #0x4000
1:  cbz     x1, 2f
    str     xzr, [x0], #8
    sub     x1, x1, #8
    b       1b
2:

    // --- Identity map 0x40080000 (RAM) ---
    // L0[0] -> L1
    ldr     x0, =0x40100000
    ldr     x1, =0x40101003
    str     x1, [x0]
    
    // L1[1] -> 0x40000000 (1GB Block)
    ldr     x0, =0x40101000
    mov     x1, #0x40000000
    add     x1, x1, #0x401 // Block, AF, Attr0
    str     x1, [x0, #8]

    // --- High VA mapping: 0x0000FFFF80000000 ---
    // L0 index: (0x0000FFFF80000000 >> 39) & 0x1FF = 511
    // L1 index: (0x0000FFFF80000000 >> 30) & 0x1FF = 510
    
    // L0[511] -> L1_high (0x40103000)
    ldr     x0, =0x40100000
    ldr     x1, =0x40103003
    str     x1, [x0, #511*8]
    
    // L1_high[510] -> 0x40000000 (1GB Block)
    ldr     x0, =0x40103000
    mov     x1, #0x40000000
    add     x1, x1, #0x401
    mov     x2, #(1 << 54) // UXN
    orr     x1, x1, x2
    str     x1, [x0, #510*8]

    // --- Identity map UART (0x09000000) ---
    // L1[0] -> L2_low (0x40102000)
    ldr     x0, =0x40101000
    ldr     x1, =0x40102003
    str     x1, [x0, #0]
    
    // UART is at 0x09000000. L2 index = (0x09000000 >> 21) = 72
    ldr     x0, =0x40102000
    mov     x1, #0x09000000
    add     x1, x1, #0x405 // Block, AF, Attr1 (Device)
    str     x1, [x0, #72*8]

    // Enable MMU
    ldr     x0, =0x40100000
    msr     ttbr0_el1, x0
    isb
    
    mrs     x0, sctlr_el1
    orr     x0, x0, #1      // M bit (MMU)
    orr     x0, x0, #4      // C bit (D-cache)
    orr     x0, x0, #0x1000 // I bit (I-cache)
    msr     sctlr_el1, x0
    isb

    // UART print 'A'
    mov     x0, #'A'
    ldr     x1, =0x09000000
    str     w0, [x1]

    // Jump to high address
    ldr     x0, =high_start
    br      x0

.section ".text.high", "ax"
high_start:
    // UART print 'B'
    mov     x0, #'B'
    ldr     x1, =0x09000000
    str     w0, [x1]
halt:
    wfe
    b       halt
