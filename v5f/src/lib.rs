/*
 * CH32H417EVT V5F application (Rust): blink the on-board LED and print on
 * USART8.
 *
 * The V5F core is woken by the V3F waker image; this application owns the
 * LED and the console UART.
 */

#![no_std]
// The dt_cfgs build script emits cfgs for device tree nodes that may not
// exist in every build; silence the check-cfg warnings for those.
#![allow(unexpected_cfgs)]

use log::info;

#[no_mangle]
extern "C" fn rust_main() {
    // SAFETY: called exactly once at startup, before any other logging.
    unsafe {
        zephyr::set_logger().unwrap();
    }

    info!("Hello from CH32H417EVT V5F!");

    do_blink();
}

#[cfg(dt = "aliases::led0")]
fn do_blink() -> ! {
    use zephyr::raw::ZR_GPIO_OUTPUT_ACTIVE;
    use zephyr::time::{sleep, Duration};

    let mut led0 = zephyr::devicetree::aliases::led0::get_instance().unwrap();

    if !led0.is_ready() {
        panic!("LED is not ready");
    }

    led0.configure(ZR_GPIO_OUTPUT_ACTIVE);
    let duration = Duration::millis_at_least(500);
    loop {
        led0.toggle_pin();
        info!("LED toggle");
        sleep(duration);
    }
}

#[cfg(not(dt = "aliases::led0"))]
fn do_blink() -> ! {
    panic!("No leds configured");
}
