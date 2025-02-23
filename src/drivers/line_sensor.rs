use defmt::debug;
use embedded_hal::digital::InputPin;

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct LineSensor<T: InputPin> {
    pin: T,
}

impl<T: InputPin> LineSensor<T> {
    pub fn new(pin: T) -> Self {
        Self { pin }
    }

    pub fn is_on_line(&mut self) -> bool {
        self.pin.is_high().unwrap()
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
        return if self.left.is_on_line() && !self.middle.is_on_line() && !self.right.is_on_line() {
            debug!("Lefter");
            LinePos::Lefter
        } else if self.left.is_on_line() && self.middle.is_on_line() && !self.right.is_on_line() {
            debug!("Left");
            LinePos::Left
        } else if !self.left.is_on_line() && self.middle.is_on_line() && !self.right.is_on_line() {
            debug!("Middle");
            LinePos::Middle
        } else if !self.left.is_on_line() && self.middle.is_on_line() && self.right.is_on_line() {
            debug!("Right");
            LinePos::Right
        } else if !self.left.is_on_line() && !self.middle.is_on_line() && self.right.is_on_line() {
            debug!("Righter");
            LinePos::Righter
        } else {
            debug!("NoLine");
            LinePos::NoLine
        };
    }
}
