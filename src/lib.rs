#![no_main]
#![no_std]

use defmt_rtt as _;
use embassy_stm32 as _;
use panic_probe as _;

pub mod drivers;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;
    #[test]
    fn some_test() {
        //Todo actual unit tests
        assert!(true)
    }
}
