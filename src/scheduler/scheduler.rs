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

// Voluntary yield 
pub fn yield_now() -> Result<(), SchedulerError> {
    cortex_m::peripheral::SCB::set_pendsv();
    Ok(())
}


