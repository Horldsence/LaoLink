//! Dependency shims for WCH's precompiled `libUHSIF.a`.
//!
//! The library was built against WCH's standard peripheral library (SPL) and
//! GCC millicode; the symbols it imports are implemented here so the Rust
//! application can link it directly:
//!
//! - `GPIO_Init`               — faithful port of SPL `ch32h417_gpio.c`
//! - `RCC_HBPeriphClockCmd`    — RCC->HBPCENR bit set/clear
//! - `RCC_HB2PeriphClockCmd`   — RCC->HB2PCENR bit set/clear
//! - `RCC_HBPeriphResetCmd`    — RCC->HBRSTR bit set/clear
//! - `__riscv_save/restore_*`  — RISC-V millicode (`-msave-restore` ABI)
//! - `UHSIF_Line*_*_Callback`  — unused (we poll instead of using interrupts)

#![allow(non_snake_case)]

use core::arch::global_asm;

/// RCC peripheral base (HBPERIPH_BASE + 0x21000).
const RCC_BASE: usize = 0x4002_1000;
const RCC_HBPCENR: *mut u32 = (RCC_BASE + 0x18) as *mut u32;
const RCC_HB2PCENR: *mut u32 = (RCC_BASE + 0x1C) as *mut u32;
const RCC_HBRSTR: *mut u32 = (RCC_BASE + 0x2C) as *mut u32;

/* ------------------------------------------------------------------------- */
/* RCC clock/reset shims                                                      */
/* ------------------------------------------------------------------------- */

unsafe fn reg_bits_set_clear(reg: *mut u32, bits: u32, enable: bool) {
    let mut v = unsafe { reg.read_volatile() };
    if enable {
        v |= bits;
    } else {
        v &= !bits;
    }
    unsafe { reg.write_volatile(v) };
}

/// void RCC_HBPeriphClockCmd(uint32_t RCC_HBPeriph, FunctionalState NewState)
#[no_mangle]
pub unsafe extern "C" fn RCC_HBPeriphClockCmd(periph: u32, state: i32) {
    reg_bits_set_clear(RCC_HBPCENR, periph, state != 0);
}

/// void RCC_HB2PeriphClockCmd(uint32_t RCC_HB2Periph, FunctionalState NewState)
#[no_mangle]
pub unsafe extern "C" fn RCC_HB2PeriphClockCmd(periph: u32, state: i32) {
    reg_bits_set_clear(RCC_HB2PCENR, periph, state != 0);
}

/// void RCC_HBPeriphResetCmd(uint32_t RCC_HBPeriph, FunctionalState NewState)
#[no_mangle]
pub unsafe extern "C" fn RCC_HBPeriphResetCmd(periph: u32, state: i32) {
    reg_bits_set_clear(RCC_HBRSTR, periph, state != 0);
}

/* ------------------------------------------------------------------------- */
/* GPIO_Init (SPL port)                                                       */
/* ------------------------------------------------------------------------- */

/// WCH GPIO register block (matches `GPIO_TypeDef` in ch32h41xhw.h).
#[repr(C)]
pub struct GpioRegs {
    cfglr: u32,
    cfghr: u32,
    indr: u32,
    outdr: u32,
    bshr: u32,
    bcr: u32,
    lckr: u32,
    speed: u32,
}

/// SPL `GPIO_InitTypeDef` layout: u16 pin, (pad), u32 speed, u32 mode.
#[repr(C)]
pub struct GpioInitTypeDef {
    pub pin: u16,
    _pad: u16,
    pub speed: u32,
    pub mode: u32,
}

const GPIO_MODE_IPD: u32 = 0x28;
const GPIO_MODE_IPU: u32 = 0x48;

/// Faithful port of SPL `GPIO_Init` from ch32h417_gpio.c.
#[no_mangle]
pub unsafe extern "C" fn GPIO_Init(gpio: *mut GpioRegs, init: *const GpioInitTypeDef) {
    let init = unsafe { &*init };
    let pin = init.pin as u32;
    let mode = init.mode;
    let mut currentmode = mode & 0x0F;

    // Field accessors: read_volatile/write_volatile live on raw pointers.
    macro_rules! reg {
        ($field:ident) => {
            core::ptr::addr_of_mut!((*gpio).$field)
        };
    }

    if mode & 0x10 != 0 {
        // Output mode: program the per-pin 2-bit SPEED field first.
        currentmode |= 0x01;
        let mut tmpreg = unsafe { reg!(speed).read_volatile() };
        for pinpos in 0..16u32 {
            if pin & (1 << pinpos) != 0 {
                let pos = pinpos * 2;
                tmpreg = (tmpreg & !(0x03 << pos)) | ((init.speed & 0x03) << pos);
            }
        }
        unsafe { reg!(speed).write_volatile(tmpreg) };
    }

    if pin & 0x00FF != 0 {
        let mut tmpreg = unsafe { reg!(cfglr).read_volatile() };
        for pinpos in 0..8u32 {
            if pin & (1 << pinpos) != 0 {
                let pos = pinpos * 4;
                tmpreg = (tmpreg & !(0x0F << pos)) | (currentmode << pos);
                if mode == GPIO_MODE_IPD {
                    unsafe { reg!(bcr).write_volatile(1 << pinpos) };
                } else if mode == GPIO_MODE_IPU {
                    unsafe { reg!(bshr).write_volatile(1 << pinpos) };
                }
            }
        }
        unsafe { reg!(cfglr).write_volatile(tmpreg) };
    }

    if pin > 0x00FF {
        let mut tmpreg = unsafe { reg!(cfghr).read_volatile() };
        for pinpos in 8..16u32 {
            if pin & (1 << pinpos) != 0 {
                let pos = (pinpos - 8) * 4;
                tmpreg = (tmpreg & !(0x0F << pos)) | (currentmode << pos);
                if mode == GPIO_MODE_IPD {
                    unsafe { reg!(bcr).write_volatile(1 << pinpos) };
                } else if mode == GPIO_MODE_IPU {
                    unsafe { reg!(bshr).write_volatile(1 << pinpos) };
                }
            }
        }
        unsafe { reg!(cfghr).write_volatile(tmpreg) };
    }
}

