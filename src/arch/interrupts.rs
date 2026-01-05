use cortex_m::interrupt::Mutex;
use rp2040_hal::{timer::{Alarm, Alarm0}};
use core::cell::RefCell;
use rp2040_hal::pac::interrupt;
use core::ptr;

use crate::{check_sleep_and_wake, scheduler::{CURRENT, PROCS, SCHEDULER}, Scheduler, PCB, QUANTUM, SLEEP_QUEUE};


static ALARM: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));

pub fn set_alarm(alarm: Alarm0) {
    cortex_m::interrupt::free(|cs| {
        ALARM.borrow(cs).replace(Some(alarm));
    });
}

// Clear and reschedule the timer alarm
#[unsafe(no_mangle)]
extern "C" fn handle_alarm() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut alarm) = ALARM.borrow(cs).borrow_mut().as_mut() {
            alarm.clear_interrupt();
            let _ = alarm.schedule(QUANTUM);
        }
    });
}


#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn TIMER_IRQ_0() {
    core::arch::naked_asm!(
        // Save r4-r11
        "mrs r0, psp", 
        "subs r0, r0, #32", 
        
        "str r4, [r0, #0]", 
        "str r5, [r0, #4]", 
        "str r6, [r0, #8]", 
        "str r7, [r0, #12]", 

        // temp reg
        "mov r1, r8",
        "str r1, [r0, #16]", 
        "mov r1, r9",
        "str r1, [r0, #20]", 
        "mov r1, r10",
        "str r1, [r0, #24]", 
        "mov r1, r11",
        "str r1, [r0, #28]", 

        // Save new PSP (points to saved R4)
        "msr psp, r0", 

        // Alarm handler
        "bl handle_alarm", 

        // Call to get new sp, result in r0 (null = no switch)
        "bl get_new_sp",

        // Check if null (no switch needed)
        "cmp r0, #0",
        "beq 1f",

        // Switch: r0 = new_sp, call setcontext
        "bl setcontext",

        // No switch: restore R4-R11 from PSP and return
        "1:",
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
        
        // Return via EXC_RETURN
        "ldr r0, =0xFFFFFFFD",
        "bx r0",
    );
}
