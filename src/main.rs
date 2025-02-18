// This file is for development and debuging

//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;

use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use clumsy_stm_bot::{self as _, drivers::motor::Motor};

use embassy_executor::Spawner;
use embassy_stm32::{
    self as _, bind_interrupts,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{TIM2, TIM3},
    time::khz,
    timer::{
        self,
        input_capture::{CapturePin, InputCapture},
        simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
    },
    Peripheral,
};
use embassy_time::Timer;

use clumsy_stm_bot as _;

type LeftMotor<'a> = Motor<SimplePwmChannel<'a, TIM3>, Output<'a>, Output<'a>>;
type RightMotor<'a> = Motor<SimplePwmChannel<'a, TIM2>, Output<'a>, Output<'a>>;
type MyLineSensor<'a> = TrippleLineSensor<Input<'a>, Input<'a>, Input<'a>>;

const SPEED: i16 = 100;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PA5, Level::High, Speed::High);
    spawner.spawn(blink(led)).unwrap();

    let pwm_pin = PwmPin::new_ch2(p.PA7, OutputType::PushPull);

    let pwm = SimplePwm::new(
        unsafe { p.TIM3.clone_unchecked() },
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

    let ch1 = CapturePin::new_ch1(p.PA6, Pull::Down);
    let ir_reader = InputCapture::new(
        p.TIM3,
        Some(ch1),
        None,
        None,
        None,
        Irqs,
        khz(38),
        Default::default(),
    );

    spawner.must_spawn(decode_ir(ir_reader));
    spawner.must_spawn(ride(left_motor, right_motor));
}

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}

bind_interrupts!(struct Irqs {
    TIM3 => timer::CaptureCompareInterruptHandler<TIM3>; //Todo write the exact timer
});

#[embassy_executor::task]
async fn decode_ir(mut ir: InputCapture<'static, TIM3>) {
    let mut prev_capture = 0i64;

    ir.set_input_capture_mode(
        timer::Channel::Ch1,
        timer::low_level::InputCaptureMode::BothEdges,
    );
    let mut bit_idx = 0;
    let mut temp_code = 0u32;

    loop {
        // info!("interrupt {}", it);
        ir.wait_for_rising_edge(timer::Channel::Ch1).await;

        let capture_value = ir.get_capture_value(timer::Channel::Ch1) as i64;
        let current_val = (capture_value - prev_capture).abs();
        //  info!("new capture! {}", capture_value);

        if current_val > 8000 {
            temp_code = 0;
            bit_idx = 0;
        } else if current_val > 1700 {
            temp_code |= 1 << (31 - bit_idx); // write 1
            bit_idx += 1;
        } else if current_val > 1000 {
            temp_code &= !(1 << (31 - bit_idx)); // write 0
            bit_idx += 1;
        }

        if bit_idx >= 4 {
            info!("new code!{}, as bin {:b}", temp_code, temp_code);
            bit_idx = 0;
        }

        Timer::after_nanos(50).await;
        prev_capture = capture_value;
    }
}

#[embassy_executor::task]
async fn ride(mut left_motor: LeftMotor<'static>, mut right_motor: RightMotor<'static>) {
    loop {
        Timer::after_nanos(50).await;
        // detect falling from the desk
    }
}
