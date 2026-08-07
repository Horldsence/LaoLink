//! Minimal register-level USART8 driver.
//!
//! USART8 (TX=PB4, RX=PB3, both AF11) is wired to the on-board WCH-LinkE
//! virtual COM port. The Zephyr UART driver performs the initial pinmux and
//! baud setup from devicetree (921600 8N1); we only keep the baud register
//! in sync across the temporary 480 MHz -> 400 MHz system clock switch that
//! happens around each capture.

const USART8_BASE: usize = 0x4000_2000;
const STATR: *mut u32 = USART8_BASE as *mut u32;
const DATAR: *mut u32 = (USART8_BASE + 0x04) as *mut u32;
const BRR: *mut u32 = (USART8_BASE + 0x08) as *mut u32;
const CTLR1: *mut u32 = (USART8_BASE + 0x0C) as *mut u32;

const STATR_RXNE: u32 = 1 << 5;
const STATR_TC: u32 = 1 << 6;
const STATR_TXE: u32 = 1 << 7;

const CTLR1_RE: u32 = 1 << 2;
const CTLR1_TE: u32 = 1 << 3;
const CTLR1_UE: u32 = 1 << 13;

/// Baud rate divisor programmed by the Zephyr UART driver at boot
/// (921600 baud with the 480 MHz system clock).
static mut BOOT_BRR: u32 = 0;

/// Latch the boot-time BRR and make sure the USART is enabled.
pub fn init() {
    unsafe {
        let brr = BRR.read_volatile();
        BOOT_BRR = brr;
        let mut ctlr1 = CTLR1.read_volatile();
        ctlr1 |= CTLR1_UE | CTLR1_TE | CTLR1_RE;
        CTLR1.write_volatile(ctlr1);
    }
}

/// Reprogram BRR for the 400 MHz capture clock.
///
/// The capture clock chain scales every bus clock by 400/480 = 5/6, and BRR
/// scales linearly with the input clock.
pub fn enter_capture_clock() {
    unsafe {
        let boot = BOOT_BRR;
        if boot != 0 {
            BRR.write_volatile((boot * 5 + 3) / 6);
        }
    }
}

/// Restore the boot-time BRR (480 MHz system clock).
pub fn leave_capture_clock() {
    unsafe {
        let boot = BOOT_BRR;
        if boot != 0 {
            BRR.write_volatile(boot);
        }
    }
}

/// Blocking transmit of one byte.
#[inline]
pub fn tx_byte(b: u8) {
    unsafe {
        while STATR.read_volatile() & STATR_TXE == 0 {}
        DATAR.write_volatile(b as u32);
    }
}

/// Blocking transmit of a slice.
pub fn write(buf: &[u8]) {
    for &b in buf {
        tx_byte(b);
    }
}

/// Wait for the last byte to leave the shift register.
pub fn flush() {
    unsafe {
        while STATR.read_volatile() & STATR_TC == 0 {}
    }
}

/// Non-blocking receive. Returns `Some(byte)` if one is pending.
#[inline]
pub fn rx_byte() -> Option<u8> {
    unsafe {
        if STATR.read_volatile() & STATR_RXNE != 0 {
            Some((DATAR.read_volatile() & 0xFF) as u8)
        } else {
            None
        }
    }
}

/// Blocking receive of one byte.
pub fn rx_byte_blocking() -> u8 {
    loop {
        if let Some(b) = rx_byte() {
            return b;
        }
    }
}
