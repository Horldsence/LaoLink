//! Capture engine: 400 MHz clock switch, UHSIF polling, software decimation,
//! edge-trigger scan and SUMP sample upload.
//!
//! Timing model
//! ------------
//! During capture the system PLL is switched to 400 MHz (HSE 25 MHz x16, the
//! same sequence WCH's own logic-analyzer firmware uses) and UHSIF samples
//! 8 bits at PCLK = SYSCLK / 4 = 100 MHz into a ring of 8 x 16 KiB slots.
//! Software keeps every k-th raw sample (k = SUMP divider + 1), so the
//! effective sample rate is exactly 100 MHz / k, matching the fixed 100 MHz
//! reference clock of sigrok's SUMP driver.

#![allow(static_mut_refs)]

use crate::uart;
use crate::uhsif;

/// Raw capture ring written by UHSIF (8 slots x 16 KiB).
const RAW_SIZE: usize = uhsif::SLOT_SIZE * uhsif::SLOT_COUNT;

/// Decimated sample ring capacity, in samples (bytes). Advertised to the
/// host as the sample memory size.
pub const CAP_SAMPLES: usize = 96 * 1024;

#[repr(align(32))]
struct RawBuf([u8; RAW_SIZE]);
#[repr(align(32))]
struct CapBuf([u8; CAP_SAMPLES]);

static mut RAW_BUF: RawBuf = RawBuf([0; RAW_SIZE]);
static mut CAP_BUF: CapBuf = CapBuf([0; CAP_SAMPLES]);

/// Line 0 "buffer completed" flag in the UHSIF interrupt-flag register.
const UHSIF_INTFR: *const u32 = 0x4003_8014 as *const u32;
const INTFR_LINE0: u32 = 1 << 11;
/// Line 0 socket status word (mirrors the library IRQ handler).
const UHSIF_LINE0_SOCKET: *mut u32 = 0x4003_9010 as *mut u32;

/// ~1 s worth of poll iterations without slot progress means the hardware
/// stalled; bail out instead of hanging forever.
const STALL_LIMIT: u32 = 50_000_000;

/// Capture parameters derived from SUMP long commands.
pub struct CaptureParams {
    /// Decimation factor k: keep every k-th 100 MHz raw sample.
    pub decimation: u64,
    /// Total samples to upload (multiple of 4, <= CAP_SAMPLES).
    pub read_count: u32,
    /// Samples to capture after the trigger (multiple of 4).
    pub delay_count: u32,
    /// Stage-0 trigger mask (channels 0..7). 0 = trigger disabled.
    pub trig_mask: u8,
    /// Stage-0 trigger value (channels 0..7).
    pub trig_value: u8,
}

pub enum CaptureResult {
    /// Capture finished; samples were uploaded.
    Done,
    /// Host aborted (SUMP reset) during capture.
    Aborted,
    /// Hardware stopped making progress.
    Stalled,
}

/* ------------------------------------------------------------------------- */
/* System clock switch: 480 MHz (Zephyr) <-> 400 MHz (capture)                */
/* ------------------------------------------------------------------------- */

const RCC_CTLR: *mut u32 = 0x4002_1000 as *mut u32;
const RCC_CFGR0: *mut u32 = 0x4002_1004 as *mut u32;
const RCC_PLLCFGR: *mut u32 = 0x4002_1008 as *mut u32;

const RCC_PLLON: u32 = 0x0100_0000;
const RCC_PLLRDY: u32 = 0x0200_0000;
const RCC_SW: u32 = 0x3;
const RCC_SW_PLL: u32 = 0x2;
const RCC_SWS: u32 = 0xC;
const RCC_SWS_PLL: u32 = 0x8;
const RCC_PLLMUL: u32 = 0x1F;
const RCC_PLLMUL16: u32 = 0x10;
const RCC_PLLSRC_HSE: u32 = 0x20;
const RCC_PLL_SRC_DIV: u32 = 0x1F00;
const RCC_SYSPLL_SEL: u32 = 0x7000_0000;
const RCC_SYSPLL_GATE: u32 = 0x8000_0000;

