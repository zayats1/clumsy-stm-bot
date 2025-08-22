//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use defmt::debug;
use defmt_rtt as _;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use clumsy_stm_bot::drivers::{line_sensor::LineSensor, motor::Motor};

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_stm32::{
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    interrupt,
    peripherals::{TIM2, TIM3},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_time::{Delay, Duration, Instant, Timer};

use embassy_stm32::{
    self as _,
    interrupt::{InterruptExt, Priority},
};

use clumsy_stm_bot as _;
use hcsr04_async::{DistanceUnit, Hcsr04, TemperatureUnit};

type LeftMotor<'a> = Motor<SimplePwmChannel<'a, TIM3>, Output<'a>, Output<'a>>;
type RightMotor<'a> = Motor<SimplePwmChannel<'a, TIM2>, Output<'a>, Output<'a>>;

struct EmbassyClock;

impl hcsr04_async::Now for EmbassyClock {
    fn now_micros(&self) -> u64 {
        Instant::now().as_micros()
    }
}
use embassy_stm32::exti::ExtiInput;
type MySonar<'a> = Hcsr04<Output<'a>, ExtiInput<'a>, EmbassyClock, Delay>;

type Distance = f64;
type MyMutex = CriticalSectionRawMutex;
type MyReceiver<'a> = Receiver<'a, MyMutex, Distance, 1>;
type MySender<'a> = Sender<'a, MyMutex, Distance, 1>;

type MyLineSensor<'a> = [LineSensor<Input<'a>>; 5];

const TEMPERATURE: f64 = 25.0;

const SPEED: f32 = 100.0;

const MINIMUM_DISTANCE: f64 = 6.0; // cm

const KP: f32 = 180.0;

const KI: f32 = 0.264;

const KD: f32 = 66.0;

const KA: f32 = 0.082; // reduction of the movement speed

const MIDDLE: f32 = 2.0;

const SONAR_MEASURE_CYCLE: Duration = Duration::from_millis(6);

static CHANNEL: Channel<MyMutex, Distance, 1> = Channel::new();

static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();

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

    let receiver = CHANNEL.receiver();
    let sender = CHANNEL.sender();

    let trigger = Output::new(p.PC0, Level::Low, Speed::High);
    let echo = ExtiInput::new(p.PC1, p.EXTI1, Pull::None);

    let config = hcsr04_async::Config {
        distance_unit: DistanceUnit::Centimeters,
        temperature_unit: TemperatureUnit::Celsius,
    };

    let clock = EmbassyClock;
    let delay = Delay;
    interrupt::UART5.set_priority(Priority::P7);
    let mp_spawner = EXECUTOR_MED.start(interrupt::UART5);
    let sonar = Hcsr04::new(trigger, echo, config, clock, delay);

    // Medium-priority executor: UART5, priority level 7

    let led = Output::new(p.PA5, Level::High, Speed::High);

    mp_spawner.must_spawn(read_sonar(sender, sonar));
    spawner.must_spawn(blink(led));

    spawner.must_spawn(follow_line(receiver, line_sensors, left_motor, right_motor));
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
    receiver: MyReceiver<'static>,
    mut sensors: MyLineSensor<'static>,
    mut left_motor: LeftMotor<'static>,
    mut right_motor: RightMotor<'static>,
) {
    let mut integral = 0.0f32;

    let mut prev_deviation = 0.0f32;

    'main_loop: loop {
        Timer::after_nanos(50).await;
        let mut the_speed = SPEED;

        let distance_cm = receiver.receive().await; // Possible cause of slugginess
        if distance_cm <= MINIMUM_DISTANCE {
            left_motor.stop();
            right_motor.stop();

            //`` integral = 0.0;
            // prev_deviation = 0.0;
            debug!("{}", "Obstacle detected");
            continue 'main_loop;
        }
        if distance_cm > MINIMUM_DISTANCE && distance_cm <= MINIMUM_DISTANCE * 1.2 {
            debug!("{}", "Comming to the obstacle");
            the_speed /= 1.2;
        }

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

        integral = (integral + deviation).clamp(-the_speed, the_speed);

        let diff = deviation - prev_deviation;

        let pid_val = KP * deviation + KI * integral + KD * diff;

        let attenuation = 1.0 - KA * deviation.abs();
        let left_speed = (the_speed - pid_val).clamp(-the_speed, the_speed) * attenuation;

        let right_speed = (the_speed + pid_val).clamp(-the_speed, the_speed) * attenuation;

        prev_deviation = deviation;

        left_motor.run(left_speed);

        right_motor.run(right_speed);
    }
}

#[embassy_executor::task]
async fn read_sonar(sender: MySender<'static>, mut sonar: MySonar<'static>) {
    loop {
        let measurment = sonar.measure(TEMPERATURE).await;

        match measurment {
            Ok(distance) => {
                debug!("distance to obstacle: {}cm", distance);
                sender.send(distance).await;
            }
            Err(err) => defmt::error!("{:?}", err),
        };

        Timer::after(SONAR_MEASURE_CYCLE).await; // for sensor to catch up with the polling rate
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn UART5() {
    unsafe { EXECUTOR_MED.on_interrupt() }
}
