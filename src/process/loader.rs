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

fn process_panic() -> ! {
    loop {}
}

/*
 * Stack structure
 * High Address (bottom of the stack since it grows downward)
 *
 * xPSR
 * PC
 * LR 
 * R12 
 * R3 
 * R2
 * R1
 * R0
 * R11 
 * R10
 * ...
 * R4
 * 
 * */
unsafe fn setup_initial_stack(stack_base: *mut u8, 
    stack_size: usize, entry: fn(*mut ()) -> !, arg: *mut()) -> *mut u32 {

    // SP pointing to top of the stack 
    let mut sp = (stack_base as usize + stack_size) as *mut u32;

    let xpsr_value: u32 = 0 | 1 << 24;
    unsafe {
        // xPSR 
        *sp = xpsr_value;
        sp = sp.offset(-1);

        // PC 
        *sp = (entry as u32) | 1; 
        sp = sp.offset(-1);

        // LR 
        *sp = process_panic as *const () as usize as u32;
        sp = sp.offset(-1);

        // R12, R3, R2, R1
        for _ in 0..4 {
            *sp = 0; 
            sp = sp.offset(-1);
        }

        // R0, the args
        *sp = arg as u32;
        sp = sp.offset(-1);
        
        // R11 to R4 
        for _ in 0..8 {
            *sp = 0; 
            sp = sp.offset(-1);
        }

        // SP now points below R4
        // Set it back to R4 
        sp = sp.offset(1);
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
