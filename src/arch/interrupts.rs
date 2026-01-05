use cortex_m::interrupt::Mutex;
use rp2040_hal::{timer::{Alarm, Alarm0}};
use core::cell::RefCell;
use rp2040_hal::pac::interrupt;
use core::ptr;

use crate::{check_sleep_and_wake, scheduler::{CURRENT, PROCS, SCHEDULER}, Scheduler, PCB, QUANTUM, SLEEP_QUEUE};


static ALARM: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));

pub fn set_alarm(alarm: Alarm0) {
    cortex_m::interrupt::free(|cs| {
        ALARM.borrow(cs).replace(Some(alarm));
    });
}

// Clear and reschedule the timer alarm
fn handle_alarm() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut alarm) = ALARM.borrow(cs).borrow_mut().as_mut() {
            alarm.clear_interrupt();
            let _ = alarm.schedule(QUANTUM);
        }
    });
}

#[interrupt]
fn TIMER_IRQ_0() {
    handle_alarm();  
    cortex_m::peripheral::SCB::set_pendsv();
}

