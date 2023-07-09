use note::Note;

use super::filter::{Type, Data, Filter};

pub struct NoteMatcher;

impl Filter for NoteMatcher {
    fn filter_type(&self) -> Type {
        Type::Frequency
    }

    fn filter(&self, data: Data) -> Option<Data> {
        let Data::Frequency(mut channels) = data else {
            return None;
        };

        channels.iter_mut().flatten().for_each(|freq_record| {
            freq_record.freq = Note::find_nearest(freq_record.freq).freq();
        });

        Some(Data::Frequency(channels))
    }
}