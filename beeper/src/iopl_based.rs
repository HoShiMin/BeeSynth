use std::arch::asm;

use crate::sound_emitter::{BeeperDivisor, BeeperFrequency, SoundEmitter};

#[derive(Default)]
pub struct BeeperIopl;

impl BeeperIopl {
    #[cfg(target_arch = "x86_64")]
    const IOPL_BITMASK: u64 = 0x3000;

    #[cfg(target_arch = "x86")]
    const IOPL_BITMASK: u32 = 0x3000;

    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[inline]
    pub fn cli() {
        unsafe {
            asm!("cli", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline]
    pub fn sti() {
        unsafe {
            asm!("sti", options(nomem, nostack, preserves_flags));
        }
    }

    #[inline]
    #[must_use]
    #[cfg(target_arch = "x86_64")]
    pub fn is_iopl_raised() -> bool {
        unsafe {
            let mut eflags: u64;
            asm!(
                "pushfq",
                "pop {eflags}",
                eflags = out(reg) eflags,
                options(nomem, preserves_flags)
            );
            (eflags & Self::IOPL_BITMASK) != 0
        }
    }

    #[inline]
    #[must_use]
    #[cfg(target_arch = "x86")]
    pub fn is_iopl_raised() -> bool {
        unsafe {
            let mut eflags: u32;
            asm!(
                "pushfd",
                "pop {eflags}",
                eflags = out(reg) eflags,
                options(nomem, preserves_flags)
            );
            (eflags & Self::IOPL_BITMASK) != 0
        }
    }
}

impl SoundEmitter for BeeperIopl {
    #[inline]
    fn prepare(&mut self) -> bool {
        unsafe {
            asm!(
                "mov {tmp}, al",
                "mov al, 0xB6",
                "out 0x43, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                options(nomem, nostack, preserves_flags)
            );
        }

        true
    }

    #[inline]
    fn play(&mut self) {
        unsafe {
            asm!(
                "mov {tmp}, al",
                "in al, 0x61",
                "or al, 00000011b",
                "out 0x61, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    #[inline]
    fn mute(&mut self) {
        unsafe {
            asm!(
                "mov {tmp}, al",
                "in al, 0x61",
                "and al, 11111100b",
                "out 0x61, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    #[inline]
    fn set_divisor(&mut self, divisor: BeeperDivisor) {
        unsafe {
            #![allow(clippy::cast_possible_truncation)]
            asm!(
                "mov {tmp}, al",
                "mov al, {low}",
                "out 0x42, al",
                "mov al, {high}",
                "out 0x42, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                low = in(reg_byte) divisor.get() as u8,
                high = in(reg_byte) (divisor.get() >> 8) as u8,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    #[inline]
    fn set_frequency(&mut self, freq: BeeperFrequency) {
        self.set_divisor(freq.into());
    }

    #[inline]
    fn up(&mut self) {
        unsafe {
            asm!(
                "mov {tmp}, al",
                "in al, 0x61",
                "or al, 00000010b",
                "out 0x61, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    #[inline]
    fn down(&mut self) {
        unsafe {
            asm!(
                "mov {tmp}, al",
                "in al, 0x61",
                "and al, 11111101b",
                "out 0x61, al",
                "mov al, {tmp}",
                tmp = out(reg_byte) _,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}