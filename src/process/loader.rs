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

/// Set up initial stack for a new process
/// Uses exception frame format so ISR can switch to it via bx lr (EXC_RETURN)
/// 
/// Memory layout (address increasing downward in this diagram):
///   sp+0:  R4         <- SP points here (lowest address)
///   sp+4:  R5
///   sp+8:  R6
///   sp+12: R7
///   sp+16: R8
///   sp+20: R9
///   sp+24: R10
///   sp+28: R11
///   sp+32: R0 (arg)   <- exception frame starts here
///   sp+36: R1
///   sp+40: R2
///   sp+44: R3
///   sp+48: R12
///   sp+52: LR
///   sp+56: PC (entry)
///   sp+60: xPSR       <- (highest address, stack top before push)
///
unsafe fn setup_initial_stack(stack_base: *mut u8, 
    stack_size: usize, entry: fn(*mut ()) -> !, arg: *mut()) -> *mut u32 {
    // Start at top of stack
    let stack_top = (stack_base as usize + stack_size) as *mut u32;
    
    // We'll write 16 words total (8 for callee-saved + 8 for exception frame)
    // SP will point to bottom (lowest address)
    let sp = unsafe { stack_top.offset(-16) };

    unsafe {
        // Callee-saved registers at sp+0 to sp+28
        *sp.offset(0) = 0;  // R4
        *sp.offset(1) = 0;  // R5
        *sp.offset(2) = 0;  // R6
        *sp.offset(3) = 0;  // R7
        *sp.offset(4) = 0;  // R8
        *sp.offset(5) = 0;  // R9
        *sp.offset(6) = 0;  // R10
        *sp.offset(7) = 0;  // R11
        
        // Exception frame at sp+32 to sp+60
        *sp.offset(8) = arg as usize as u32;  // R0 = arg
        *sp.offset(9) = 0;   // R1
        *sp.offset(10) = 0;  // R2
        *sp.offset(11) = 0;  // R3
        *sp.offset(12) = 0;  // R12
        *sp.offset(13) = 0xFFFFFFFF;  // LR (dummy)
        *sp.offset(14) = (entry as usize as u32) | 1;  // PC
        *sp.offset(15) = 0x01000000;  // xPSR (Thumb bit)
    }

    sp
}


pub unsafe fn create_process(stack_size: usize, 
    entry: fn(* mut()) -> !, parg: *mut ()) -> Result<u8, ProcessError> {
    let stack_start = allocate_stack(stack_size)?;

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
