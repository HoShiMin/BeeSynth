use crate::nano_sleep::NanoWaiter;
use crate::sound_emitter::SoundEmitter;
use crate::xm_header::{XmHeader, GenericPattern, PatternData};
use crate::playback_event::{Action, FreqHz, Percentage};

use std::time::{Instant, Duration};


pub fn play(beeper: &mut impl SoundEmitter, sleep: &impl NanoWaiter, xm_buf: &[u8], callback: Option<&impl Fn(FreqHz, Percentage) -> Action>) {
    if !XmHeader::is_xm(xm_buf) {
        return;
    }

    let xm = XmHeader::from_raw(xm_buf);

    let song_length = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(xm.song_length)) };
    let default_tempo = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(xm.default_tempo)) };
    let default_bpm = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(xm.default_bpm)) };
    let number_of_patterns = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(xm.number_of_patterns)) };
    let number_of_channels = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(xm.number_of_channels)) };

    println!("Tempo: {default_tempo}, BPM: {default_bpm}, Pattern count: {number_of_patterns}, Channel count: {number_of_channels}, Song length: {song_length}");

    let pattern_headers = xm.pattern_headers();
    'pattern_loop: for pattern_pos in 0..song_length as usize {
        let pattern_index = xm.pattern_order_table[pattern_pos];
        
        println!("Pattern index: {pattern_index}");

        if (pattern_index as usize) > pattern_headers.len() {
            println!("Pattern index > count of pattern headers");
            break;
        }
        
        let pattern_header = pattern_headers[pattern_index as usize];
        let mut row = vec![PatternData::default(); number_of_channels as usize];
        let mut pattern_pointer = pattern_header.first_pattern();

        let row_duration = Duration::from_millis(20 * u64::from(default_tempo));
        let switch_interval_nanos = 1000 * 1000 * 15;

        for _ in 0..pattern_header.number_of_rows_in_pattern as usize {
            let mut mute_pattern = true;
            for col_in_row in &mut row {
                let pattern = unsafe { (*pattern_pointer).pattern() };
                
                if let Some(note) = pattern.note {
                    if note != 0 {
                        col_in_row.note = Some(note);
                        col_in_row.note_freq = pattern.note_freq;
                        col_in_row.volume = 64; // Default volume
                    }
                } else {
                    col_in_row.note = None;
                    col_in_row.note_freq = None;
                }

                if pattern.volume != 0 {
                    col_in_row.volume = pattern.volume;
                }

                if pattern.effect != 0 {
                    col_in_row.effect = pattern.effect;
                }

                if pattern.effect_param != 0 {
                    col_in_row.effect_param = pattern.effect_param;
                }

                if col_in_row.note.is_some() {
                    mute_pattern = false;
                }

                pattern_pointer = unsafe { (*pattern_pointer).next_pattern() };
            }            

            let end_time = Instant::now() + row_duration;
            if mute_pattern {
                beeper.mute();
            } else {
                beeper.play();
            }

            while Instant::now() < end_time {
                for col_in_row in &row {
                    match col_in_row.note_freq {
                        Some(freq) => {
                            beeper.set_frequency(freq.into());
                            sleep.nano_sleep(switch_interval_nanos);
                            if Instant::now() >= end_time {
                                break;
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }

            if let Some(callback) = &callback {
                let action = callback(123, 123);
                if let Action::Break = action {
                    break 'pattern_loop
                }
            }
        }
    }

    beeper.mute();
}