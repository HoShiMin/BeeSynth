#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

pub mod amplitudes {
    use beeper::sound_emitter::SoundEmitter;
    use nano_sleep::NanoWaiter;
    use crate::wave::filter::{PositionRecord, Position};

    pub trait Peeker {
        fn peek(&mut self) -> Option<PositionRecord>;
    }

    pub fn play(
        emitter: &mut impl SoundEmitter,
        peeker: &mut impl Peeker,
        waiter: &impl NanoWaiter)
    {
        let mut prev_position = Position::Down;
        while let Some(sample) = peeker.peek() {
            if sample.position != prev_position {
                match sample.position {
                    Position::Up => emitter.up(),
                    Position::Down => emitter.down()
                }
                prev_position = sample.position;
            }

            waiter.nano_sleep(sample.duration);
        }
    }
}

pub mod frequencies {
    use beeper::sound_emitter::SoundEmitter;
    use nano_sleep::NanoWaiter;

    use crate::wave::filter::{FreqRecord, HertzInt, Nsec};

    pub trait Peeker {
        fn peek(&mut self, channel_number: usize) -> Option<FreqRecord<HertzInt>>;
        fn channel_count(&self) -> usize;
    }

    mod singlechannel {
        use beeper::sound_emitter::{SoundEmitter, BeeperFrequency};
        use nano_sleep::NanoWaiter;
        use super::Peeker;

        pub fn play(
            emitter: &mut impl SoundEmitter,
            waiter: &impl NanoWaiter,
            peeker: &mut impl Peeker)
        {
            emitter.prepare();
            emitter.play();
        
            let mut is_mute = false;
            
            while let Some(sample) = peeker.peek(0) {
                if sample.freq != 0 {
                    let beeper_freq = BeeperFrequency::new_clamped(sample.freq);
                    emitter.set_frequency(beeper_freq);
                    if is_mute {
                        emitter.play();
                        is_mute = false;
                    }
                } else if !is_mute {
                    emitter.mute();
                    is_mute = true;
                }
        
                waiter.nano_sleep(sample.duration);
            }
        
            emitter.mute();
        }
    }

    mod multichannel {
        use std::arch::x86_64;

        use beeper::sound_emitter::{SoundEmitter, BeeperFrequency};
        use nano_sleep::NanoWaiter;
        use crate::wave::filter::{Ticks, Nsec};

        use super::Peeker;

        enum State {
            Freq(BeeperFrequency, Ticks),
            Mute(Ticks),
            TheEnd
        }

        struct Channel {
            state: State,
            channel_number: usize
        }

        impl Channel {
            pub fn new(channel_number: usize, state: State) -> Self {
                Self { state, channel_number }
            }

            #[inline]
            #[must_use]
            pub fn channel_number(&self) -> usize {
                self.channel_number
            }

            #[inline]
            #[must_use]
            pub fn state(&self) -> &State {
                &self.state
            }

            #[must_use]
            pub fn spend(&mut self, peeker: &mut impl Peeker, ticks: Ticks, ticks_per_ns: f32) -> &State {
                let remaining_ticks = match self.state {
                    State::Freq(_, ref mut remaining_ticks)
                    | State::Mute(ref mut remaining_ticks) => remaining_ticks,
                    State::TheEnd => return &self.state
                };

                if *remaining_ticks > ticks {
                    *remaining_ticks -= ticks;
                    return &self.state;
                }
                
                let mut overflow = ticks - *remaining_ticks;
                let next_sample = loop {
                    let Some(mut next_sample) = peeker.peek(self.channel_number) else {
                        self.state = State::TheEnd;
                        return &self.state;
                    };

                    if next_sample.duration < overflow {
                        overflow -= next_sample.duration;
                        continue;
                    }

                    next_sample.duration -= overflow;
                    break next_sample;
                };

                self.state = if next_sample.freq > 0 {
                    State::Freq(BeeperFrequency::new_clamped(next_sample.freq), ((next_sample.duration as f32) * ticks_per_ns) as Ticks)
                } else {
                    State::Mute(((next_sample.duration as f32) * ticks_per_ns) as Ticks)
                };

                &self.state
            }
        }

