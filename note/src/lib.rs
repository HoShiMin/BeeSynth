#![warn(clippy::pedantic)]

use std::str::{self, FromStr};

pub type SemitoneNumber = u8;
pub type OctaveNumber = u8;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum Shift {
    #[default]
    None,
    Diesis,
    Bemolle
}

#[derive(PartialOrd, Copy, Clone, Debug)]
pub enum Note {
    C(OctaveNumber),
    Cs(OctaveNumber),
    Db(OctaveNumber),
    D(OctaveNumber),
    Ds(OctaveNumber),
    Eb(OctaveNumber),
    E(OctaveNumber),
    F(OctaveNumber),
    Fs(OctaveNumber),
    Gb(OctaveNumber),
    G(OctaveNumber),
    Gs(OctaveNumber),
    Ab(OctaveNumber),
    A(OctaveNumber),
    As(OctaveNumber),
    Bb(OctaveNumber),
    B(OctaveNumber)
}

impl Note {
    pub const SEMITONE_COUNT: u8 = 12;

    #[must_use]
    pub const fn from_semitone(semitone: SemitoneNumber) -> Note {
        let octave = semitone / Self::SEMITONE_COUNT;
        match semitone - (octave * Self::SEMITONE_COUNT) {
            0 => Note::C(octave),
            1 => Note::Cs(octave),
            2 => Note::D(octave),
            3 => Note::Ds(octave),
            4 => Note::E(octave),
            5 => Note::F(octave),
            6 => Note::Fs(octave),
            7 => Note::G(octave),
            8 => Note::Gs(octave),
            9 => Note::A(octave),
            10 => Note::As(octave),
            11 => Note::B(octave),
            _ => unreachable!()
        }
    }

    #[must_use]
    pub fn find_nearest(freq_hz: f32) -> Note {
        let mut lower_bound = 0_u8;
        let mut upper_bound = u8::MAX;
        loop {
            let median = lower_bound + (upper_bound - lower_bound) / 2;
            let probe_note = Note::from_semitone(median);
            let probe_freq = probe_note.freq();

            let median_freq_delta = (freq_hz - probe_freq).abs();

            if (lower_bound == upper_bound) || (median_freq_delta < 0.5_f32) {
                return probe_note;
            }

            if median == lower_bound {
                let upper_note = Note::from_semitone(upper_bound);
                if (upper_note.freq() - freq_hz).abs() > median_freq_delta {
                    return probe_note;
                }
                return upper_note;
            }

            if median == upper_bound {
                let lower_note = Note::from_semitone(lower_bound);
                if (lower_note.freq() - freq_hz).abs() > median_freq_delta {
                    return probe_note;
                }
                return lower_note;
            }

            if freq_hz < probe_freq {
                upper_bound = median;
            } else if freq_hz > probe_freq {
                lower_bound = median;
            } else {
                return probe_note;
            }
        }
    }

    #[must_use]
    pub const fn octave_number(&self) -> OctaveNumber {
        match *self {
            Note::C(octave_number)  |
            Note::Cs(octave_number) |
            Note::Db(octave_number) |
            Note::D(octave_number)  |
            Note::Ds(octave_number) |
            Note::Eb(octave_number) |
            Note::E(octave_number)  |
            Note::F(octave_number)  |
            Note::Fs(octave_number) |
            Note::Gb(octave_number) |
            Note::G(octave_number)  |
            Note::Gs(octave_number) |
            Note::Ab(octave_number) |
            Note::A(octave_number)  |
            Note::As(octave_number) |
            Note::Bb(octave_number) |
            Note::B(octave_number)  => octave_number
        }
    }

    #[must_use]
    pub const fn semitone_shift(&self) -> Shift {
        match *self {
            Note::Cs(_) | Note::Ds(_) | Note::Fs(_) | Note::Gs(_) | Note::As(_) => Shift::Diesis,
            Note::Db(_) | Note::Eb(_) | Note::Gb(_) | Note::Ab(_) | Note::Bb(_) => Shift::Bemolle,
            _ => Shift::None
        }
    }

