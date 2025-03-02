#![no_main]
#![no_std]

pub mod drivers;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

use defmt_rtt as _;
use embassy_stm32 as _;

use panic_probe as _;

use embedded_hal::digital::ErrorKind;
use embedded_hal::digital::{ErrorType, InputPin};

#[derive(Default)]
struct MockPin {
    state: bool,
}

impl MockPin {
    fn new(state: bool) -> Self {
        Self { state }
    }
}

impl ErrorType for MockPin {
    type Error = ErrorKind;
}

impl InputPin for MockPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.state)
    }
}

// run with cargo test --lib
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use super::*;
    use defmt::assert;
    #[test]
    fn line_sensor() {
        use crate::drivers::line_sensor::LineSensor;
        let mut sensor = LineSensor::new(MockPin::new(true));
        assert!(sensor.is_on_line());
        let new_pin = MockPin::new(false);
        let mut sensor2 = LineSensor::new(new_pin);
        assert!(!sensor2.is_on_line());
    }
}
