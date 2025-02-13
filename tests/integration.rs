#![no_std]
#![no_main]

use embedded_hal::digital::ErrorKind;
use embedded_hal::digital::{ErrorType, InputPin};

fn setup_log() {
    rtt_target::rtt_init_defmt!();
}

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
        Ok(self.state)
    }
}

#[cfg(test)]
#[embedded_test::tests(setup=crate::setup_log())]
mod integration {
    use super::*;
    use clumsy_stm_bot::drivers::line_sensor::LineSensor;
    use embassy_stm32 as _;

    #[test]
    fn line_sensor() {
        let mut sensor = LineSensor::new(MockPin::new(true));
        assert!(sensor.is_on_line(), "{}", true);
        let mut sensor2 = LineSensor::new(MockPin::new(false));
        assert!(sensor.is_on_line(), "{}", false);
    }
}
