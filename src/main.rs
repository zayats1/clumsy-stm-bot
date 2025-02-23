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
        sonar::Sonar,
    },
};

use clumsy_stm_bot as _;
use defmt::debug;
use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{TIM2, TIM3},
    time::hz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_sync::channel::{Receiver, Sender};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;

type LeftMotor<'a> = Motor<SimplePwmChannel<'a, TIM3>, Output<'a>, Output<'a>>;
type RightMotor<'a> = Motor<SimplePwmChannel<'a, TIM2>, Output<'a>, Output<'a>>;

type MySonar<'a> = Sonar<Output<'a>, ExtiInput<'a>>;
type MyLineSensor<'a> = TrippleLineSensor<Input<'a>, Input<'a>, Input<'a>>;

type MyReceiver<'a> = Receiver<'a, ThreadModeRawMutex, u64, 1>;
type MySender<'a> = Sender<'a, ThreadModeRawMutex, u64, 1>;

const SPEED: i16 = 100;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

static CHANNEL: Channel<ThreadModeRawMutex, u64, 1> = Channel::new();

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
        hz(200),
        Default::default(),
    );
    let mut ch2 = pwm.split().ch2;
    ch2.enable();

    let left_motor = Motor::new(
        ch2,
        Output::new(p.PB6, Level::Low, Speed::Low),
        Output::new(p.PC7, Level::Low, Speed::Low),
        0,
        Default::default(),
    );

    let pwm_pin = PwmPin::new_ch3(p.PB10, OutputType::PushPull);
    let pwm2 = SimplePwm::new(
        p.TIM2,
        None,
        None,
        Some(pwm_pin),
        None,
        hz(200),
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

    let sonar = Sonar::new(
        Output::new(p.PA2, Level::Low, Speed::High),
        ExtiInput::new(p.PA3, p.EXTI3, Pull::Down),
    );
    let receiver = CHANNEL.receiver();
    let sender = CHANNEL.sender();

    spawner.must_spawn(read_sonar(sender, sonar));
    spawner.must_spawn(roam(receiver, line_sensor, left_motor, right_motor));
}

#[embassy_executor::task]
async fn read_sonar(sender: MySender<'static>, mut sonar: MySonar<'static>) {
    loop {
        let distance_mm = sonar.read().await;
        debug!("distance to obstacle: {}mm", distance_mm);
        sender.send(distance_mm).await;
        Timer::after_nanos(50).await;
    }
}

#[embassy_executor::task]
async fn roam(
    receiver: MyReceiver<'static>,
    mut line_sensor: MyLineSensor<'static>,
    mut left: LeftMotor<'static>,
    mut right: RightMotor<'static>,
) {
    let speed = SPEED;
    loop {
        if line_sensor.read() != LinePos::NoLine {
            // Stumbled on Line
            left.stop();
            right.stop();
        }

        left.run(speed);
        right.run(speed);

        if let Some(distance_mm) = receiver.try_receive().ok() {
            if distance_mm <= 20 {
                left.stop();
                right.stop();
            }
        }
        Timer::after_nanos(50).await;
    }
}

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
