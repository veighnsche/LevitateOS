.global cpu_switch_to
cpu_switch_to:
    mov     x10, sp
    stp     x19, x20, [x0, #16 * 0]
    stp     x21, x22, [x0, #16 * 1]
    stp     x23, x24, [x0, #16 * 2]
    stp     x25, x26, [x0, #16 * 3]
    stp     x27, x28, [x0, #16 * 4]
    stp     x29, x30, [x0, #16 * 5]
    str     x10,      [x0, #16 * 6]
    mrs     x11,      tpidr_el0
    str     x11,      [x0, #16 * 7]

    ldp     x19, x20, [x1, #16 * 0]
    ldp     x21, x22, [x1, #16 * 1]
    ldp     x23, x24, [x1, #16 * 2]
    ldp     x25, x26, [x1, #16 * 3]
    ldp     x27, x28, [x1, #16 * 4]
    ldp     x29, x30, [x1, #16 * 5]
    ldr     x10,      [x1, #16 * 6]
    ldr     x11,      [x1, #16 * 7]
    msr     tpidr_el0, x11
    mov     sp, x10
    ret

.global task_entry_trampoline
task_entry_trampoline:
    bl      post_switch_hook
    mov     x0, #0
    blr     x19
    bl      task_exit
    b       .