struct SavedClock {
    ctlr: u32,
    cfgr0: u32,
    pllcfgr: u32,
}

/// Switch the system clock to the 400 MHz capture PLL configuration.
/// Follows `UHSIF_Clock_Set(RCC_PLLMUL16)` from WCH's reference firmware.
unsafe fn enter_capture_clock() -> SavedClock {
    let saved = SavedClock {
        ctlr: RCC_CTLR.read_volatile(),
        cfgr0: RCC_CFGR0.read_volatile(),
        pllcfgr: RCC_PLLCFGR.read_volatile(),
    };

    // Switch to HSI while the PLL is reconfigured.
    let mut cfgr0 = RCC_CFGR0.read_volatile() & !RCC_SW;
    RCC_CFGR0.write_volatile(cfgr0);
    while RCC_CFGR0.read_volatile() & RCC_SWS != 0 {}

    RCC_PLLCFGR.write_volatile(RCC_PLLCFGR.read_volatile() & !RCC_SYSPLL_GATE);
    RCC_CTLR.write_volatile(RCC_CTLR.read_volatile() & !RCC_PLLON);

    // Sys PLL = HSE 25 MHz x16 = 400 MHz, source divider 1, and select the
    // sys PLL (not the USBHS PLL that Zephyr's 480 MHz setup uses).
    let mut pllcfgr = RCC_PLLCFGR.read_volatile();
    pllcfgr &= !(RCC_PLLMUL | RCC_PLLSRC_HSE | RCC_PLL_SRC_DIV | RCC_SYSPLL_SEL);
    pllcfgr |= RCC_PLLMUL16 | RCC_PLLSRC_HSE;
    RCC_PLLCFGR.write_volatile(pllcfgr);

    RCC_CTLR.write_volatile(RCC_CTLR.read_volatile() | RCC_PLLON);
    while RCC_CTLR.read_volatile() & RCC_PLLRDY == 0 {}
    RCC_PLLCFGR.write_volatile(RCC_PLLCFGR.read_volatile() | RCC_SYSPLL_GATE);

    cfgr0 = (RCC_CFGR0.read_volatile() & !RCC_SW) | RCC_SW_PLL;
    RCC_CFGR0.write_volatile(cfgr0);
    while RCC_CFGR0.read_volatile() & RCC_SWS != RCC_SWS_PLL {}

    saved
}

/// Restore the clock configuration saved by `enter_capture_clock`.
unsafe fn leave_capture_clock(saved: &SavedClock) {
    // Back to HSI first.
    RCC_CFGR0.write_volatile(RCC_CFGR0.read_volatile() & !RCC_SW);
    while RCC_CFGR0.read_volatile() & RCC_SWS != 0 {}

    // The USBHS PLL used by the 480 MHz configuration kept running; restore
    // the original selection/gating and switch back to PLL.
    RCC_PLLCFGR.write_volatile(saved.pllcfgr);
    RCC_CTLR.write_volatile(saved.ctlr);
    RCC_CFGR0.write_volatile(saved.cfgr0);
    while RCC_CFGR0.read_volatile() & RCC_SWS != (saved.cfgr0 & RCC_SWS) {}
}

/* ------------------------------------------------------------------------- */
/* Capture state                                                              */
/* ------------------------------------------------------------------------- */

struct Engine {
    k: u64,
    read_count: u32,
    delay_count: u32,
    mask: u8,
    value: u8,
    trig_armed: bool,

    /// Distance (in raw samples) from the current position to the next kept
    /// sample; counts down modulo k.
    rem: u64,
    /// Write position in the CAP ring.
    head: usize,
    /// Total kept samples since arm.
    out_total: u64,
    /// Kept-sample index of the trigger edge.
    trig_at: Option<u64>,
    /// Whether the previously kept sample matched the trigger pattern.
    prev_match: bool,
    /// Set by a host "finish now" command: forces a trigger at the next sample.
    force_trig: bool,
}

