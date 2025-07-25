#![no_std]
#![no_main]

use core::fmt::Write;

use clumsy_stm_bot::drivers::servo::Servo;
use clumsy_stm_bot::{self as _};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Output, OutputType, Pull, Speed};

use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usart::Uart;
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_stm32::{exti::ExtiInput, gpio::Level};
use embassy_time::{Delay, Duration, Instant, Timer};
use hcsr04_async::{DistanceUnit, Hcsr04, TemperatureUnit};
use heapless::String;
use num_traits::float::FloatCore;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

const FOV: usize = 180;
const RESOLUTION: usize = 20;
const TEMPERATURE: f64 = 22.0;

const DISTANCE_MEASURE_INTERVAL: Duration = Duration::from_millis(50);

// to run faster: cargo run --release --bin  clumsy-stm-bot
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

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
    let mut config = usart::Config::default();
    config.baudrate = 115200;
    let mut usart =
        Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH7, p.DMA1_CH6, config).unwrap();

    let config = hcsr04_async::Config {
        distance_unit: DistanceUnit::Centimeters,
        temperature_unit: TemperatureUnit::Celsius,
    };

    let mut sensor = Hcsr04::new(trigger, echo, config, clock, delay);

    // The temperature of the environment, if known, can be used to adjust the speed of sound.
    // If unknown, an average estimate must be used.

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

    let mut the_map = [(0, 0.0); FOV / RESOLUTION + 1];
    let mut s: String<2048> = String::new();
    loop {
        for (i, angle) in (0..=FOV)
            .step_by(RESOLUTION)
            .enumerate()
            .chain((0..FOV).step_by(RESOLUTION).enumerate().rev())
        {
            let distance = sensor.measure(TEMPERATURE).await;
            servo.set_angle(angle as f32);
            //  info!("angle {}", angle);

            match distance {
                Ok(distance) => {
                    // info!("Distance: {} cm", distance);
                    the_map[i] = (angle, (distance * 10.0).round() / 10.0);
                }
                Err(e) => {
                    info!("Error: {:?}", e);
                }
            }

            Timer::after(DISTANCE_MEASURE_INTERVAL).await;
        }
        // core::write!(&mut s, "[").unwrap();

        for (angle, distance) in the_map {
            core::write!(&mut s, "{},{};", angle, distance).unwrap();
        }

        // core::write!(&mut s, "]").unwrap();
        core::write!(&mut s, "\n").unwrap();
        unwrap!(usart.write(s.as_bytes()).await);
        s.clear();
        // println!("{:?}", the_map);
        //Timer::after(Duration::from_secs(1)).await;
    }
}
