use crate::{check_sleep_and_wake, Scheduler, CURRENT, PCB, PROCS, SCHEDULER, SLEEP_QUEUE};
use core::ptr;

#[unsafe(no_mangle)]
extern "C" fn get_new_sp() -> *const u32 {
    let psp: *mut u32 = cortex_m::register::psp::read() as *mut u32;
    let sched = ptr::addr_of_mut!(SCHEDULER);
    let sleep_q = ptr::addr_of_mut!(SLEEP_QUEUE);
    
    unsafe {
        let old_pid = CURRENT.unwrap();

        // wake up all sleeping processes 
        while (*sleep_q).get_size() > 0 {
            if check_sleep_and_wake().is_err() {
                break; 
            }
        }

        // Get new process
        let next_pid = (*sched).dequeue().ok().unwrap();

        let old_pcb: *mut PCB = PROCS[old_pid as usize].as_mut().unwrap();
        match (*old_pcb).state {
            crate::ProcessState::Ready | crate::ProcessState::Running => {
                let _ = (*sched).enqueue(old_pid);
            }
            _ => {},
        }

        (*old_pcb).sp = psp;
        
        if old_pid == next_pid {
            return (*old_pcb).sp as *const u32;
        }
        
        CURRENT = Some(next_pid);
        
        let new_pcb: *mut PCB = PROCS[next_pid as usize].as_mut().unwrap();
        
        (*old_pcb).state = crate::ProcessState::Ready;
        (*new_pcb).state = crate::ProcessState::Running;

        return (*new_pcb).sp as *const u32;
    }
}


/*
 * Function should never return, call to run first process given the sp 
 * */
pub fn start_first_process() -> () {
    let sched = core::ptr::addr_of_mut!(SCHEDULER); 
    unsafe {
        let pid = (*sched).dequeue().unwrap();
        let process = PROCS[pid as usize].unwrap();
        CURRENT = Some(pid);

        // This function should not return 
        run_first_process(process.sp);
    }

    #[allow(unreachable_code)]
    { panic!("Code should not reach here"); }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn run_first_process(sp: *const u32) -> ! {
    core::arch::naked_asm!(
        // Restore r4-r7 registers from SP (from function arg at r0)
        "ldr r4, [r0, #0]", 
        "ldr r5, [r0, #4]", 
        "ldr r6, [r0, #8]", 
        "ldr r7, [r0, #12]", 

        // Can't directly do ldr on r8-r11 because thumb only, whatever that means
        // We use r1 as temporary reg instead
        "ldr r1, [r0, #16]", 
        "mov r8, r1", 
        "ldr r1, [r0, #20]", 
        "mov r9, r1",
        "ldr r1, [r0, #24]", 
        "mov r10, r1",
        "ldr r1, [r0, #28]", 
        "mov r11, r1",

        // Advance sp to point to r0 
        "adds r0, r0, #32", 

        // Set the process sp to r0
        "msr psp, r0", 

        // Switch to thread mode by writing 1 to control reg
        // Add on bit index 1 (which is 2)
        "movs r1, #2",

        // Save back to control
        "msr CONTROL, r1", 

        // Sync barrier
        "isb", 

        // Restore special registers, pop from sp to r0-r3
        // Since in thread mode, and we set sp to psp, this is valid
        "pop {{r0-r3}}",
        "pop {{r4}}", 
        "mov r12, r4",

        "pop {{r4}}",       // LR  
        "mov lr, r4",

        "pop {{r4, r5}}",   // PC lives in r4 temporarily, discard r5

        "bx r4",
    );
}


/*
 *
 * Since we currently running kernel code, 
 * and want to jump to process code, we restore the 
 * first process registers, then run the process code.
 *
 * Literally setcontext from C 
 * */
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setcontext(sp: *const u32) -> ! {
    core::arch::naked_asm!(
        // Restore r4-r7 registers from SP (from function arg at r0)
        "ldr r4, [r0, #0]", 
        "ldr r5, [r0, #4]", 
        "ldr r6, [r0, #8]", 
        "ldr r7, [r0, #12]", 

        // Can't directly do ldr on r8-r11 because thumb only, whatever that means
        // We use r1 as temporary reg instead
        "ldr r1, [r0, #16]", 
        "mov r8, r1", 
        "ldr r1, [r0, #20]", 
        "mov r9, r1",
        "ldr r1, [r0, #24]", 
        "mov r10, r1",
        "ldr r1, [r0, #28]", 
        "mov r11, r1",

        // Advance sp to point to r0 
        "adds r0, r0, #32", 

        // Set the process sp to r0
        "msr psp, r0", 

        "ldr r0, =0xFFFFFFFD",
        "bx r0",
    );
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn PendSV() {
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

        // Save new r0 
        "msr psp, r0", 

        // Call to get new sp, result stores in r0 
        "bl get_new_sp",

        // set new context given r0
        "bl setcontext",
    );
}
