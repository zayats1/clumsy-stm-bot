use embassy_time::Instant;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct Sonar<T: OutputPin, U: OutputPin + InputPin + Wait> {
    trig_pin: T,
    sonar_pin: U,
}

impl<T, U> Sonar<T, U>
where
    T: OutputPin,
    U: OutputPin + InputPin + Wait,
{
    pub fn new(trig_pin: T, sonar_pin: U) -> Self {
        Self {
            trig_pin,
            sonar_pin,
        }
    }

    async fn trig(&mut self, delay_us: impl AsyncFn(u64) -> ()) {
        self.trig_pin.set_low().unwrap();
        delay_us(2).await;
        self.trig_pin.set_high().unwrap();
        delay_us(2).await;
        self.trig_pin.set_low().unwrap();
    }
    pub async fn read(&mut self, delay_us: impl AsyncFn(u64) -> ()) -> u64 {
        let before = Instant::now();
        self.trig(delay_us).await;
        self.sonar_pin.set_high().unwrap();
        self.sonar_pin.wait_for_high().await.unwrap();
        let after = Instant::now();
        let duration = after - before;
        let distance = (duration.as_micros() * 343 / 10000) / 2; //cm
        return distance;
    }
}