impl Engine {
    fn new(p: &CaptureParams) -> Self {
        Engine {
            k: p.decimation.max(1),
            read_count: p.read_count,
            delay_count: p.delay_count,
            mask: p.trig_mask,
            value: p.trig_value & p.trig_mask,
            trig_armed: p.trig_mask != 0,
            rem: 0,
            head: 0,
            out_total: 0,
            trig_at: None,
            prev_match: false,
            force_trig: false,
        }
    }

    #[inline]
    fn done(&self) -> bool {
        if self.trig_armed {
            match self.trig_at {
                Some(t) => self.out_total - t >= self.delay_count as u64,
                None => false,
            }
        } else {
            self.out_total >= self.read_count as u64
        }
    }

    /// Store one kept sample, updating trigger state. Returns capture-done.
    #[inline]
    fn keep(&mut self, b: u8, cap: &mut CapBuf) -> bool {
        if self.trig_armed && self.trig_at.is_none() {
            let m = b & self.mask == self.value;
            if (m && !self.prev_match) || self.force_trig {
                self.trig_at = Some(self.out_total);
            }
            self.prev_match = m;
        }
        cap.0[self.head] = b;
        self.head += 1;
        if self.head == CAP_SAMPLES {
            self.head = 0;
        }
        self.out_total += 1;
        self.done()
    }

    /// Process one full raw slot. Returns true when the capture is complete.
    fn process_slot(&mut self, raw: &[u8], cap: &mut CapBuf) -> bool {
        if self.k == 1 {
            self.process_slot_full_rate(raw, cap)
        } else {
            self.process_slot_decimate(raw, cap)
        }
    }

    /// k == 1: every raw byte is a sample. Word-wise copy with a bit-trick
    /// trigger scan (zero-byte detect) when a trigger is armed.
    fn process_slot_full_rate(&mut self, raw: &[u8], cap: &mut CapBuf) -> bool {
        if self.trig_armed && self.trig_at.is_none() {
            let bc_m = (self.mask as u32) * 0x0101_0101;
            let bc_v = (self.value as u32) * 0x0101_0101;
            for chunk in raw.chunks_exact(4) {
                let w = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                // Lane top bit set where (byte & mask) == value.
                let x = (w ^ bc_v) & bc_m;
                let z = x.wrapping_sub(0x0101_0101) & !x & 0x8080_8080;
                let prev = (z << 8) | if self.prev_match { 0x80 } else { 0 };
                let edges = z & !prev;
                if edges != 0 || self.force_trig {
                    let lane = if self.force_trig {
                        0
                    } else {
                        (edges.trailing_zeros() / 8) as u64
                    };
                    self.trig_at = Some(self.out_total + lane);
                    self.force_trig = false;
                }
                self.prev_match = z & 0x8000_0000 != 0;

                // Store whole word (head stays word-aligned on this path).
                let h = self.head;
                cap.0[h..h + 4].copy_from_slice(&w.to_le_bytes());
                self.head = (h + 4) % CAP_SAMPLES;
                self.out_total += 4;
                if self.done() {
                    return true;
                }
            }
        } else {
            // No (pending) trigger: bulk copy.
            for chunk in raw.chunks_exact(4) {
                let h = self.head;
                cap.0[h..h + 4].copy_from_slice(chunk);
                self.head = (h + 4) % CAP_SAMPLES;
                self.out_total += 4;
                if self.done() {
                    return true;
                }
            }
        }
        false
    }

    /// k > 1: keep one byte every k raw bytes.
    fn process_slot_decimate(&mut self, raw: &[u8], cap: &mut CapBuf) -> bool {
        let k = self.k;
        let mut rem = self.rem;
        for chunk in raw.chunks_exact(4) {
            let w = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let mut r = rem;
            while r < 4 {
                let b = (w >> (r * 8)) as u8;
                if self.keep(b, cap) {
                    self.rem = r + k - 4;
                    return true;
                }
                r += k;
            }
            rem = r - 4;
        }
        self.rem = rem;
        false
    }
}

/* ------------------------------------------------------------------------- */
/* Top level                                                                  */
/* ------------------------------------------------------------------------- */

