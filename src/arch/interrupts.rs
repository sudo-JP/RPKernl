use cortex_m::interrupt::Mutex;
use rp2040_hal::{fugit::ExtU32, timer::{Alarm, Alarm0}};
use core::cell::RefCell;
use rp2040_hal::pac::interrupt;
use core::ptr;

use crate::{scheduler::{CURRENT, PROCS, SCHEDULER}, switch_context_isr, Scheduler, PCB, QUANTUM};


static ALARM: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));

pub fn set_alarm(alarm: Alarm0) {
    cortex_m::interrupt::free(|cs| {
        ALARM.borrow(cs).replace(Some(alarm));
    });
}

/// Schedule next process from ISR context
/// Returns (old_sp_ptr, new_sp) or None if no switch needed
unsafe fn schedule_next_isr() -> Option<(*mut *mut u32, *const u32)> {
    let sched = ptr::addr_of_mut!(SCHEDULER);
    
    unsafe {
        let old_pid = CURRENT?;
        let next_pid = (*sched).dequeue().ok()?;
        
        // Re-enqueue current process
        let _ = (*sched).enqueue(old_pid);
        
        if old_pid == next_pid {
            return None;
        }
        
        CURRENT = Some(next_pid);
        
        let old_pcb: *mut PCB = PROCS[old_pid as usize].as_mut()?;
        let new_pcb: *mut PCB = PROCS[next_pid as usize].as_mut()?;
        
        (*old_pcb).state = crate::ProcessState::Ready;
        (*new_pcb).state = crate::ProcessState::Running;
        
        let old_sp_ptr = ptr::addr_of_mut!((*old_pcb).sp);
        let new_sp = (*new_pcb).sp;
    
        Some((old_sp_ptr, new_sp))
    }
}

#[interrupt]
fn TIMER_IRQ_0() {
    // Clear interrupt and reschedule alarm
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut alarm) = ALARM.borrow(cs).borrow_mut().as_mut() {
            alarm.clear_interrupt();
            let _ = alarm.schedule(QUANTUM); // 100ms time slice
        }
    });

    // Try to switch to next process
    unsafe {
        if let Some((old_sp_ptr, new_sp)) = schedule_next_isr() {
            switch_context_isr(old_sp_ptr, new_sp);
        }
    }
}

/// PendSV handler - used for voluntary yield
/// Same logic as timer interrupt but triggered by software
#[unsafe(no_mangle)]
pub unsafe extern "C" fn PendSV() {
    unsafe { if let Some((old_sp_ptr, new_sp)) = schedule_next_isr() {
        switch_context_isr(old_sp_ptr, new_sp);
    }}
}
