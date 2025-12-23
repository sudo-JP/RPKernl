pub mod context; 
pub mod interrupts;

pub use context::*;
pub use interrupts::*;
use rp2040_hal::fugit::MicrosDurationU32;

pub static QUANTUM: MicrosDurationU32 = MicrosDurationU32::micros(10_000);