#[inline]
fn fence() {
    unsafe { core::arch::asm!("fence", options(nomem, nostack)) }
}

/// Run one acquisition and upload the samples over UART (SUMP data phase).
pub fn capture_and_upload(params: &CaptureParams) -> CaptureResult {
    let raw = unsafe { &mut *core::ptr::addr_of_mut!(RAW_BUF) };
    let cap = unsafe { &mut *core::ptr::addr_of_mut!(CAP_BUF) };

    // Switch to the 400 MHz capture clock, then re-time the UART.
    let saved = unsafe { enter_capture_clock() };
    uart::enter_capture_clock();

    let result = if uhsif::init_capture(raw.0.as_mut_ptr(), uhsif::UHSIF_DIV4).is_err() {
        CaptureResult::Stalled
    } else {
        run_engine(params, raw, cap)
    };

    uhsif::stop();

    // Samples are uploaded inside run_engine() while the capture clock is
    // still active (the UART baud divisor was adjusted accordingly, so the
    // host always sees the nominal baud rate). Restore the 480 MHz clock.
    unsafe { leave_capture_clock(&saved) };
    uart::leave_capture_clock();

    result
}

fn run_engine(params: &CaptureParams, raw: &mut RawBuf, cap: &mut CapBuf) -> CaptureResult {
    let mut eng = Engine::new(params);
    let mut next_slot: usize = 0;
    let mut stall: u32 = 0;

    fence();
    uhsif::start();

    let outcome = loop {
        // Host abort (SUMP reset) / soft trigger (Demon-core finish-now).
        if let Some(b) = uart::rx_byte() {
            match b {
                0x00 => break CaptureResult::Aborted,
                0x05 => eng.force_trig = true,
                _ => {}
            }
        }

        if unsafe { UHSIF_INTFR.read_volatile() } & INTFR_LINE0 != 0 {
            let slot = &raw.0[next_slot * uhsif::SLOT_SIZE..(next_slot + 1) * uhsif::SLOT_SIZE];
            fence();
            let done = eng.process_slot(slot, cap);
            fence();
            uhsif::rearm_slot(next_slot);
            // Ack the event flag the same way the library IRQ handler does:
            // write back whichever status bit the hardware raised.
            unsafe {
                let s = UHSIF_LINE0_SOCKET.read_volatile();
                let ack = s & 0x1C1;
                if ack != 0 {
                    UHSIF_LINE0_SOCKET.write_volatile(s | ack);
                }
            }
            next_slot = (next_slot + 1) % uhsif::SLOT_COUNT;
            stall = 0;
            if done {
                break CaptureResult::Done;
            }
        } else {
            stall += 1;
            if stall >= STALL_LIMIT {
                break CaptureResult::Stalled;
            }
        }
    };

    uhsif::stop();

    if matches!(outcome, CaptureResult::Done) {
        upload_window(&eng, cap);
    }
    outcome
}

/// Send exactly `read_count` samples, oldest first, ending at the last
/// captured sample. If the trigger fired before enough pre-trigger history
/// existed, the front is padded with the oldest captured value so the host
/// always receives the agreed sample count.
fn upload_window(eng: &Engine, cap: &CapBuf) {
    let end = eng.out_total; // one past the last kept sample
    let read = eng.read_count as u64;
    let start = end.saturating_sub(read);
    let oldest = end.saturating_sub(CAP_SAMPLES as u64);
    let start = start.max(oldest);

    let pad = (read - (end - start)) as usize;
    if pad > 0 && end > start {
        let b = cap.0[(start % CAP_SAMPLES as u64) as usize];
        for _ in 0..pad {
            uart::tx_byte(b);
        }
    }

    // Chunked linear sends out of the ring.
    let mut i = start;
    while i < end {
        let idx = (i % CAP_SAMPLES as u64) as usize;
        let n = (CAP_SAMPLES - idx).min((end - i) as usize);
        uart::write(&cap.0[idx..idx + n]);
        i += n as u64;
    }
    uart::flush();
}
