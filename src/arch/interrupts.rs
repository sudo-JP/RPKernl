use cortex_m::interrupt::Mutex;
use cortex_m_rt::exception;
use rp2040_hal::{timer::{Alarm, Alarm0}};
use core::cell::RefCell;
use rp2040_hal::pac::interrupt;
use core::ptr;

use crate::{check_sleep_and_wake, scheduler::{CURRENT, PROCS, SCHEDULER}, switch_context_isr, Scheduler, PCB, QUANTUM, SLEEP_QUEUE};


static ALARM: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));

pub fn set_alarm(alarm: Alarm0) {
    cortex_m::interrupt::free(|cs| {
        ALARM.borrow(cs).replace(Some(alarm));
    });
}

/// Clear and reschedule the timer alarm
fn handle_alarm() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut alarm) = ALARM.borrow(cs).borrow_mut().as_mut() {
            alarm.clear_interrupt();
            let _ = alarm.schedule(QUANTUM);
        }
    });
}

/// Schedule next process from ISR context
/// Returns (old_sp_ptr, new_sp) or None if no switch needed
fn schedule_next_isr() -> Option<(*mut *mut u32, *const u32)> {
    let sched = ptr::addr_of_mut!(SCHEDULER);
    let sleep_q = ptr::addr_of_mut!(SLEEP_QUEUE);
    
    unsafe {
        let old_pid = CURRENT?;

        // wake up all sleeping processes 
        while (*sleep_q).get_size() > 0 {
            if check_sleep_and_wake().is_err() {
                break; 
            }
        }

        let next_pid = (*sched).dequeue().ok()?;

        let old_pcb: *mut PCB = PROCS[old_pid as usize].as_mut()?;
        match (*old_pcb).state {
            crate::ProcessState::Ready | crate::ProcessState::Running => {
                let _ = (*sched).enqueue(old_pid);
            }
            _ => {},
        }
        
        // Re-enqueue current process
        let _ = (*sched).enqueue(old_pid);
        
        if old_pid == next_pid {
            return None;
        }
        
        CURRENT = Some(next_pid);
        
        let new_pcb: *mut PCB = PROCS[next_pid as usize].as_mut()?;
        
        (*old_pcb).state = crate::ProcessState::Ready;
        (*new_pcb).state = crate::ProcessState::Running;
        
        let old_sp_ptr = ptr::addr_of_mut!((*old_pcb).sp);
        let new_sp = (*new_pcb).sp;
    
        Some((old_sp_ptr, new_sp))
    }
}

/// Inner handler called after R4-R11 are saved to PSP
/// Returns 1 if context switch happened, 0 if not
#[unsafe(no_mangle)]
extern "C" fn timer_irq_inner() -> u32 {
    handle_alarm();
    
    if let Some((old_sp_ptr, new_sp)) = schedule_next_isr() {
        unsafe {
            // Don't call switch_context_isr here - we'll do it in asm
            // Just store the values for the asm to use
            OLD_SP_PTR = old_sp_ptr;
            NEW_SP = new_sp;
        }
        1  // Signal that we need to switch
    } else {
        0  // No switch needed
    }
}

// Globals for passing context switch info from Rust to asm
#[unsafe(no_mangle)]
static mut OLD_SP_PTR: *mut *mut u32 = core::ptr::null_mut();
#[unsafe(no_mangle)]
static mut NEW_SP: *const u32 = core::ptr::null();

