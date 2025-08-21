use defmt::debug;
use embedded_hal::digital::InputPin;

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct LineSensor<T: InputPin> {
    pin: T,
    invert: bool,
}

impl<T: InputPin> LineSensor<T> {
    pub fn new(pin: T) -> Self {
        Self { pin, invert: false }
    }
    pub fn new_invert(pin: T) -> Self {
        Self { pin, invert: true }
    }

    pub fn is_on_line(&mut self) -> bool {
        if self.invert {
            self.pin.is_low().unwrap()
        } else {
            self.pin.is_high().unwrap()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, defmt::Format, PartialEq)]
pub enum LinePos {
    #[default]
    NoLine,
    Lefter,
    Left,
    Middle,
    Right,
    Righter,
}

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct TrippleLineSensor<T: InputPin, U: InputPin, V: InputPin> {
    left: LineSensor<T>,
    middle: LineSensor<U>,
    right: LineSensor<V>,
}

impl<T, U, V> TrippleLineSensor<T, U, V>
where
    T: InputPin,
    U: InputPin,
    V: InputPin,
{
    pub fn new(left: T, middle: U, right: V) -> Self {
        Self {
            left: LineSensor::new(left),
            middle: LineSensor::new(middle),
            right: LineSensor::new(right),
        }
    }

    pub fn read(&mut self) -> LinePos {
        match (
            self.left.is_on_line(),
            self.middle.is_on_line(),
            self.right.is_on_line(),
        ) {
            (true, false, false) => {
                debug!("Lefter");
                LinePos::Lefter
            }
            (true, true, false) => {
                debug!("Left");
                LinePos::Left
            }
            (false, true, false) => {
                debug!("Middle");
                LinePos::Middle
            }
            (false, true, true) => {
                debug!("Right");
                LinePos::Right
            }
            (false, false, true) => {
                debug!("Righter");
                LinePos::Righter
            }
            _ => {
                debug!("NoLine");
                LinePos::NoLine
            }
        }
    }
}
