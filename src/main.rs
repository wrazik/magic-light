//! Blinks an LED
//!
//! This assumes that a LED is connected to the pin assigned to `led`. (GPIO15)

#![no_std]
#![no_main]

use core::cell::RefCell;
use core::borrow::BorrowMut;
use esp32_hal::gpio::{Event, Gpio32, Input, PullDown};
use critical_section::Mutex;

use esp32_hal::{
    clock::ClockControl,
    gpio::IO,
    pac::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay,
    Rtc,
};
use esp_backtrace as _;
use xtensa_lx_rt::entry;

static REED_SWITCH: Mutex<RefCell<Option<Gpio32<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    // Set GPIO15 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio25.into_push_pull_output();
    let mut reed_switch = io.pins.gpio32.into_pull_down_input();
    reed_switch.listen(Event::FallingEdge);

    led.set_high().unwrap();
    critical_section::with(|cs| REED_SWITCH.borrow_ref_mut(cs).replace(reed_switch));
    esp32_hal::interrupt::enable(esp32_hal::pac::Interrupt::GPIO, esp32_hal::interrupt::Priority::Priority2).unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    loop {
        esp_println::println!("Debug");
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}

#[interrupt]
fn GPIO() {

}