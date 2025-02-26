//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use clumsy_stm_bot::{self as _, drivers::sonar::Sonar};

use clumsy_stm_bot as _;
use defmt::debug;
use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
};
use embassy_sync::channel::{Receiver, Sender};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;

type MySonar<'a> = Sonar<Output<'a>, ExtiInput<'a>>;

type MyReceiver<'a> = Receiver<'a, ThreadModeRawMutex, u64, 1>;
type MySender<'a> = Sender<'a, ThreadModeRawMutex, u64, 1>;

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

static CHANNEL: Channel<ThreadModeRawMutex, u64, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led = Output::new(p.PA5, Level::High, Speed::High);
    spawner.spawn(blink(led)).unwrap();

    let sonar = Sonar::new(
        Output::new(p.PC0, Level::Low, Speed::High),
        ExtiInput::new(p.PC1, p.EXTI1, Pull::None),
    );
    let receiver = CHANNEL.receiver();
    let sender = CHANNEL.sender();

    spawner.must_spawn(read_sonar(sender, sonar));
    spawner.must_spawn(user(receiver));
}

#[embassy_executor::task]
async fn read_sonar(sender: MySender<'static>, mut sonar: MySonar<'static>) {
    loop {
        let distance_mm = sonar.read().await;
        sender.send(distance_mm).await;
        Timer::after_nanos(50).await;
    }
}

#[embassy_executor::task]
async fn user(receiver: MyReceiver<'static>) {
    loop {
        if let Some(distance_mm) = receiver.try_receive().ok() {
            if distance_mm <= 20 {
                defmt::info!("Closer");
            } else {
                defmt::info!("Further");
            }
            debug!("distance to obstacle: {}mm", distance_mm);
        }
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
