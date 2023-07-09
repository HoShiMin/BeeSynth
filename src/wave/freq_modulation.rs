///                            𝘵
/// 𝒇(𝘵) = 𝘈𝘮𝘱 * 𝘤𝘰𝘴(𝟸𝜋𝛺𝘵 + 𝟸𝜋𝜔⎰𝘈(𝜏)ⅆ𝜏)
///                           𝟶
/// Where:
///   𝒇(𝘵) - frequency-modulated signal.
///   𝘵 - time.
///   𝘈𝘮𝘱 - amplitude of the desired signal (1.0 in our case).
///   𝛺 - frequency of the carrier signal.
///   𝜔 - available deviation from the carrier frequency.
///   𝘈(𝜏) - original amplitude-modulated signal.
/// 

use std::f32::consts::TAU;
use super::filter::{Type, Data, Filter, WaveData};

pub struct FreqModulation {
    carrier_freq: f32,
    carrier_amplitude: f32,
    deviation_freq: f32
}

impl FreqModulation {
    #[must_use]
    pub fn new(carrier_freq: f32, carrier_amplitude: f32, deviation_freq: f32) -> Self {
        Self { carrier_freq, carrier_amplitude, deviation_freq }
    }

    #[must_use]
    pub fn modulate(&self, samples: &[f32]) -> Vec<f32> {
        #![allow(clippy::cast_precision_loss)]

        let mut result = vec![0.0_f32; samples.len()];
    
        let tau_mul_carrier_freq = TAU * self.carrier_freq;
        let tau_mul_deviation_freq = TAU * self.deviation_freq;
    
        let mut sum: f32 = 0.0_f32;
        for (time, (sample, modulated_sample)) in samples.iter().zip(result.iter_mut()).enumerate() {
            sum += *sample;
            *modulated_sample = self.carrier_amplitude * f32::cos((tau_mul_carrier_freq + tau_mul_deviation_freq * sum) * (time as f32));
        }
    
        result
    }
}

impl Default for FreqModulation {
    #[must_use]
    fn default() -> Self {
        Self::new(0.0_f32, 1.0_f32, 0.0005_f32)
    }
}

impl Filter for FreqModulation {
    #[must_use]
    fn filter_type(&self) -> Type {
        Type::Amplitude
    }

    #[must_use]
    fn filter(&self, data: Data) -> Option<Data> {
        match data {
            Data::Amplitude(wave) => Some(
                Data::Amplitude(
                    WaveData {
                        samples: self.modulate(&wave.samples),
                        sample_rate: wave.sample_rate
                    }
                )
            ),
            _ => None
        }
    }
}
