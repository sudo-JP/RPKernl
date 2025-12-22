use core::clone::Clone;
use core::marker::Copy;

#[repr(C)]
pub enum ProcessState {
    Ready, 
    Running, 
    Blocked,
}

impl Clone for ProcessState {
    fn clone(&self) -> Self {
        match self {
            ProcessState::Ready => ProcessState::Ready,
            ProcessState::Running => ProcessState::Running,
            ProcessState::Blocked => ProcessState::Blocked,
        }
    }
}

impl Copy for ProcessState {}

#[repr(C)]
pub struct PCB {
    pub sp: *mut u32,           // Stack pointer, we on 32-bit arch 
    pub pid: u8, 
    pub state: ProcessState, 
    pub stack_base: *mut u8,    // Where stack allocation starts 
    pub stack_size: usize,      // Stack size, native size 
}

impl Clone for PCB {
    fn clone(&self) -> Self {
        PCB {
            sp: self.sp, 
            pid: self.pid,
            state: self.state,
            stack_base: self.stack_base, 
            stack_size: self.stack_size,
        }
    }
}

impl Copy for PCB {}
