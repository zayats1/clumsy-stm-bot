//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use clumsy_stm_bot::{
    self as _,
    drivers::{
        line_sensor::{LinePos, TrippleLineSensor},
        motor::Motor,
    },
};

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{TIM2, TIM3},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_time::Timer;

use clumsy_stm_bot as _;

type LeftMotor<'a> = Motor<SimplePwmChannel<'a, TIM3>, Output<'a>, Output<'a>>;
type RightMotor<'a> = Motor<SimplePwmChannel<'a, TIM2>, Output<'a>, Output<'a>>;
type MyLineSensor<'a> = TrippleLineSensor<Input<'a>, Input<'a>, Input<'a>>;

const SPEED: i16 = 100;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PA5, Level::High, Speed::High);
    spawner.spawn(blink(led)).unwrap();

    let pwm_pin = PwmPin::new_ch2(p.PA7, OutputType::PushPull);

    let pwm = SimplePwm::new(
        p.TIM3,
        None,
        Some(pwm_pin),
        None,
        None,
        khz(10),
        Default::default(),
    );
    let mut ch2 = pwm.split().ch2;
    ch2.enable();

    let left_motor = Motor::new(
        ch2,
        Output::new(p.PB6, Level::Low, Speed::Low),
        Output::new(p.PC7, Level::Low, Speed::Low),
        0,` 1`
        Default::default(),
    );

    let pwm_pin = PwmPin::new_ch3(p.PB10, OutputType::PushPull);
    let pwm2 = SimplePwm::new(
        p.TIM2,
        None,
        None,
        Some(pwm_pin),
        None,
        khz(10),
        Default::default(),
    );
    let mut ch3 = pwm2.split().ch3;
    ch3.enable();

    let right_motor = Motor::new(
        ch3,
        Output::new(p.PA9, Level::Low, Speed::Low),
        Output::new(p.PA8, Level::Low, Speed::Low),
        0,
        Default::default(),
    );
    let line_sensor = TrippleLineSensor::new(
        Input::new(p.PB4, Pull::Down),
        Input::new(p.PB5, Pull::Down),
        Input::new(p.PB3, Pull::Down),
    );

    spawner.must_spawn(follow_line(line_sensor, left_motor, right_motor));
}

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn follow_line(
    mut sensor: MyLineSensor<'static>,
    mut left_motor: LeftMotor<'static>,
    mut right_motor: RightMotor<'static>,
) {
    loop {
        Timer::after_nanos(50).await;

        match sensor.read() {
            LinePos::NoLine => {
                left_motor.stop();
                right_motor.stop();
                continue;
            }
            LinePos::Lefter => {
                left_motor.run(SPEED/2);
                right_motor.run((SPEED as f32 / 1.25) as i16);
            }
            LinePos::Left => {
                left_motor.run(SPEED / 2);
                right_motor.run(SPEED);
            }
            LinePos::Middle => {
                left_motor.run(SPEED);
                right_motor.run(SPEED);
            }
            LinePos::Right => {
                left_motor.run(SPEED);
                right_motor.run(SPEED / 2);
            }
            LinePos::Righter => {
                left_motor.run((SPEED as f32 / 1.25) as i16);
                right_motor.run(SPEED/2);
            }
        };
    }
}
