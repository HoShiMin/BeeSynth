use rustfft::{FftPlanner, num_complex::Complex};

use super::filter::{Type, Data, Filter, FreqRecord, FreqChannel, FreqData};

pub struct FreqExtractor {
    lower_bound_hz: Option<u32>,
    upper_bound_hz: Option<u32>,
    sampling_size: u32,
    step_by: u32,
    sample_rate: u32,
    number_of_peaks: u8
}

impl FreqExtractor {
    #[must_use]
    pub fn new(
        lower_bound_hz: Option<u32>,
        upper_bound_hz: Option<u32>,
        sampling_size: u32, // E.g., 4096
        step_by: u32,       // E.g., 1024
        sample_rate: u32,   // E.g., 44100
        number_of_peaks: u8
    ) -> Self {
        Self { lower_bound_hz, upper_bound_hz, sampling_size, step_by, sample_rate, number_of_peaks }
    }
}

#[derive(Default, Clone, Copy)]
struct Peak {
    index: usize,
    magnitude: f32
}

const NANOS_IN_SEC: u32 = 1_000_000_000;

impl Filter for FreqExtractor {
    #[must_use]
    fn filter_type(&self) -> Type {
        Type::Amplitude
    }

    #[must_use]
    fn filter(&self, data: Data) -> Option<Data> {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_precision_loss)]
        #![allow(clippy::cast_sign_loss)]
        
        let Data::Amplitude(wave) = data else {
            return None;
        };

        let duration = (u64::from(NANOS_IN_SEC) * u64::from(self.step_by)) / u64::from(self.sample_rate);

        let fft_processor = FftPlanner::new().plan_fft_forward(self.sampling_size as usize);

        let mut channels = FreqData::default();
        channels.resize_with(usize::from(self.number_of_peaks), FreqChannel::default);

        let lower_index = match self.lower_bound_hz {
            Some(freq) => (freq * self.sampling_size) / self.sample_rate,
            None => 0
        } as usize;

        let upper_index = match self.upper_bound_hz {
            Some(freq) => (freq * self.sampling_size) / self.sample_rate,
            None => self.sampling_size / 2
        } as usize;

        for i in (0..wave.samples.len() - (self.sampling_size as usize)).step_by(self.step_by as usize) {
            let chunk = &wave.samples[i..i + self.sampling_size as usize];
            if chunk.len() < self.sampling_size as usize {
                break;
            }

            let mut samples = chunk
                .iter()
                .map(|sample| Complex::new(*sample, 0_f32))
                .collect::<Vec<Complex<f32>>>()
                ;

            fft_processor.process(&mut samples);

            let magnitudes = samples
                .into_iter()
                .skip(lower_index)
                .take(upper_index - lower_index)
                .map(|value| 20_f32 * (value.re.powi(2) + value.im.powi(2)).sqrt().log10()) // Convert to dB: 20 * log(sqrt(Re^2 + Im^2))
                .collect::<Vec<f32>>()
                ;

            let peaks = find_peaks::PeakFinder::new(&magnitudes);

            for (peak, channel) in peaks.find_peaks().iter().zip(&mut channels) {
                let freq = (self.sample_rate * (peak.middle_position() + lower_index) as u32) as f32 / self.sampling_size as f32;

                if let Some(last_sample) = channel.last_mut() {
                    let diff_percentage = freq * 100_f32 / last_sample.freq;
                    
                    if diff_percentage < 5_f32 {
                        last_sample.duration += duration;
                    } else {
                        channel.push(FreqRecord { freq, duration });
                    }
                } else {
                    channel.push(FreqRecord { freq, duration });
                }
            }            
        }

        Some(Data::Frequency(channels))
    }
}