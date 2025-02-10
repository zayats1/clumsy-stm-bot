//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::u16;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use pico_bot::{self as _};

use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Level, Output, Pin};
use embassy_time::Timer;

use defmt::info;
use pico_bot as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    defmt::info!("Hello World!");
    defmt::println!("Hello");

    spawner.spawn(blink(p.PIN_25.degrade())).unwrap();

    spawner
        .spawn(semafor(
            p.PIN_17.degrade(),
            p.PIN_18.degrade(),
            p.PIN_16.degrade(),
        ))
        .unwrap();
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
async fn semafor(red_led: AnyPin, yellow_led: AnyPin, green_led: AnyPin) {
    let mut red_led = Output::new(red_led, Level::High);

    let mut yellow_led = Output::new(yellow_led, Level::High);

    let mut green_led = Output::new(green_led, Level::High);

    loop {
        info!("Red");
        red_led.set_high();
        yellow_led.set_low();
        green_led.set_low();
        Timer::after_secs(5).await;

        info!("Yellow");
        red_led.set_low();
        yellow_led.set_high();
        green_led.set_low();
        Timer::after_secs(3).await;

        info!("Blinking yellow");
        red_led.set_low();
        green_led.set_low();
        for _ in 0..5 {
            yellow_led.set_high();
            Timer::after_millis(125).await;
            yellow_led.set_low();
            Timer::after_millis(125).await;
        }

        info!("Green");
        red_led.set_low();
        yellow_led.set_low();
        green_led.set_high();
        Timer::after_secs(5).await;

        info!("Done");
        red_led.set_low();
        yellow_led.set_low();
        green_led.set_low();
    }
}
