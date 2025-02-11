use defmt::debug;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct Motor<T: SetDutyCycle, U: OutputPin, W: OutputPin> {
    pwm_pin: T,
    forward_pin: U,
    backward_pin: W,
    speed: i16,
    direction: Direction,
}

impl<T, U, W> Motor<T, U, W>
where
    T: SetDutyCycle,
    U: OutputPin,
    W: OutputPin,
{
    pub fn new(
        pwm_pin: T,
        forward_pin: U,
        backward_pin: W,
        speed: i16,
        direction: Direction,
    ) -> Self {
        Self {
            pwm_pin,
            forward_pin,
            backward_pin,
            speed,
            direction,
        }
    }

    pub fn set_dir(&mut self, dir: Direction) {
        debug!("direction = {}", self.direction);
        self.direction = dir;
    }

    pub fn get_dir(&mut self) -> Direction {
        self.direction
    }

    // Todo suppot for negative speed(invert dirrection)
    pub fn get_speed(&self) -> i16 {
        self.speed
    }

    pub fn stop(&mut self) {
        debug!("stop");
        self.speed = 0;
        let _ = self.pwm_pin.set_duty_cycle_fully_off();
        let (_, _) = (self.forward_pin.set_low(), self.backward_pin.set_high());
    }

    pub fn run(&mut self, speed: i16) {
        if speed < 0 {
            self.speed = speed * (-1);
            self.set_dir(Direction::Backward);
        } else {
            self.set_dir(Direction::Forward);
            self.speed = speed;
        }

        if self.speed > 100 {
            self.speed = 100;
        }

        let (_, _) = match self.direction {
            Direction::Forward => (self.forward_pin.set_high(), self.backward_pin.set_low()),
            Direction::Backward => (self.forward_pin.set_low(), self.backward_pin.set_high()),
        };
        self.pwm_pin
            .set_duty_cycle_percent(self.speed as u8)
            .unwrap();
    }
}
