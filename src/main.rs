#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;
// Some traits we need
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use hal::pac;
use rp2040_hal::gpio::{Pin, FunctionSioOutput, PullDown};
use hal::gpio::bank0::{Gpio0, Gpio1};
use core::ptr;

use rp2040_scheduler::{create_process, start_first_process, yield_now, Scheduler, CURRENT, PROCS, SCHEDULER};

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;


/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;


type Led0 = Pin<Gpio0, FunctionSioOutput, PullDown>;
type Led1 = Pin<Gpio1, FunctionSioOutput, PullDown>;
pub static mut LED0: Option<Led0> = None;
pub static mut LED1: Option<Led1> = None;
pub static mut TIMER: Option<hal::Timer> = None;

#[rp2040_hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO0 as an output


    let led_pin0 = pins.gpio0.into_push_pull_output(); 
    let led_pin1 = pins.gpio1.into_push_pull_output(); 
    let stack_size = 1024; 
    unsafe {
        LED0 = Some(led_pin0);
        LED1 = Some(led_pin1);
        TIMER = Some(timer);

        create_process(stack_size, blink_fast, core::ptr::null_mut())
            .unwrap();
        create_process(stack_size, blink_slow, core::ptr::null_mut())
            .unwrap();

        if PROCS[0].is_none() || PROCS[1].is_none() {
        // Process creation failed - blink SOS
            loop {
                (*ptr::addr_of_mut!(LED0)).as_mut().unwrap().set_high().unwrap();
                (*ptr::addr_of_mut!(TIMER)).as_mut().unwrap().delay_ms(20);
                (*ptr::addr_of_mut!(LED0)).as_mut().unwrap().set_low().unwrap();
                (*ptr::addr_of_mut!(TIMER)).as_mut().unwrap().delay_ms(20);
            }
        }
        CURRENT = Some(0);
        start_first_process(PROCS[0].unwrap().sp);
    }
    
    loop {
        // TODO: This will eventually be your scheduler loop
        /*led_pin1.set_high();
        timer.delay_ms(500);
        led_pin1.set_high();
        timer.delay_ms(500);*/
    }
}
fn blink_fast(_arg: *mut ()) -> ! {    
    loop {
        unsafe {
            let led = ptr::addr_of_mut!(LED0)
                .cast::<Option<Led0>>()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap();
            
            let timer = ptr::addr_of_mut!(TIMER)
                .cast::<Option<hal::Timer>>()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap();
            
            led.set_high().unwrap();
            timer.delay_ms(500);
            led.set_low().unwrap();
            timer.delay_ms(500);
            yield_now().unwrap();
        }
    }
}

fn blink_slow(_arg: *mut ()) -> ! {
    loop {
        unsafe {
            let led = ptr::addr_of_mut!(LED0)
                .cast::<Option<Led1>>()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap();
            
            let timer = ptr::addr_of_mut!(TIMER)
                .cast::<Option<hal::Timer>>()
                .as_mut()
                .unwrap()
                .as_mut()
                .unwrap();
            
            led.set_high().unwrap();
            timer.delay_ms(1000);
            led.set_low().unwrap();
            timer.delay_ms(1000);
            yield_now().unwrap();
        }
    }
}

