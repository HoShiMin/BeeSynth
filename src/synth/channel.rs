use crate::wave::filter::{FreqRecord, FreqData};

use super::note_record::{NoteRecord, NoteStyle};

pub type Channel = Vec<NoteRecord>;

pub struct Channels {
    channels: Vec<Channel>,
    bpm: u16 // Bits per minute
}

impl Channels {
    #[must_use]
    pub fn new(bpm: u16) -> Channels {
        Channels { channels: Vec::new(), bpm }
    }

    pub fn push(&mut self, channel: Channel) {
        self.channels.push(channel);
    }

    #[must_use]
    pub fn bpm(&self) -> u16 {
        self.bpm
    }

    #[must_use]
    pub fn channels(&self) -> &Vec<Channel> {
        &self.channels
    }
}



impl From<Channels> for crate::wave::filter::Data {
    fn from(channels: Channels) -> Self {
        let mut freq_data = FreqData::default();

        for channel in channels.channels() {
            let mut freq_channel = vec![];
            for note in channel {
                if let Some(freq) = note.freq() {
                    let duration = note.duration_nsec(channels.bpm());
                    freq_channel.push(FreqRecord { freq, duration });

                    if note.style() != NoteStyle::Legato {
                        let unstyled_duration = note.duration_nsec_unstyled(channels.bpm());
                        if unstyled_duration > duration {
                            freq_channel.push(FreqRecord { freq: 0.0_f32, duration: unstyled_duration - duration });
                        }
                    }
                } else {
                    // It's a pause, ignore all styles:
                    freq_channel.push(FreqRecord {
                        freq: 0.0_f32,
                        duration: note.duration_nsec_unstyled(channels.bpm())
                    });
                }
            }

            freq_data.push(freq_channel);
        }

        crate::wave::filter::Data::Frequency(freq_data)
    }
}