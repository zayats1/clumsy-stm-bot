use embedded_hal::digital::{InputPin, OutputPin};

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct Sonar<T: OutputPin, U: OutputPin + InputPin> {
    sonar_pin: T,
    enable_pin: U,
}

impl<T: OutputPin, U: OutputPin + InputPin> Sonar<T, U> {
    pub fn new(sonar_pin: T, enable_pin: U) -> Self {
        Self { sonar_pin, enable_pin }
    }
    pub fn read(&self) -> u16 {
       todo!()
    }
}
