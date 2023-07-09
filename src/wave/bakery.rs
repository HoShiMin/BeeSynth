use crate::wave::filter::PositionRecord;

use super::filter::{Filter, Type, Data, PositionData, Position};

pub type Percentage = u8;

pub enum Strategy {
    Simple, // Up if the sample is > 0, Down otherwise
    Differential(Percentage)
}

pub struct Bakery {
    strategy: Strategy
}

impl Bakery {
    pub fn new(strategy: Strategy) -> Self {
        Self { strategy }
    }
}

impl Filter for Bakery {
    fn filter_type(&self) -> Type {
        Type::Amplitude
    }

    fn filter(&self, data: Data) -> Option<Data> {
        const NS_IN_SEC: u64 = 1_000_000_000;

        let Data::Amplitude(wave) = data else {
            return None;
        };

        let sample_duration_ns = NS_IN_SEC / u64::from(wave.sample_rate);
        let mut position_data = PositionData::default();

        match self.strategy {
            Strategy::Simple => {
                for sample in wave.samples {
                    let position = if sample > 0.0_f32 {
                        Position::Up
                    } else {
                        Position::Down
                    };

                    if let Some(last) = position_data.last_mut() {
                        if last.position == position {
                            last.duration += sample_duration_ns;
                        } else {
                            position_data.push(PositionRecord {
                                position,
                                duration: sample_duration_ns
                            });
                        }
                    } else {
                        position_data.push(PositionRecord {
                            position,
                            duration: sample_duration_ns
                        });
                    }
                }
            },
            Strategy::Differential(switch_percentage) => {
                let mut previous = 0.0_f32;

                for sample in wave.samples {
                    if let Some(last) = position_data.last_mut() {
                        let diff = sample - previous;
                        let diff_percentage = (previous + diff) * 100_f32 / previous;
                        
                        if diff_percentage > f32::from(switch_percentage) {
                            let position = if diff > 0.0_f32 {
                                Position::Up
                            } else {
                                Position::Down
                            };

                            if last.position == position {
                                last.duration += sample_duration_ns;
                            } else {
                                position_data.push(PositionRecord {
                                    position,
                                    duration: sample_duration_ns
                                });
                            }
                        } else {
                            last.duration += sample_duration_ns;
                        }
                    } else {
                        let position = if sample > 0.0_f32 {
                            Position::Up
                        } else {
                            Position::Down
                        };

                        position_data.push(PositionRecord {
                            position,
                            duration: sample_duration_ns
                        });
                    }

                    previous = sample;
                }
            }
        }

        Some(Data::Position(position_data))
    }
}