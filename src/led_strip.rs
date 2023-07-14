
use core::cell::RefCell;
use critical_section::Mutex;
use esp32_hal::gpio::{Event, Gpio35, Gpio32, Input, Output, PullDown, PullUp, PushPull};
use crate::led::Led;


struct LedStrip {
    size: usize,

}


impl LedStrip {
    pub(crate) fn new(size: usize) -> Self {
        Self { size }
    }

    fn low(&self) {
        unimplemented!();
    }

    pub(crate) fn set_led(&self, index: usize, color: Led) {
        unimplemented!();

    }
}
