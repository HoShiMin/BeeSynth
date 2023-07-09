use super::wav_header::{Wave, WaveView};
use super::filter::{Data, WaveData};

impl From<WaveView<'_>> for Data {
    #[allow(clippy::cast_possible_truncation)]
    fn from(wave_view: WaveView) -> Self {
        let wave = wave_view.lookup_samples();
        match wave {
            Wave::Unknown => Data::Amplitude(
                WaveData {
                    samples: Vec::new(),
                    sample_rate: wave_view.header().sample_rate as u16
                }
            ),
            Wave::Wave8(samples) => Data::Amplitude(
                // [0, 255] -> [-1.0, 1.0]:
                WaveData {
                    samples: samples.iter()
                        .step_by(usize::from(wave_view.header().num_channels))
                        .map(|&sample| ((f32::from(sample) - f32::from(i8::MAX)) / f32::from(i8::MAX)).clamp(-1.0, 1.0))
                        .collect(),
                    sample_rate: wave_view.header().sample_rate as u16
                }
            ),
            Wave::Wave16(samples) => Data::Amplitude(
                // [-32768, 32767] -> [0.0, 1.0]:
                WaveData {
                    samples: samples.iter()
                        .step_by(usize::from(wave_view.header().num_channels))
                        .map(|&sample| (f32::from(sample) / f32::from(i16::MAX)).clamp(-1.0, 1.0))
                        .collect(),
                    sample_rate: wave_view.header().sample_rate as u16
                }
            ),
            Wave::Wave32(samples) => Data::Amplitude(
                // [-2'147'483'648, 2'147'483'647] -> [0.0, 1.0]:
                WaveData {
                    samples: samples.iter()
                        .step_by(usize::from(wave_view.header().num_channels))
                        .map(|&sample| (f64::from(sample) / f64::from(i32::MAX)).clamp(-1.0, 1.0) as f32)
                        .collect(),
                    sample_rate: wave_view.header().sample_rate as u16
                }
            )
        }
    }
}