//! Rust wrapper around WCH's precompiled UHSIF driver (`libUHSIF.a`).
//!
//! UHSIF is the CH32H417's universal high-speed parallel interface. We use it
//! exactly like WCH's own logic-analyzer firmware does: "slave FPGA" mode with
//! an internally generated sample clock (PCLK on PC0), 8 data lines sampled
//! into a chain of 8 SRAM buffers of 16 KiB each ("slots").
//!
//! Data pins (fixed by hardware, see datasheet table 2-2-16):
//!   DATA[0..7] = PD10, PD11, PD12, PD13, PD14, PD15, PF0, PF1
//!
//! The driver library only exists as a compiled object; its interrupt handler
//! relies on WCH hardware interrupt stacking which Zephyr does not enable, so
//! instead of interrupts we poll the per-line slot register and re-arm
//! completed slots with `UHSIF_Trans_Cfg`, mirroring what the library's own
//! IRQ handler does.

#![allow(non_snake_case)]

/// Size of one capture slot in bytes (DEF_BUFF_SIZE in the library).
pub const SLOT_SIZE: usize = 16 * 1024;
/// Number of slots we arm (maximum supported by a line).
pub const SLOT_COUNT: usize = 8;

// UHSIF library API (see ch32h417_uhsif.h in the WCH EVT).
extern "C" {
    /// mode: 0=slave FPGA, 1=slave SOC, 2=master
    /// port_rm: 0..2 pin remap; clk_rm: 0..3 clock pin remap
    /// clk_div: RCC_UHSIFDIV value (division factor - 1)
    /// width_bit: 0=8bit, 1=16bit, 2=24bit, 3=32bit
    fn UHSIF_GPIO_Init(mode: u8, port_rm: u8, clk_rm: u8, clk_div: u8, width_bit: u8) -> u8;
    fn UHSIF_Cfg();
    /// line: 0..3; dir: 0=IN, 1=OUT; buff_count: 1..8; pbuf: buffer base
    fn UHSIF_Line_Cfg(line: u8, dir: u8, buff_count: u8, pbuf: *mut u8, water_mark: u16) -> u8;
    /// sta: 0=disable, 1=enable
    fn UHSIF_Start(sta: i32);
    /// Re-arm buffer `buff` of `line` so reception continues into it.
    fn UHSIF_Trans_Cfg(line: u8, buff: u8, trans_size: u32) -> u8;
}

const MODE_SLAVE_FPGA: u8 = 0;
const PIN_REMAP2: u8 = 2;
const CLK_REMAP1: u8 = 1;
const DATA_8BIT: u8 = 0;
const LINE_DIR_IN: u8 = 0;

/// RCC_UHSIFDIV_DIV4: PCLK = SYSCLK / 4 (= 100 MHz when SYSCLK is 400 MHz).
pub const UHSIF_DIV4: u8 = 0x03;

/// Initialize UHSIF for 8-bit parallel capture into `buf`.
///
/// Must be called while the system clock is the 400 MHz capture clock.
/// `clk_div` selects the PCLK division from SYSCLK (raw RCC_UHSIFDIV value).
pub fn init_capture(buf: *mut u8, clk_div: u8) -> Result<(), u8> {
    unsafe {
        UHSIF_Start(0);
        let r = UHSIF_GPIO_Init(MODE_SLAVE_FPGA, PIN_REMAP2, CLK_REMAP1, clk_div, DATA_8BIT);
        if r != 0 {
            return Err(r);
        }
        UHSIF_Cfg();
        let r = UHSIF_Line_Cfg(0, LINE_DIR_IN, SLOT_COUNT as u8, buf, 0);
        if r != 0 {
            return Err(r);
        }
    }
    Ok(())
}

/// Start sampling.
pub fn start() {
    unsafe { UHSIF_Start(1) }
}

/// Stop sampling.
pub fn stop() {
    unsafe { UHSIF_Start(0) }
}

/// Re-arm a completed slot so the hardware can write into it again.
#[inline]
pub fn rearm_slot(slot: usize) {
    unsafe {
        UHSIF_Trans_Cfg(0, slot as u8, 0);
    }
}
