pub mod scheduler;
pub mod round_robin;
pub mod sleep;

pub use scheduler::*;
pub use round_robin::*;
pub use sleep::*;

use crate::PCB;


const MAX_PROCS: usize = 256;

pub static mut SCHEDULER: RR = RR::new();
pub static mut PROCS: [Option<PCB>; MAX_PROCS] = [None; MAX_PROCS];
pub static mut CURRENT: Option<u8> = None; 
