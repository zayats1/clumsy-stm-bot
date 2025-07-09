#![no_std]
#![no_main]

use clumsy_stm_bot::drivers::servo::Servo;
use clumsy_stm_bot::{self as _};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Output, OutputType, Pull, Speed};

use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{exti::ExtiInput, gpio::Level};
use embassy_time::{Delay, Duration, Instant, Timer};
use hcsr04_async::{DistanceUnit, Hcsr04, TemperatureUnit};
use {defmt_rtt as _, panic_probe as _};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let trigger = Output::new(p.PC0, Level::Low, Speed::High);
    let echo = ExtiInput::new(p.PC1, p.EXTI1, Pull::Down);

    struct EmbassyClock;

    impl hcsr04_async::Now for EmbassyClock {
        fn now_micros(&self) -> u64 {
            Instant::now().as_micros()
        }
    }

    let clock = EmbassyClock;
    let delay = Delay;

    let config = hcsr04_async::Config {
        distance_unit: DistanceUnit::Centimeters,
        temperature_unit: TemperatureUnit::Celsius,
    };

    let mut sensor = Hcsr04::new(trigger, echo, config, clock, delay);

    // The temperature of the environment, if known, can be used to adjust the speed of sound.
    // If unknown, an average estimate must be used.
    let temperature = 24.0;

    let ch3_pin = PwmPin::new_ch3(p.PA10, OutputType::PushPull);
    let pwm = SimplePwm::new(
        p.TIM1,
        None,
        None,
        Some(ch3_pin),
        None,
        hz(50),
        Default::default(),
    );

    let mut ch3 = pwm.split().ch3;
    ch3.enable();

    let max_duty = ch3.max_duty_cycle();

    let mut servo = Servo::new(ch3, 20u8, 180.0, max_duty);

    loop {
        for angle in (0..=180).step_by(10).chain((0..180).step_by(10).rev()) {
            let distance = sensor.measure(temperature).await;
            servo.set_angle(angle as f32);
            info!("angle {}", angle);
            match distance {
                Ok(distance) => {
                    info!("Distance: {} cm", distance);
                }
                Err(e) => {
                    info!("Error: {:?}", e);
                }
            }
            Timer::after(Duration::from_millis(10)).await;
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
