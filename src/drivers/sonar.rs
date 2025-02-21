use embassy_time::{Instant, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct Sonar<T: OutputPin, U: InputPin + Wait> {
    trig_pin: T,
    sonar_pin: U,
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
        }
    }

    async fn trig(&mut self) {
        self.trig_pin.set_low().unwrap();
        Timer::after_micros(2).await;
        self.trig_pin.set_high().unwrap();
        Timer::after_micros(2).await;
        self.trig_pin.set_low().unwrap();
    }
    pub async fn read(&mut self) -> u64 {
        let time = Instant::now();
        self.trig().await;
        self.sonar_pin.wait_for_high().await.unwrap();
        let duration = time.elapsed();
        let distance = (duration.as_micros() * 343 / 10000) / 2; //cm
        return distance;
    }
}