/// Timer interrupt - naked so we can save R4-R11 immediately
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn TIMER_IRQ_0() {
    core::arch::naked_asm!(
        // Get PSP and save R4-R11 immediately (before any Rust code)
        "mrs r0, psp",
        "subs r0, r0, #32",
        
        // Save R4-R7
        "str r4, [r0, #0]",
        "str r5, [r0, #4]",
        "str r6, [r0, #8]",
        "str r7, [r0, #12]",
        
        // Save R8-R11 (need to move to low reg first on M0)
        "mov r1, r8",
        "str r1, [r0, #16]",
        "mov r1, r9",
        "str r1, [r0, #20]",
        "mov r1, r10",
        "str r1, [r0, #24]",
        "mov r1, r11",
        "str r1, [r0, #28]",
        
        // Update PSP to include saved registers
        "msr psp, r0",
        
        // Now call Rust handler (R4-R11 are safely saved)
        "bl timer_irq_inner",
        
        // r0 = return value: 1 = switch, 0 = no switch
        "cmp r0, #0",
        "beq 1f",
        
        // === Context switch path ===
        // Load OLD_SP_PTR and NEW_SP from globals
        "ldr r0, =OLD_SP_PTR",
        "ldr r0, [r0]",
        "ldr r1, =NEW_SP", 
        "ldr r1, [r1]",
        
        // Store current PSP to *OLD_SP_PTR
        "mrs r2, psp",
        "str r2, [r0]",
        
        // Load new context: r1 = new SP pointing to R4-R11
        "ldr r4, [r1, #0]",
        "ldr r5, [r1, #4]",
        "ldr r6, [r1, #8]",
        "ldr r7, [r1, #12]",
        
        "ldr r0, [r1, #16]",
        "mov r8, r0",
        "ldr r0, [r1, #20]",
        "mov r9, r0",
        "ldr r0, [r1, #24]",
        "mov r10, r0",
        "ldr r0, [r1, #28]",
        "mov r11, r0",
        
        // Set PSP to exception frame (past R4-R11)
        "adds r1, r1, #32",
        "msr psp, r1",
        
        // Return via EXC_RETURN to new process
        "ldr r0, =0xFFFFFFFD",
        "bx r0",
        
        // === No switch path ===
        "1:",
        // Restore R4-R11 from PSP and return normally
        "mrs r0, psp",
        
        "ldr r4, [r0, #0]",
        "ldr r5, [r0, #4]",
        "ldr r6, [r0, #8]",
        "ldr r7, [r0, #12]",
        
        "ldr r1, [r0, #16]",
        "mov r8, r1",
        "ldr r1, [r0, #20]",
        "mov r9, r1",
        "ldr r1, [r0, #24]",
        "mov r10, r1",
        "ldr r1, [r0, #28]",
        "mov r11, r1",
        
        // Restore PSP to exception frame
        "adds r0, r0, #32",
        "msr psp, r0",
        
        // Normal return
        "ldr r0, =0xFFFFFFFD",
        "bx r0",
    );
}

/// PendSV handler - used for voluntary yield
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn PendSV() {
    core::arch::naked_asm!(
        // Get PSP and save R4-R11 immediately
        "mrs r0, psp",
        "subs r0, r0, #32",
        
        "str r4, [r0, #0]",
        "str r5, [r0, #4]",
        "str r6, [r0, #8]",
        "str r7, [r0, #12]",
        
        "mov r1, r8",
        "str r1, [r0, #16]",
        "mov r1, r9",
        "str r1, [r0, #20]",
        "mov r1, r10",
        "str r1, [r0, #24]",
        "mov r1, r11",
        "str r1, [r0, #28]",
        
        "msr psp, r0",
        
        // Call scheduler (reuse same inner function, but it won't touch alarm)
        "bl pendsv_inner",
        
        "cmp r0, #0",
        "beq 1f",
        
        // Context switch
        "ldr r0, =OLD_SP_PTR",
        "ldr r0, [r0]",
        "ldr r1, =NEW_SP",
        "ldr r1, [r1]",
        
        "mrs r2, psp",
        "str r2, [r0]",
        
        "ldr r4, [r1, #0]",
        "ldr r5, [r1, #4]",
        "ldr r6, [r1, #8]",
        "ldr r7, [r1, #12]",
        
        "ldr r0, [r1, #16]",
        "mov r8, r0",
        "ldr r0, [r1, #20]",
        "mov r9, r0",
        "ldr r0, [r1, #24]",
        "mov r10, r0",
        "ldr r0, [r1, #28]",
        "mov r11, r0",
        
        "adds r1, r1, #32",
        "msr psp, r1",
        
        "ldr r0, =0xFFFFFFFD",
        "bx r0",
        
        "1:",
        // No switch - restore and return
        "mrs r0, psp",
        
        "ldr r4, [r0, #0]",
        "ldr r5, [r0, #4]",
        "ldr r6, [r0, #8]",
        "ldr r7, [r0, #12]",
        
        "ldr r1, [r0, #16]",
        "mov r8, r1",
        "ldr r1, [r0, #20]",
        "mov r9, r1",
        "ldr r1, [r0, #24]",
        "mov r10, r1",
        "ldr r1, [r0, #28]",
        "mov r11, r1",
        
        "adds r0, r0, #32",
        "msr psp, r0",
        
        "ldr r0, =0xFFFFFFFD",
        "bx r0",
    );
}

#[unsafe(no_mangle)]
extern "C" fn pendsv_inner() -> u32 {
    if let Some((old_sp_ptr, new_sp)) = schedule_next_isr() {
        unsafe {
            OLD_SP_PTR = old_sp_ptr;
            NEW_SP = new_sp;
        }
        1
    } else {
        0
    }
}



