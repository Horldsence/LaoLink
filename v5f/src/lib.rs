/*
 * CH32H417EVT V5F logic analyzer (Rust).
 *
 * 8 channels, up to 100 MSa/s, sampled by the UHSIF high-speed parallel
 * interface and streamed to a PC over USART8 (921600 8N1) using the SUMP
 * protocol — connect with PulseView/sigrok as "Openbench Logic Sniffer".
 *
 * Channel pins (fixed by the UHSIF peripheral):
 *   ch0..ch7 = PD10, PD11, PD12, PD13, PD14, PD15, PF0, PF1
 * The 100 MHz sample clock is output on PC0 (PCLK); it can be left open.
 *
 * The V5F core is woken by the V3F waker image; this application owns the
 * LED (status) and USART8 (protocol; no Zephyr console on it).
 */

#![no_std]
// The dt_cfgs build script emits cfgs for device tree nodes that may not
// exist in every build; silence the check-cfg warnings for those.
#![allow(unexpected_cfgs)]
// This firmware is mostly MMIO register access; every unsafe block is a
// volatile read/write of a hardware register documented in the CH32H417
// reference material, so per-block SAFETY comments would be noise.
#![allow(clippy::undocumented_unsafe_blocks)]

mod capture;
mod shims;
mod sump;
mod uart;
mod uhsif;

use capture::CaptureResult;
use sump::Sump;

/// LED blink period when idle (main-loop iterations).
const IDLE_BLINK_ITERS: u32 = 3_000_000;

#[no_mangle]
extern "C" fn rust_main() {
    // SAFETY: called exactly once at startup, before any other logging.
    unsafe {
        let _ = zephyr::set_logger();
    }

    uart::init();

    let mut led = Led::new();
    let mut sump = Sump::new();
    let mut iters: u32 = 0;

    loop {
        if let Some(result) = sump.poll() {
            match result {
                CaptureResult::Done => {}
                // Aborted/stalled: nothing was uploaded; the host recovers
                // via its own timeout/reset path.
                CaptureResult::Aborted | CaptureResult::Stalled => {}
            }
            led.off();
            iters = 0;
        }

        // Idle heartbeat.
        iters += 1;
        if iters >= IDLE_BLINK_ITERS {
            iters = 0;
            led.toggle();
        }
    }
}

/// Status LED on PB1 (board alias led0). Tolerates the LED being absent.
struct Led {
    #[cfg(dt = "aliases::led0")]
    dev: Option<zephyr::device::gpio::GpioPin>,
}

impl Led {
    fn new() -> Self {
        #[cfg(dt = "aliases::led0")]
        {
            let mut dev = zephyr::devicetree::aliases::led0::get_instance();
            if let Some(d) = dev.as_mut() {
                if d.is_ready() {
                    d.configure(zephyr::raw::ZR_GPIO_OUTPUT_ACTIVE);
                } else {
                    dev = None;
                }
            }
            Led { dev }
        }
        #[cfg(not(dt = "aliases::led0"))]
        {
            Led {}
        }
    }

    fn toggle(&mut self) {
        #[cfg(dt = "aliases::led0")]
        if let Some(d) = self.dev.as_mut() {
            d.toggle_pin();
        }
    }

    fn off(&mut self) {
        #[cfg(dt = "aliases::led0")]
        if let Some(d) = self.dev.as_mut() {
            d.set(false);
        }
    }
}