    #[must_use]
    pub fn semitone_number(&self) -> SemitoneNumber {
        match *self {
            Note::C(octave_number) => octave_number * Self::SEMITONE_COUNT,
            Note::Cs(octave_number) | Note::Db(octave_number) => octave_number * Self::SEMITONE_COUNT + 1,
            Note::D(octave_number) => octave_number * Self::SEMITONE_COUNT + 2,
            Note::Ds(octave_number) | Note::Eb(octave_number) => octave_number * Self::SEMITONE_COUNT + 3,
            Note::E(octave_number) => octave_number * Self::SEMITONE_COUNT + 4,
            Note::F(octave_number) => octave_number * Self::SEMITONE_COUNT + 5,
            Note::Fs(octave_number) | Note::Gb(octave_number) => octave_number * Self::SEMITONE_COUNT + 6,
            Note::G(octave_number) => octave_number * Self::SEMITONE_COUNT + 7,
            Note::Gs(octave_number) | Note::Ab(octave_number) => octave_number * Self::SEMITONE_COUNT + 8,
            Note::A(octave_number) => octave_number * Self::SEMITONE_COUNT + 9,
            Note::As(octave_number) | Note::Bb(octave_number) => octave_number * Self::SEMITONE_COUNT + 10,
            Note::B(octave_number) => octave_number * Self::SEMITONE_COUNT + 11,
        }
    }

    #[must_use]
    pub fn freq(&self) -> f32 {
        const A4: Note = Note::A(4);
        const A4_FREQ: f32 = 440_f32;

        // https://en.wikipedia.org/wiki/Musical_note
        //
        // f = 440 Hz * 2^(n/12), where:
        //   12 - number of semitones in gamma.
        //   440 Hz - frequency of the A4 note.
        //   n - semitone number relative to the A4 note.
        //
        let power = (f32::from(self.semitone_number()) - f32::from(A4.semitone_number())) / 12_f32;
        A4_FREQ * 2_f32.powf(power)
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        match *self {
            Note::C(octave_number)  => format!("C{octave_number}"),
            Note::Cs(octave_number) => format!("C{octave_number}#"),
            Note::Db(octave_number) => format!("D{octave_number}b"),
            Note::D(octave_number)  => format!("D{octave_number}"),
            Note::Ds(octave_number) => format!("D{octave_number}#"),
            Note::Eb(octave_number) => format!("E{octave_number}b"),
            Note::E(octave_number)  => format!("E{octave_number}"),
            Note::F(octave_number)  => format!("F{octave_number}"),
            Note::Fs(octave_number) => format!("F{octave_number}#"),
            Note::Gb(octave_number) => format!("G{octave_number}b"),
            Note::G(octave_number)  => format!("G{octave_number}"),
            Note::Gs(octave_number) => format!("G{octave_number}#"),
            Note::Ab(octave_number) => format!("A{octave_number}b"),
            Note::A(octave_number)  => format!("A{octave_number}"),
            Note::As(octave_number) => format!("A{octave_number}#"),
            Note::Bb(octave_number) => format!("B{octave_number}b"),
            Note::B(octave_number)  => format!("B{octave_number}"),
        }
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::C(left_octave_number) , Self::C(right_octave_number))   |
            (Self::Cs(left_octave_number) | Self::Db(left_octave_number), Self::Cs(right_octave_number) | Self::Db(right_octave_number)) |
            (Self::D(left_octave_number) , Self::D(right_octave_number))   |
            (Self::Ds(left_octave_number) | Self::Eb(left_octave_number), Self::Ds(right_octave_number) | Self::Eb(right_octave_number)) |
            (Self::E(left_octave_number) , Self::E(right_octave_number))   |
            (Self::F(left_octave_number) , Self::F(right_octave_number))   |
            (Self::Fs(left_octave_number) | Self::Gb(left_octave_number), Self::Fs(right_octave_number) | Self::Gb(right_octave_number)) |
            (Self::G(left_octave_number) , Self::G(right_octave_number))   |
            (Self::Gs(left_octave_number) | Self::Ab(left_octave_number), Self::Gs(right_octave_number) | Self::Ab(right_octave_number)) |
            (Self::A(left_octave_number) , Self::A(right_octave_number))   |
            (Self::As(left_octave_number) | Self::Bb(left_octave_number), Self::As(right_octave_number) | Self::Bb(right_octave_number)) |
            (Self::B(left_octave_number) , Self::B(right_octave_number))   => left_octave_number == right_octave_number,
            _ => false,
        }
    }
}


