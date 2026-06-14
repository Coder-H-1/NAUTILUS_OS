.global vector_table

.macro ventry label
    .align 7
    b \label
.endm

.align 11
vector_table:
    // Current EL with SP0
    ventry sync_current_el_sp0
    ventry irq_current_el_sp0
    ventry fiq_current_el_sp0
    ventry serror_current_el_sp0

    // Current EL with SPx
    ventry sync_current_el_spx
    ventry irq_current_el_spx
    ventry fiq_current_el_spx
    ventry serror_current_el_spx

    // Lower EL using AArch64
    ventry sync_lower_el_aarch64
    ventry irq_lower_el_aarch64
    ventry fiq_lower_el_aarch64
    ventry serror_lower_el_aarch64

    // Lower EL using AArch32
    ventry sync_lower_el_aarch32
    ventry irq_lower_el_aarch32
    ventry fiq_lower_el_aarch32
    ventry serror_lower_el_aarch32

// We define simple handlers that save state and call the rust functions
sync_lower_el_aarch64:
    // Save trap frame on stack
    sub sp, sp, #272 // 34 registers * 8 bytes
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    
    // Pass pointer to TrapFrame (sp) in x0
    mov x0, sp
    
    mrs x1, esr_el1
    mrs x2, elr_el1
    mrs x3, far_el1
    bl sync_handler
    
    // Restore state
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp, #0]
    add sp, sp, #272
    eret

irq_lower_el_aarch64:
    bl irq_handler
    eret

// Dummy catch-alls
sync_current_el_sp0: b sync_current_el_sp0
irq_current_el_sp0: b irq_current_el_sp0
fiq_current_el_sp0: b fiq_current_el_sp0
serror_current_el_sp0: b serror_current_el_sp0
sync_current_el_spx: b sync_current_el_spx
irq_current_el_spx: b irq_current_el_spx
fiq_current_el_spx: b fiq_current_el_spx
serror_current_el_spx: b serror_current_el_spx
sync_lower_el_aarch32: b sync_lower_el_aarch32
irq_lower_el_aarch32: b irq_lower_el_aarch32
fiq_lower_el_aarch32: b fiq_lower_el_aarch32
serror_lower_el_aarch32: b serror_lower_el_aarch32
fiq_lower_el_aarch64: b fiq_lower_el_aarch64
serror_lower_el_aarch64: b serror_lower_el_aarch64