/* ------------------------------------------------------------------------- */
/* UHSIF line callbacks — unused; capture is driven by polling.               */
/* ------------------------------------------------------------------------- */

macro_rules! unused_callback {
    ($name:ident, ($($arg:ident : $ty:ty),*)) => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $ty),*) {
            let _ = ($($arg),*);
        }
    };
}

unused_callback!(UHSIF_Line0_IN_Callback, (count: u32));
unused_callback!(UHSIF_Line1_IN_Callback, (count: u32));
unused_callback!(UHSIF_Line2_IN_Callback, (count: u32));
unused_callback!(UHSIF_Line3_IN_Callback, (count: u32));
unused_callback!(UHSIF_Line0_OUT_Callback, ());
unused_callback!(UHSIF_Line1_OUT_Callback, ());
unused_callback!(UHSIF_Line2_OUT_Callback, ());
unused_callback!(UHSIF_Line3_OUT_Callback, ());

/* ------------------------------------------------------------------------- */
/* RISC-V millicode (-msave-restore ABI)                                      */
/*                                                                            */
/* Convention: invoked with `jalr t0`; saves ra + s0..s(N-1), returns with    */
/* `jr t0`. Restore is tail-jumped via `jr t1` and ends in `ret`.             */
/* ------------------------------------------------------------------------- */

global_asm!(
    ".section .text.__riscv_save_1,\"ax\",@progbits",
    ".globl __riscv_save_1",
    ".type __riscv_save_1,@function",
    "__riscv_save_1:",
    "addi sp, sp, -16",
    "sw s0, 0(sp)",
    "sw ra, 4(sp)",
    "jr t0",
    ".size __riscv_save_1, .-__riscv_save_1",
    ".section .text.__riscv_restore_1,\"ax\",@progbits",
    ".globl __riscv_restore_1",
    ".type __riscv_restore_1,@function",
    "__riscv_restore_1:",
    "lw s0, 0(sp)",
    "lw ra, 4(sp)",
    "addi sp, sp, 16",
    "ret",
    ".size __riscv_restore_1, .-__riscv_restore_1",
    ".section .text.__riscv_save_8,\"ax\",@progbits",
    ".globl __riscv_save_8",
    ".type __riscv_save_8,@function",
    "__riscv_save_8:",
    "addi sp, sp, -48",
    "sw s0, 0(sp)",
    "sw s1, 4(sp)",
    "sw s2, 8(sp)",
    "sw s3, 12(sp)",
    "sw s4, 16(sp)",
    "sw s5, 20(sp)",
    "sw s6, 24(sp)",
    "sw s7, 28(sp)",
    "sw ra, 32(sp)",
    "jr t0",
    ".size __riscv_save_8, .-__riscv_save_8",
    ".section .text.__riscv_restore_8,\"ax\",@progbits",
    ".globl __riscv_restore_8",
    ".type __riscv_restore_8,@function",
    "__riscv_restore_8:",
    "lw s0, 0(sp)",
    "lw s1, 4(sp)",
    "lw s2, 8(sp)",
    "lw s3, 12(sp)",
    "lw s4, 16(sp)",
    "lw s5, 20(sp)",
    "lw s6, 24(sp)",
    "lw s7, 28(sp)",
    "lw ra, 32(sp)",
    "addi sp, sp, 48",
    "ret",
    ".size __riscv_restore_8, .-__riscv_restore_8",
    ".section .text.__riscv_save_10,\"ax\",@progbits",
    ".globl __riscv_save_10",
    ".type __riscv_save_10,@function",
    "__riscv_save_10:",
    "addi sp, sp, -48",
    "sw s0, 0(sp)",
    "sw s1, 4(sp)",
    "sw s2, 8(sp)",
    "sw s3, 12(sp)",
    "sw s4, 16(sp)",
    "sw s5, 20(sp)",
    "sw s6, 24(sp)",
    "sw s7, 28(sp)",
    "sw s8, 32(sp)",
    "sw s9, 36(sp)",
    "sw ra, 40(sp)",
    "jr t0",
    ".size __riscv_save_10, .-__riscv_save_10",
    ".section .text.__riscv_restore_10,\"ax\",@progbits",
    ".globl __riscv_restore_10",
    ".type __riscv_restore_10,@function",
    "__riscv_restore_10:",
    "lw s0, 0(sp)",
    "lw s1, 4(sp)",
    "lw s2, 8(sp)",
    "lw s3, 12(sp)",
    "lw s4, 16(sp)",
    "lw s5, 20(sp)",
    "lw s6, 24(sp)",
    "lw s7, 28(sp)",
    "lw s8, 32(sp)",
    "lw s9, 36(sp)",
    "lw ra, 40(sp)",
    "addi sp, sp, 48",
    "ret",
    ".size __riscv_restore_10, .-__riscv_restore_10",
);
