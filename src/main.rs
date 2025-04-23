#![no_std]
#![no_main]

use clumsy_stm_bot::{
    self as _,
    drivers::{
        line_sensor::{LinePos, TrippleLineSensor},
        motor::Motor,
        servo::Servo,
    },
};

use defmt_rtt as _;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_stm32::{
    self as _,
    interrupt::{InterruptExt, Priority},
};
use embassy_stm32::{interrupt, peripherals::TIM1};
use panic_probe as _;

use defmt::debug;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, OutputType, Pull, Speed},
    peripherals::{TIM2, TIM3},
    time::hz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_sync::channel::Channel;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Receiver, Sender},
};
use embassy_time::{Delay, Duration, Instant, Timer};
use hcsr04_async::{DistanceUnit, Hcsr04, TemperatureUnit};

type LeftMotor<'a> = Motor<SimplePwmChannel<'a, TIM3>, Output<'a>, Output<'a>>;
type RightMotor<'a> = Motor<SimplePwmChannel<'a, TIM2>, Output<'a>, Output<'a>>;

struct EmbassyClock;

impl hcsr04_async::Now for EmbassyClock {
    fn now_micros(&self) -> u64 {
        Instant::now().as_micros()
    }
}

type MySonar<'a> = Hcsr04<Output<'a>, ExtiInput<'a>, EmbassyClock, Delay>;
type MyLineSensor<'a> = TrippleLineSensor<Input<'a>, Input<'a>, Input<'a>>;

type Distance = f64;
type MyMutex = CriticalSectionRawMutex;
type MyReceiver<'a> = Receiver<'a, MyMutex, Distance, 1>;
type MySender<'a> = Sender<'a, MyMutex, Distance, 1>;
type MyServo<'a> = Servo<SimplePwmChannel<'a, TIM1>>;

// The temperature of the environment, if known, can be used to adjust the speed of sound.
// If unknown, an average estimate must be used.
const TEMPERATURE: f64 = 25.0;

const SPEED: i16 = 100;

const MINIMUM_DISTANCE: f64 = 18.0; // cm

const SONAR_MEASURE_CYCLE: Duration = Duration::from_millis(60);

static CHANNEL: Channel<MyMutex, Distance, 1> = Channel::new();

static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

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

    let sonar = Hcsr04::new(trigger, echo, config, clock, delay);

    // Medium-priority executor: UART5, priority level 7
    interrupt::UART5.set_priority(Priority::P7);
    let mp_spawner = EXECUTOR_MED.start(interrupt::UART5);

    let led = Output::new(p.PA5, Level::High, Speed::High);

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

    let servo = Servo::new(ch3, 20u8, 180.0, max_duty);

    spawner.spawn(blink(led)).unwrap();
    mp_spawner.must_spawn(read_sonar(sender, sonar));

    spawner.must_spawn(roam(receiver, line_sensor, servo, left_motor, right_motor));
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

#[embassy_executor::task]
async fn roam(
    receiver: MyReceiver<'static>,
    mut line_sensor: MyLineSensor<'static>,
    mut servo: MyServo<'static>,
    mut left: LeftMotor<'static>,
    mut right: RightMotor<'static>,
) {
    let speed = SPEED;

    // center the sonar
    servo.set_angle(90.0);
    loop {
        if line_sensor.read() != LinePos::NoLine {
            // Stumbled on Line
            left.stop();
            right.stop();
        }

        let distance_cm = receiver.receive().await;
        if distance_cm >= MINIMUM_DISTANCE {
            left.run(speed);
            right.run(speed);
        } else {
            left.stop();
            right.stop();

            servo.set_angle(180.0);
            Timer::after_millis(300).await;
            let distance_cm_left = receiver.receive().await;
            servo.set_angle(0.0);
            Timer::after_millis(300).await;
            let distance_cm_right = receiver.receive().await;
            servo.set_angle(90.0);
            Timer::after_millis(300).await;

            if distance_cm_left <= MINIMUM_DISTANCE && distance_cm_right <= MINIMUM_DISTANCE {
                // turn back
                left.run(-speed);
                right.run(-speed);
            } else if distance_cm_left < distance_cm_right {
                // turn right
                left.run(speed);
                right.run(-speed);
            } else {
                // turn left
                left.run(-speed);
                right.run(speed);
            }
            Timer::after_millis(500).await;
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

#[allow(non_snake_case)]
#[interrupt]
unsafe fn UART5() {
    unsafe { EXECUTOR_MED.on_interrupt() }
}
