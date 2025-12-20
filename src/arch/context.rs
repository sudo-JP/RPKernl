use crate::process::*;
use core::ptr;

/*
Save old process context
1. Push R4-R11 onto current stack
2. Save current SP into old_pcb.sp
3. Load new_pcb.sp into SP register
4. Pop R4-R11 from new stack
5. Return (hardware will restore R0-R3, R12, LR, PC, PSR)

r0 correspond to old_pcb 
r1 correspond to new_pcb
*/
#[naked]
#[no_mangle]
pub unsafe extern "C" fn switch_context(old_pcb: *mut PCB, new_pcb: *const PCB) {
    core::arch::asm!(
        "push {{r4-r11}}",          // Save callee-saved registers
        "mov r2, sp",               // Get stack pointer 
        "str r2, [r0, #8]",         // Store the r2 (current sp) to old_pcb->sp
        "ldr r2, [r1, #8]",         // Load sp from new_pcb to r2 
        "mov sp, r2",               // Place new_pcb->sp to current sp 
        "pop {{r4-r11}}",           // Pop from callee
        "bx lr",                    // Return 
        options(noreturn), 
    );
}