impl FromStr for Note {
    type Err = &'static str;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let mut chars = name.chars();

        let Some(note_name) = chars.next() else {
            return Err("Invalid note format: the note letter [A..G] or [a..g] is not present.");
        };

        let octave_number_count = chars.clone().take_while(|ch| (*ch >= '0') && (*ch <= '9')).count();
        if octave_number_count == 0 {
            return Err("Invalid octave format: must be the integer number.");
        }

        let Ok(octave_number) = chars.as_str()[0..octave_number_count].parse::<OctaveNumber>() else {
            return Err("Invalid octave number: must be in the range of 0..255.");
        };

        let shift = if let Some(shift_char) = chars.nth(octave_number_count) {
            match shift_char {
                's' | '#' | '♯' => Shift::Diesis,
                'b' | '♭' => Shift::Bemolle,
                _ => return Err("Invalid semitone shift format: must absent or be one of [s, #, ♯, b, ♭]")
            }
        } else {
            Shift::None
        };

        let note = match note_name {
            'A' | 'a' => match shift {
                Shift::None => Note::A(octave_number),
                Shift::Diesis => Note::As(octave_number),
                Shift::Bemolle => Note::Ab(octave_number)
            },
            'B' | 'b' => match shift {
                Shift::None => Note::B(octave_number),
                Shift::Diesis => Note::C(octave_number + 1),
                Shift::Bemolle => Note::Bb(octave_number)
            },
            'C' | 'c' => match shift {
                Shift::None => Note::C(octave_number),
                Shift::Diesis => Note::Cs(octave_number),
                Shift::Bemolle => if octave_number > 0 {
                    Note::B(octave_number - 1)
                } else {
                    return Err("Note is too low.");
                }
            },
            'D' | 'd' => match shift {
                Shift::None => Note::D(octave_number),
                Shift::Diesis => Note::Ds(octave_number),
                Shift::Bemolle => Note::Db(octave_number)
            },
            'E' | 'e' => match shift {
                Shift::None => Note::E(octave_number),
                Shift::Diesis => Note::F(octave_number),
                Shift::Bemolle => Note::Eb(octave_number),
            },
            'F' | 'f' => match shift {
                Shift::None => Note::F(octave_number),
                Shift::Diesis => Note::Fs(octave_number),
                Shift::Bemolle => Note::E(octave_number),
            },
            'G' | 'g' => match shift {
                Shift::None => Note::G(octave_number),
                Shift::Diesis => Note::Gs(octave_number),
                Shift::Bemolle => Note::Gb(octave_number)
            },
            _ => return Err("Invalid note name: unexpected note letter, must be in the range of [A..G] or [a..g].")
        };

        Ok(note)
    }
}



#[test]
fn test() {
    assert_eq!(Note::from_str("c4s").unwrap(), Note::Cs(4));
    assert_eq!(Note::from_str("C4#").unwrap(), Note::Db(4));
    assert_eq!(Note::from_str("c135s").unwrap().to_string(), "C135#");
    assert_eq!(Note::from_semitone(16), Note::E(1));

    for i in 0..=u8::MAX {
        let probe_note = Note::from_semitone(i);
        
        let found = Note::find_nearest(probe_note.freq());
        assert_eq!(found, probe_note);

        let found = Note::find_nearest(probe_note.freq());
        assert_eq!(found, probe_note);
    }
}