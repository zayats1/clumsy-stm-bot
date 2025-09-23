//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use defmt::debug;
use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use clumsy_stm_bot::drivers::{line_sensor::LineSensor, motor::Motor};

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
type MyLineSensor<'a> = [LineSensor<Input<'a>>; 5];

const SPEED: f32 = 100.0;

const KP: f32 = 160.0;

const KI: f32 = 0.200;

const KD: f32 = 160.0;

const KA: f32 = 0.004; // reduction of the movement speed

const MIDDLE: f32 = 2.0;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let pwm_pin = PwmPin::new(p.PA7, OutputType::PushPull);

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
        0.0,
        Default::default(),
    );

    let pwm_pin = PwmPin::new(p.PB10, OutputType::PushPull);
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
        0.0,
        Default::default(),
    );
    let line_sensors = [
        LineSensor::new_invert(Input::new(p.PB0, Pull::Down)),
        LineSensor::new(Input::new(p.PB4, Pull::Down)),
        LineSensor::new(Input::new(p.PB5, Pull::Down)),
        LineSensor::new(Input::new(p.PB3, Pull::Down)),
        LineSensor::new_invert(Input::new(p.PA4, Pull::Down)),
    ];
    spawner.must_spawn(follow_line(line_sensors, left_motor, right_motor));

    let led = Output::new(p.PA5, Level::High, Speed::High);
    spawner.spawn(blink(led)).unwrap();
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
    mut sensors: MyLineSensor<'static>,
    mut left_motor: LeftMotor<'static>,
    mut right_motor: RightMotor<'static>,
) {
    let mut integral = 0.0f32;

    let mut prev_deviation = 0.0f32;
    loop {
        Timer::after_nanos(50).await;
        let deviation = {
            let mut sum = 0;
            let mut activated = 0;
            for (idx, sensor) in sensors.iter_mut().enumerate() {
                if sensor.is_on_line() {
                    activated += 1;
                    sum += idx;
                }
            }

            if activated == 5 {
                left_motor.stop();
                right_motor.stop();
                debug!("{}", "No line");
                continue;
            }

            if activated == 0 {
                prev_deviation
            } else {
                MIDDLE - sum as f32 / activated as f32
            }
        };

        debug!("{}", deviation);

        // let deviation = match line_pos {
        //     LinePos::NoLine => {
        //         left_motor.stop();
        //         right_motor.stop();
        //         continue;
        //     }
        //     LinePos::Lefter => -2.0,
        //     LinePos::Left => -1.0,
        //     LinePos::Middle => 0.0,
        //     LinePos::Right => 1.0,
        //     LinePos::Righter => 2.0,
        // };

        integral = (integral + deviation).clamp(-SPEED, SPEED);

        let diff = deviation - prev_deviation;

        let pid_val = KP * deviation + KI * integral + KD * diff;

        let attenuation = 1.0 - KA * deviation.abs();
        let left_speed = (SPEED - pid_val).clamp(-SPEED, SPEED) * attenuation;

        let right_speed = (SPEED + pid_val).clamp(-SPEED, SPEED) * attenuation;

        prev_deviation = deviation;

        left_motor.run(left_speed);

        right_motor.run(right_speed);
    }
}
