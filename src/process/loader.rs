use crate::{process::*, Scheduler, PROCS, SCHEDULER};
use crate::layout::MemoryLayout;
use core::ptr;

use core::result::Result;
use core::result::Result::{Ok, Err};


static mut NEXT_FREE: usize = 0; 
static mut ID: u8 = 0; 

#[derive(Debug)]
pub enum ProcessError {
    NoMemory, 
    InvalidSize, 
} 

/*
 * Return the start of the stack, given size
 * */
fn allocate_stack(size: usize) -> Result<*mut u8, ProcessError> {
    if size == 0 {
        return Err(ProcessError::InvalidSize);
    }
    let region = MemoryLayout::new(); 
    unsafe {
        if NEXT_FREE + size > region.processes.size {
            return Err(ProcessError::NoMemory);
        }
        else {
            let ptr: *mut u8 = (NEXT_FREE + region.processes.start) as *mut u8;
            NEXT_FREE += size; 
            return Ok(ptr); 
        }
    }
}

use crate::arch::process_trampoline;

/// Set up initial stack for a new process
/// Stack layout (growing down):
///   [High address]
///   LR (-> trampoline)
///   R7, R6, R5 (arg), R4 (entry)
///   R11, R10, R9, R8 (as R7-R4)
///   [Low address] <- SP points here
unsafe fn setup_initial_stack(stack_base: *mut u8, 
    stack_size: usize, entry: fn(*mut ()) -> !, arg: *mut()) -> *mut u32 {
    let mut sp = (stack_base as usize + stack_size) as *mut u32;

    unsafe {
        // LR - points to trampoline
        sp = sp.offset(-1);
        *sp = (process_trampoline as usize as u32) | 1; // Thumb bit

        // R4-R7: R4=entry, R5=arg, R6=0, R7=0
        sp = sp.offset(-1);
        *sp = 0; // R7
        sp = sp.offset(-1);
        *sp = 0; // R6
        sp = sp.offset(-1);
        *sp = arg as usize as u32; // R5 = arg
        sp = sp.offset(-1);
        *sp = (entry as usize as u32) | 1; // R4 = entry (with Thumb bit)

        // R8-R11 (stored as R4-R7 in push order)
        sp = sp.offset(-1);
        *sp = 0; // R11
        sp = sp.offset(-1);
        *sp = 0; // R10
        sp = sp.offset(-1);
        *sp = 0; // R9
        sp = sp.offset(-1);
        *sp = 0; // R8
    }

    sp
}


pub unsafe fn create_process(stack_size: usize, 
    entry: fn(* mut()) -> !, parg: *mut ()) -> Result<u8, ProcessError> {
    let stack_start = allocate_stack(stack_size)?;
    unsafe {
        core::ptr::write_volatile(stack_start, 0xAA);
        if core::ptr::read_volatile(stack_start) != 0xAA {
            loop {
                
            }
        }
    }

    unsafe {
        let sp = setup_initial_stack(stack_start, stack_size, entry, parg);
        let id: u8 = ID; 
        let pcb = PCB {
            sp: sp,
            pid: id, 
            state: ProcessState::Ready, 
            stack_base: stack_start, 
            stack_size: stack_size, 
        };
        PROCS[id as usize] = Some(pcb); 
        let sched = ptr::addr_of_mut!(SCHEDULER); 
        (*sched).enqueue(id).unwrap();
        ID += 1; 
        Ok(id)
    }
}
