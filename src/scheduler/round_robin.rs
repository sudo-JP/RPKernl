use crate::scheduler::{Scheduler, MAX_PROCS};
use crate::{SchedulerError};

pub struct RR {
    queue: [Option<u8>; MAX_PROCS],
    head: usize, 
    tail: usize, 
    size: usize, 
}

impl RR {
    pub const fn new() -> Self {
        Self {
            queue: [None; MAX_PROCS], 
            head: 0, 
            tail: 0, 
            size: 0, 
        }
    }
}

impl Scheduler<u8> for RR {
    fn enqueue(&mut self, pid: u8) -> Result<(), SchedulerError> {
        if self.size == MAX_PROCS {
            return Err(SchedulerError::NoSpace);
        }

        self.size += 1; 
        self.queue[self.tail] = Some(pid); 
        self.tail = (self.tail + 1) % MAX_PROCS; 

        Ok(())
    }

    fn dequeue(&mut self) -> Result<u8, SchedulerError> {
        match self.queue[self.head] {
            Some(pid) => {
                self.queue[self.head] = None;
                self.size -= 1; 
                self.head = (self.head + 1) % MAX_PROCS; 
                Ok(pid)
            }, 
            None => Err(SchedulerError::Empty), 
        }
    }  
                                                           
}

