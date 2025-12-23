use crate::scheduler::{Scheduler, MAX_PROCS};
use crate::{SchedulerError, PROCS};
use core::ptr; 

/*
 *
 * Timer register to save process requested time 
 *
 * */
static mut TIMER: *const rp2040_hal::Timer = ptr::null_mut();

pub unsafe fn register_timer(timer: &rp2040_hal::Timer) {
    unsafe {
        TIMER = timer as *const _;
    }
}

fn get_time_us() -> u64 {
    unsafe {
        if TIMER.is_null() {
            panic!("Timer not registered");
        }

        (*TIMER).get_counter().ticks()
    }
}

/*
 * Sleep queue
 * */
#[derive(Clone, Copy)]
pub struct SleepEntry {
    pid: u8, 
    wake_time: u64,
}

const DUMMY: SleepEntry = SleepEntry { pid: 0, wake_time: core::u64::MAX, };

pub struct SleepQueue {
    heap: [SleepEntry; MAX_PROCS],
    size: usize, 
}

impl SleepQueue {
    pub const fn new() -> Self {
        Self {
            heap: [DUMMY; MAX_PROCS], 
            size: 0, 
        }
    }

    fn parent(&self, idx: usize) -> usize {
        (idx - 1) >> 1 
    }

    fn left_child(&self, idx: usize) -> usize {
        2 * idx + 1
    }

    fn right_child(&self, idx: usize) -> usize {
        2 * idx + 2
    }

    fn bubble_up(&mut self, idx: usize) {
        let mut i = idx; 
        while i > 0 {
            let parent_idx = self.parent(i);  
            let parent: SleepEntry = self.heap[parent_idx];
            let child: SleepEntry = self.heap[i];

            if parent.wake_time > child.wake_time {
                self.heap.swap(parent_idx, i);
                // Move to parent 
                i = parent_idx; 
            } else {

                // Heap satisfied 
                break; 
            }
        }
    }

    fn bubble_down(&mut self, mut i: usize) {
        loop {
            let left_idx = self.left_child(i);
            let right_idx = self.right_child(i);
            let mut smallest = i;
            
            // Check if left child exists AND is smaller
            if left_idx < self.size && self.heap[left_idx].wake_time < self.heap[smallest].wake_time {
                smallest = left_idx;
            }
            
            // Check if right child exists AND is smaller
            if right_idx < self.size && self.heap[right_idx].wake_time < self.heap[smallest].wake_time {
                smallest = right_idx;
            }
            
            // If current is smallest, heap property satisfied
            if smallest == i {
                break;
            }
            
            self.heap.swap(i, smallest);
            i = smallest;
        }
    }

    /*
     * Retrieve min from the heap, pop the min element 
     * */
    fn extract_min(&mut self) -> Result<SleepEntry, SchedulerError> {
        if self.size == 0 {
            return Err(SchedulerError::Empty);
        }
        let min_node = self.heap[0];

        // Perform bubble down here
        self.size -= 1;
        self.heap[0] = self.heap[self.size];
        self.bubble_down(0);

        Ok(min_node)
    }
}

impl Scheduler<SleepEntry> for SleepQueue {
    /*
     * Heap insertion
     * Assumed entry->wake_time is not defined when this function is called 
     * */
    fn enqueue(&mut self, node: SleepEntry) -> Result<(), SchedulerError> {
        if self.size == MAX_PROCS {
            return Err(SchedulerError::NoSpace);
        }

        // Heap insertion, size is equivalent to last element 
        let last_idx = self.size; 
        self.heap[last_idx] = node;

        // Bubble up 
        self.bubble_up(last_idx);

        self.size += 1; 
        Ok(())
    }
    

    fn dequeue(&mut self) -> Result<u8, SchedulerError> {
        if self.size == 0 {
            return Err(SchedulerError::Empty);
        }

        let now = get_time_us(); 
        if self.heap[0].wake_time > now {
            return Err(SchedulerError::NotRunnable);
        }

        Ok(self.extract_min()?.pid)
    }
}