        #[must_use]
        fn prepare_channels(peeker: &mut impl Peeker, ticks_per_ns: f32) -> Vec<Channel> {
            let mut channels = vec![];
            for channel_number in 0..peeker.channel_count() {
                if let Some(sample) = peeker.peek(channel_number) {
                    channels.push(Channel {
                        state: if sample.freq > 0 {
                            State::Freq(BeeperFrequency::new_clamped(sample.freq), ((sample.duration as f32) * ticks_per_ns) as Ticks)
                        } else {
                            State::Mute(((sample.duration as f32) * ticks_per_ns) as Ticks)
                        },
                        channel_number
                    });
                }
            }

            channels
        }

        pub fn play(
            emitter: &mut impl SoundEmitter,
            peeker: &mut impl Peeker,
            waiter: &impl NanoWaiter,
            switch_interval: Nsec)
        {            
            let ticks_per_ns = waiter.ticks_in_nanosecond();
            let ticks_per_switch_interval = ((switch_interval as f32) * ticks_per_ns) as Ticks;
        
            // Prepare the channels:
            let mut playback_channels = prepare_channels(peeker, ticks_per_ns);

            emitter.prepare();
            emitter.play();
        
            let channel_count = playback_channels.len();
            let mut is_mute = false;
            let mut previous_tick_count = unsafe { x86_64::_rdtsc() };
            let mut channel_switch_timestamp = previous_tick_count;
            let mut channel_index = 0;
            loop {
                let current_ticks = unsafe { x86_64::_rdtsc() };
                let elapsed_ticks = current_ticks - previous_tick_count;
                previous_tick_count = current_ticks;
        
                let mut has_sound = false;
                let mut has_unfinished_channels = false;
                for channel in &mut playback_channels {
                    match channel.spend(peeker, elapsed_ticks, ticks_per_ns) {
                        State::Freq(_, _) => {
                            has_sound = true;
                            has_unfinished_channels = true;
                        },
                        State::Mute(_) => {
                            has_unfinished_channels = true;
                        },
                        State::TheEnd => continue,
                    }
                }
                
                if !has_unfinished_channels {
                    break;
                }

                if has_sound == is_mute {
                    // Inverse the mute state:
                    if is_mute {
                        emitter.play();
                        is_mute = false;
                    } else {
                        emitter.mute();
                        is_mute = true;
                    }
                }

                if !has_sound {
                    continue;
                }

                let elapsed_channel_time = current_ticks - channel_switch_timestamp;
                if elapsed_channel_time < ticks_per_switch_interval {
                    // Switch time has not come yet.
                    continue;
                }

                //
                // It's time to switch the channel, find the first non-muted channel.
                // It is guaranteed that there is at least one non-muted channel.
                //
                loop {
                    if channel_index < channel_count - 1 {
                        channel_index += 1;
                    } else {
                        channel_index = 0;
                    }

                    // Enable the new channel:
                    match playback_channels[channel_index].state {
                        State::Freq(ref freq, _) => {
                            emitter.set_frequency(*freq);
                            break;
                        },
                        State::Mute(_) | State::TheEnd => continue,
                    }
                }

                channel_switch_timestamp = current_ticks;
            };
        
            emitter.mute();
        }
    }

    pub fn play(
        emitter: &mut impl SoundEmitter,
        peeker: &mut impl Peeker,
        waiter: &impl NanoWaiter,
        switch_interval: Nsec)
    {
        match peeker.channel_count() {
            0 => (), // There is nothing to play.
            1 => singlechannel::play(emitter, waiter, peeker),
            _ => multichannel::play(emitter, peeker, waiter, switch_interval)
        }
    }
}
