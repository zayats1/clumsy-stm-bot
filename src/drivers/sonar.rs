use embassy_time::{Delay, Instant, Timer};
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
};
use embedded_hal_async::digital::Wait;
#[derive(Clone)]
pub struct Sonar<T: OutputPin, U: InputPin + Wait> {
    trig_pin: T,
    sonar_pin: U,
    delay: Delay,
}

impl<T, U> Sonar<T, U>
where
    T: OutputPin,
    U: InputPin + Wait,
{
    pub fn new(trig_pin: T, sonar_pin: U) -> Self {
        Self {
            trig_pin,
            sonar_pin,
            delay: Delay,
        }
    }

    pub async fn read(&mut self) -> u64 {
        self.trig_pin.set_low().unwrap();
        self.delay.delay_us(2);
        self.trig_pin.set_high().unwrap();
        self.delay.delay_us(10);
        self.trig_pin.set_low().unwrap();
        let time = Instant::now();
        self.sonar_pin.wait_for_high().await.unwrap();
        //   self.sonar_pin.wait_for_low().await.unwrap();
        let duration = time.elapsed();
        let distance = duration.as_micros() + 4; // mm
        defmt::debug!("{:?}", duration);
        return distance;
    }
}
