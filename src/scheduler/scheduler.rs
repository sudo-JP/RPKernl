use core::ptr;
use crate::{switch_context, scheduler::{CURRENT, PROCS, SCHEDULER}, PCB};

#[derive(Debug)]
pub enum SchedulerError {
    NoSpace, 
    Empty, 
    NoCurrent, 
    ProcessNotFound,
    NotRunnable, 
}

pub trait Scheduler {
    fn enqueue(&mut self, pid: u8) -> Result<(), SchedulerError>; 
    fn dequeue(&mut self) -> Result<u8, SchedulerError>;  
}

pub fn current() -> Option<u8> {
    unsafe { CURRENT }
}

pub fn yield_now() -> Result<(), SchedulerError> {
    unsafe {
        let sched = ptr::addr_of_mut!(SCHEDULER); 
        let old_pid = CURRENT.ok_or(SchedulerError::NoCurrent)?;
        let next_pid = (*sched).dequeue()?;

        if old_pid == next_pid {
            (*sched).enqueue(old_pid)?;
            return Ok(());
        }

        (*sched).enqueue(old_pid)?;
        CURRENT = Some(next_pid);
        
        let old_pcb: *mut PCB = PROCS[old_pid as usize].as_mut().ok_or(SchedulerError::ProcessNotFound)?;
        let new_pcb: *mut PCB = PROCS[next_pid as usize].as_mut().ok_or(SchedulerError::ProcessNotFound)?;

        (*old_pcb).state = crate::ProcessState::Ready;
        (*new_pcb).state = crate::ProcessState::Running;
        
        // Get pointers/values before switch
        let old_sp_ptr = ptr::addr_of_mut!((*old_pcb).sp);
        let new_sp = (*new_pcb).sp;
        
        // Single unified context switch - works for all cases!
        switch_context(old_sp_ptr, new_sp);
    }
    Ok(())
}
