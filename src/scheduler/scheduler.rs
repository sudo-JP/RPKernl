use core::ptr;
use crate::{scheduler::{CURRENT, PROCS, SCHEDULER}, PCB};

#[derive(Debug)]
pub enum SchedulerError {
    NoSpace, 
    Empty, 
    NoCurrent, 
    ProcessNotFound,
    NotRunnable, 
}

pub trait Scheduler<T> {
    fn enqueue(&mut self, entry: T) -> Result<(), SchedulerError>; 
    fn dequeue(&mut self) -> Result<u8, SchedulerError>;  
}

pub fn current() -> Option<u8> {
    unsafe { CURRENT }
}

/// Voluntary yield - triggers PendSV to do the actual context switch
/// This ensures we always switch in handler mode with proper exception frame
pub fn yield_now() -> Result<(), SchedulerError> {
    // Trigger PendSV - the PendSV handler will do the actual switch
    cortex_m::peripheral::SCB::set_pendsv();
    // PendSV is lowest priority, will run when we exit this function
    Ok(())
}


