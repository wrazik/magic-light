//! Blinks an LED
//!
//! This assumes that a LED is connected to the pin assigned to `led`. (GPIO15)

#![no_std]
#![no_main]

mod led_strip;
mod led;

use core::borrow::BorrowMut;
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::RefCell;
use critical_section::Mutex;
use esp32_hal::gpio::{Event, Gpio25, Gpio32, Input, Output, PullDown, PullUp, PushPull};

use esp32_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::{self, Peripherals, TIMG0},
    prelude::*,
    timer::{Timer0, Timer, TimerGroup},
    Delay, Rtc,
};
use esp_backtrace as _;
use xtensa_lx_rt::entry;

enum STATE {
    LOW,
    HIGH,
    NONE
}

static REED_SWITCH: Mutex<RefCell<Option<Gpio32<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Gpio25<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>>>>> = Mutex::new(RefCell::new(None));

static WAS_ON: AtomicBool = AtomicBool::new(false);


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks
    );
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();
    let mut timer0 = timer_group0.timer0;
    timer0.start(100000000u64.nanos());
    timer0.listen();
    critical_section::with(|cs| {
        TIMER0.borrow_ref_mut(cs).replace(timer0);
    });
    // Set GPIO15 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio25.into_push_pull_output();
    let mut reed_switch = io.pins.gpio32.into_pull_down_input();
    reed_switch.listen(Event::FallingEdge);

    led.set_high().unwrap();
    critical_section::with(|cs| {
        REED_SWITCH.borrow_ref_mut(cs).replace(reed_switch);
        LED.borrow_ref_mut(cs).replace(led);

    });

    esp32_hal::interrupt::enable(
        peripherals::Interrupt::TG0_T0_LEVEL,
        esp32_hal::interrupt::Priority::Priority1,
    )
    .unwrap();
    loop {}
}

#[interrupt]
fn TG0_T0_LEVEL() {
    critical_section::with(|cs| {
        let mut timer = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer.as_mut().unwrap();
        timer0.clear_interrupt();

        if WAS_ON.load(Ordering::Relaxed) {
            LED.borrow_ref_mut(cs).borrow_mut().as_mut().unwrap().set_high().unwrap();
            WAS_ON.store(false, Ordering::Relaxed);
            timer0.start(600u64.nanos());
        } else {
            LED.borrow_ref_mut(cs).borrow_mut().as_mut().unwrap().set_low().unwrap();
            WAS_ON.store(true, Ordering::Relaxed);
            timer0.start(300u64.nanos());
        }
    });
}

