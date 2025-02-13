#![no_main]
#![no_std]

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
