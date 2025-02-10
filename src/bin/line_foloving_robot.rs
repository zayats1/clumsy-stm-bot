//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::{f32::consts::PI, u16};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use pico_bot::{
    self as _,
    conversions::angle_to_speed::angle_to_speed,
    drivers::{
        line_sensor::{LinePos, TrippleLineSensor},
        motor::Motor,
    },
};

use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{AnyPin, Input, Level, Output, Pin, Pull},
    pwm::{self, Pwm},
};
use embassy_time::Timer;

use defmt::info;
use pico_bot as _;

type MyMotor<'a> = Motor<Pwm<'a>, Output<'a>, Output<'a>>;
type MyLineSensor<'a> = TrippleLineSensor<Input<'a>, Input<'a>, Input<'a>>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    defmt::info!("Hello World!");
    defmt::println!("Hello");

    spawner.spawn(blink(p.PIN_25.degrade())).unwrap();

    let pwm_conf = pwm::Config::default();

    let pwm = Pwm::new_output_a(p.PWM_SLICE1, p.PIN_2, pwm_conf.clone());

    let left_motor: Motor<Pwm<'_>, Output<'_>, Output<'_>> = Motor::new(
        pwm,
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        0,
        Default::default(),
    );

    let pwm2 = Pwm::new_output_b(p.PWM_SLICE4, p.PIN_9, pwm_conf.clone());

    let right_motor: Motor<Pwm<'_>, Output<'_>, Output<'_>> = Motor::new(
        pwm2,
        Output::new(p.PIN_7, Level::Low),
        Output::new(p.PIN_8, Level::Low),
        0,
        Default::default(),
    );
    let line_sensor = TrippleLineSensor::new(
        Input::new(p.PIN_21, Pull::Up),
        Input::new(p.PIN_19, Pull::Up),
        Input::new(p.PIN_18, Pull::Up),
    );

    spawner.must_spawn(follow_line(line_sensor, left_motor, right_motor));
}

#[embassy_executor::task]
async fn blink(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::High);
    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn follow_line(
    mut sensor: MyLineSensor<'static>,
    mut left_motor: MyMotor<'static>,
    mut right_motor: MyMotor<'static>,
) {
    let speed = 80; // reduce speed
    loop {
        //for controller to not halt
        Timer::after_micros(1).await;
        let angle: f32 = match sensor.read() {
            LinePos::NoLine => {
                left_motor.stop();
                right_motor.stop();
                continue;
            }
            LinePos::Lefter => PI / 4.0,
            LinePos::Left => PI / 12.0,
            LinePos::Middle => 0.0,
            LinePos::Right => -PI / 12.0,
            LinePos::Righter => PI / 4.0,
        };

        let (left_speed, right_speed) = if angle < 0.0 {
            (angle_to_speed(speed as f32, angle), speed)
        } else {
            (speed, angle_to_speed(speed as f32, angle))
        };
        left_motor.run(left_speed);
        right_motor.run(right_speed);
    }
}
