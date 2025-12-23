use core::clone::Clone;
use core::marker::Copy;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum ProcessState {
    Ready, 
    Running, 
    Blocked(BlockReason),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum BlockReason {
    Sleeping(u64),   // wake_time
    WaitingForWifi, 
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct PCB {
    pub sp: *mut u32,           // Stack pointer, we on 32-bit arch 
    pub pid: u8, 
    pub state: ProcessState, 
    pub stack_base: *mut u8,    // Where stack allocation starts 
    pub stack_size: usize,      // Stack size, native size 
}

