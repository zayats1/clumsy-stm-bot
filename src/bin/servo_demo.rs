#![no_std]
#![no_main]

use clumsy_stm_bot as _;
use clumsy_stm_bot::drivers::servo::Servo;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

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

    info!("PWM initialized");

    let max_duty = ch3.max_duty_cycle();
    info!("PWM max duty {}", max_duty);

    let mut servo = Servo::new(ch3, 20u8, 180.0, max_duty);

    loop {
        for angle in 0..180 {
            servo.set_angle(angle as f32);
            Timer::after_millis(100).await;
        }
        for angle in (0..180).rev() {
            servo.set_angle(angle as f32);
            Timer::after_millis(100).await;
        }
    }
}
