#![allow(clippy::cast_possible_truncation)]

use std::f32::consts::PI;
use super::filter::{Type, Data, Filter};

#[inline]
#[must_use]
fn calc_rc(freq: f32) -> f32 {
    1_f32 / (2_f32 * PI * freq)
}

#[inline]
#[must_use]
#[allow(clippy::cast_precision_loss)]
fn calc_dt(sample_rate_hz: u32) -> f32 {
    1_f32 / (sample_rate_hz as f32)
}


pub struct HighPass {
    dt: f32,
    rc: f32
}

impl HighPass {
    #[must_use]
    pub fn new(sample_rate_hz: u32, freq_hz: f32) -> Self {
        Self { dt: calc_dt(sample_rate_hz), rc: calc_rc(freq_hz) }
    }

    pub fn apply(&self, samples: &mut [f32]) {
        let alpha = self.rc / (self.rc + self.dt);

        let mut prev_sample = samples[0];
        let mut prev_filtered = prev_sample;
    
        for sample in samples {
            let current_sample = *sample;
            let current_filtered = alpha * (prev_filtered + current_sample - prev_sample);
            
            *sample = current_filtered;
            
            prev_sample = current_sample;
            prev_filtered = current_filtered;
        }
    }
}

impl Filter for HighPass {
    #[must_use]
    fn filter_type(&self) -> super::filter::Type {
        Type::Amplitude
    }

    #[must_use]
    fn filter(&self, data: Data) -> Option<Data> {
        match data {
            Data::Amplitude(mut data) => {
                self.apply(&mut data.samples);
                Some(Data::Amplitude(data))
            },
            _ => None
        }
    }
}



pub struct LowPass {
    dt: f32,
    rc: f32
}

impl LowPass {
    #[must_use]
    pub fn new(sample_rate_hz: u32, freq_hz: f32) -> Self {
        Self { dt: calc_dt(sample_rate_hz), rc: calc_rc(freq_hz) }
    }

    pub fn apply(&self, samples: &mut [f32]) {
        let alpha = self.dt / (self.rc + self.dt);

        let mut prev_filtered: f32 = alpha * samples[0];
    
        for sample in samples {
            let current_sample = *sample;
            let current_filtered = alpha * current_sample + (1.0_f32 - alpha) * prev_filtered;
    
            *sample = current_filtered;
    
            prev_filtered = current_filtered;
        }
    }
}

impl Filter for LowPass {
    #[must_use]
    fn filter_type(&self) -> super::filter::Type {
        Type::Amplitude
    }

    #[must_use]
    fn filter(&self, data: Data) -> Option<Data> {
        match data {
            Data::Amplitude(mut data) => {
                self.apply(&mut data.samples);
                Some(Data::Amplitude(data))
            },
            _ => None
        }
    }
}
