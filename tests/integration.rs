#![no_std]
#![no_main]

fn setup_log() {
    rtt_target::rtt_init_defmt!();
}

#[cfg(test)]
#[embedded_test::tests(setup=crate::setup_log())]
mod integration {

    use embassy_stm32 as _;

    #[test]
    fn tesxzt() {}
}
