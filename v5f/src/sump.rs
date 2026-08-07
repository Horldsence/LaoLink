//! SUMP (Open Bench Logic Sniffer) protocol server.
//!
//! Speaks the subset of SUMP that sigrok's `openbench-logic-sniffer` driver
//! uses, so PulseView/sigrok-cli can drive the analyzer directly. Short
//! commands: 0x00 reset, 0x01 run, 0x02 ID ("1ALS"), 0x04 metadata. Long
//! commands: 0x80 divider, 0x81 read/delay count, 0x82 flags, and
//! 0xC0/0xC1/0xC2 (+stage*4) trigger mask/value/config.
//!
//! All multi-byte fields are big-endian. The SUMP reference clock is a fixed
//! 100 MHz: sample rate = 100 MHz / (divider + 1). The capture engine samples
//! at exactly 100 MHz and decimates by (divider + 1), so reported timing is
//! exact.

use crate::capture::{self, CaptureParams, CaptureResult};
use crate::uart;

/// SUMP short/long command opcodes.
mod cmd {
    pub const RESET: u8 = 0x00;
    pub const RUN: u8 = 0x01;
    pub const ID: u8 = 0x02;
    pub const METADATA: u8 = 0x04;
    pub const FINISH_NOW: u8 = 0x05;
    pub const SET_DIVIDER: u8 = 0x80;
    pub const SET_COUNT: u8 = 0x81;
    pub const SET_FLAGS: u8 = 0x82;
}

/// Maximum sample rate we advertise (SUMP reference clock).
const MAX_SAMPLE_RATE_HZ: u32 = 100_000_000;

/// SUMP server state.
pub struct Sump {
    params: CaptureParams,
}

impl Sump {
    pub const fn new() -> Self {
        Sump {
            params: CaptureParams {
                decimation: 499, // 100 MHz / 500 = 200 kHz, the sigrok default
                read_count: 1024,
                delay_count: 0,
                trig_mask: 0,
                trig_value: 0,
            },
        }
    }

    fn reset(&mut self) {
        self.params.trig_mask = 0;
        self.params.trig_value = 0;
    }

    fn read_u32_be(&self) -> u32 {
        let b0 = uart::rx_byte_blocking() as u32;
        let b1 = uart::rx_byte_blocking() as u32;
        let b2 = uart::rx_byte_blocking() as u32;
        let b3 = uart::rx_byte_blocking() as u32;
        (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
    }

    fn send_metadata(&self) {
        uart::write(&[0x01]); // device name
        uart::write(b"CH32H417-LA\0");
        uart::write(&[0x20]); // number of probes, u32 BE
        uart::write(&8u32.to_be_bytes());
        uart::write(&[0x21]); // sample memory in bytes, u32 BE
        uart::write(&(capture::CAP_SAMPLES as u32).to_be_bytes());
        uart::write(&[0x22]); // dynamic memory in bytes, u32 BE
        uart::write(&0u32.to_be_bytes());
        uart::write(&[0x23]); // max sample rate in Hz, u32 BE
        uart::write(&MAX_SAMPLE_RATE_HZ.to_be_bytes());
        uart::write(&[0x24]); // protocol version, u32 BE
        uart::write(&2u32.to_be_bytes());
        uart::write(&[0x00]); // end
        uart::flush();
    }

    /// Poll one command byte; returns Some(result) when a capture ran.
    pub fn poll(&mut self) -> Option<CaptureResult> {
        let cmd = uart::rx_byte()?;
        match cmd {
            cmd::RESET => self.reset(),
            cmd::RUN => {
                // Clamp to what the sample memory can hold.
                if self.params.read_count > capture::CAP_SAMPLES as u32 {
                    self.params.read_count = capture::CAP_SAMPLES as u32;
                }
                self.params.read_count &= !3;
                if self.params.read_count < 4 {
                    self.params.read_count = 4;
                }
                if self.params.delay_count > self.params.read_count {
                    self.params.delay_count = self.params.read_count;
                }
                self.params.delay_count &= !3;
                if self.params.decimation == 0 {
                    self.params.decimation = 1;
                }
                return Some(capture::capture_and_upload(&self.params));
            }
            cmd::ID => {
                uart::write(b"1ALS");
                uart::flush();
            }
            cmd::METADATA => self.send_metadata(),
            cmd::FINISH_NOW => {} // only meaningful during capture; handled there
            cmd::SET_DIVIDER => {
                let div = self.read_u32_be();
                self.params.decimation = div as u64 + 1;
            }
            cmd::SET_COUNT => {
                let v = self.read_u32_be();
                self.params.read_count = ((v >> 16) & 0xFFFF) * 4;
                self.params.delay_count = (v & 0xFFFF) * 4;
            }
            cmd::SET_FLAGS => {
                let _ = self.read_u32_be(); // demux/RLE/groups unsupported
            }
            c if c & 0xF0 == 0xC0 => {
                // Basic trigger long commands, stage = (cmd >> 2) & 3.
                let v = self.read_u32_be();
                let stage = (c >> 2) & 3;
                if stage == 0 {
                    match c & 0x03 {
                        0 => self.params.trig_mask = v as u8,
                        1 => self.params.trig_value = v as u8,
                        _ => {} // config: delay/level/start — not needed
                    }
                }
            }
            _ => {}
        }
        None
    }
}
