///
/// Base frequency of the system clock generator is 1'193'182 Hz.
/// We can set a divisor of the clock in the range of [1..65535] as it has 16-bit width.
///
/// So we have the equation:
///   Frequency(Hz) = Base(Hz) / Divisor
///   or
///   Divisor = Base(Hz) / Frequency(Hz)
///
/// Where:
///   Base(Hz) = 1'193'182 Hz
///   Divisor ∈ [1..65535], zero value is inapplicable as we can't divide by zero
///   
/// As follows:
///   Fmin = 1'193'182 / 65535 ≈ 18.206 Hz
///   Fmax = 1'193'182 / 1 = 1'193'182 Hz
///


pub struct ClockGenerator;

impl ClockGenerator {
    const BASE_FREQ: u32 = 1_193_182; // In Hz
}

type Hertz = u32;

#[derive(Copy, Clone)]
pub struct BeeperFrequency(Hertz);

impl BeeperFrequency {
    pub const MIN: Self = Self(ClockGenerator::BASE_FREQ / (u16::MAX as u32) + 1); // 19 Hz (18.206 Hz rounded up)
    pub const MAX: Self = Self(ClockGenerator::BASE_FREQ);

    /// Clamps the given frequency into the bounds [Fmin, Fmax].
    #[inline]
    #[must_use]
    pub fn new_clamped(freq_hz: Hertz) -> Self {
        BeeperFrequency(freq_hz.clamp(Self::MIN.0, Self::MAX.0))
    }

    #[inline]
    #[must_use]
    pub fn new(freq_hz: Hertz) -> Option<Self> {
        if (Self::MIN.0..=Self::MAX.0).contains(&freq_hz) {
            Some(Self(freq_hz))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn get(&self) -> Hertz {
        self.0
    }
}

impl From<u32> for BeeperFrequency {
    fn from(freq_hz: u32) -> Self {
        Self::new_clamped(freq_hz)
    }
}

impl From<u16> for BeeperFrequency {
    fn from(freq_hz: u16) -> Self {
        Self::new_clamped(freq_hz.into())
    }
}

impl From<u8> for BeeperFrequency {
    fn from(freq_hz: u8) -> Self {
        Self::new_clamped(freq_hz.into())
    }
}

impl From<f32> for BeeperFrequency {
    fn from(freq_hz: f32) -> Self {
        #![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        
        let freq = if freq_hz < 0.0_f32 {
            Self::MIN.0
        } else if freq_hz > (Self::MAX.0 as f32) { 
            Self::MAX.0
        } else {
            freq_hz.round() as Hertz
        };

        Self::new_clamped(freq)
    }
}

impl From<f64> for BeeperFrequency {
    fn from(freq_hz: f64) -> Self {
        #![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        
        let freq = if freq_hz < 0.0_f64 {
            Self::MIN.0
        } else if freq_hz > f64::from(Self::MAX.0) { 
            Self::MAX.0
        } else {
            freq_hz.round() as Hertz
        };

        Self::new_clamped(freq)
    }
}



#[derive(Copy, Clone)]
pub struct BeeperDivisor(u16);

impl BeeperDivisor {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(u16::MAX);
     
    #[must_use]
    pub fn new(divisor: u16) -> Self {
        if divisor == 0 {
            Self(Self::MIN.0)
        } else {
            Self(divisor)
        }
    }

    #[must_use]
    pub fn from_freq(freq_hz: Hertz) -> Self {
        let freq = BeeperFrequency::new_clamped(freq_hz);
        
        #[allow(clippy::cast_possible_truncation)]
        Self((ClockGenerator::BASE_FREQ / freq.0) as u16)
    }

    #[must_use]
    pub fn get(&self) -> u16 {
        self.0
    }
}

impl From<BeeperFrequency> for BeeperDivisor {
    fn from(freq: BeeperFrequency) -> Self {
        Self::from_freq(freq.0)
    }
}



pub trait SoundEmitter {
    fn prepare(&mut self) -> bool; // Usually sets a beeper regime
    
    fn play(&mut self);
    fn mute(&mut self);
    
    fn set_divisor(&mut self, divisor: BeeperDivisor);
    fn set_frequency(&mut self, freq: BeeperFrequency);
    
    fn up(&mut self);
    fn down(&mut self);
}