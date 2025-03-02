use embedded_hal::pwm::SetDutyCycle;

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Servo<T: SetDutyCycle> {
    pwm_out: T,
    duty_on_zero: u16,
    duty_per_degree: f32,
    max_angle: f32,
}

impl<T: SetDutyCycle> Servo<T> {
    pub fn new(pwm_out: T, period: u8, max_angle: f32, max_duty: u16) -> Self {
        let duty_on_zero = max_duty / (period * 2) as u16; // servo pulse range
        let duty_on_90 = duty_on_zero * 3;
        let duty_per_degree = (duty_on_90 - duty_on_zero) as f32 / 90.0;
        Self {
            pwm_out,
            duty_on_zero,
            duty_per_degree,
            max_angle,
        }
    }

    pub fn set_angle(&mut self, angle: f32) {
        let mut angle = angle;

        if angle > self.max_angle {
            angle = self.max_angle
        }

        let duty_on_the_degree = (self.duty_per_degree * angle) as u16 + self.duty_on_zero;

        self.pwm_out.set_duty_cycle(duty_on_the_degree).unwrap();
    }
}
