
use std::str::FromStr;

use note::Note;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NoteDivisor {
    Whole = 1,
    Half = 2,
    Quarter = 4,
    Eighth = 8,
    Sixtinth = 16,
    ThirtySecond = 32,
    SixtyFourth = 64
}

impl From<NoteDivisor> for u8 {
    fn from(value: NoteDivisor) -> Self {
        match value {
            NoteDivisor::Whole => 1,
            NoteDivisor::Half => 2,
            NoteDivisor::Quarter => 4,
            NoteDivisor::Eighth => 8,
            NoteDivisor::Sixtinth => 16,
            NoteDivisor::ThirtySecond => 32,
            NoteDivisor::SixtyFourth => 64
        }
    }
}

impl AsRef<str> for NoteDivisor {
    fn as_ref(&self) -> &str {
        match self {
            NoteDivisor::Whole => "W",
            NoteDivisor::Half => "H",
            NoteDivisor::Quarter => "Q",
            NoteDivisor::Eighth => "E",
            NoteDivisor::Sixtinth => "S",
            NoteDivisor::ThirtySecond => "T",
            NoteDivisor::SixtyFourth => "F"
        }
    }
}

impl FromStr for NoteDivisor {
    type Err = String;

    fn from_str(divisor: &str) -> Result<Self, Self::Err> {
        match divisor {
            "W" | "1" => Ok(NoteDivisor::Whole),
            "H" | "2" => Ok(NoteDivisor::Half),
            "Q" | "4" => Ok(NoteDivisor::Quarter),
            "E" | "8" => Ok(NoteDivisor::Eighth),
            "S" | "16" => Ok(NoteDivisor::Sixtinth),
            "T" | "32" => Ok(NoteDivisor::ThirtySecond),
            "X" | "64" => Ok(NoteDivisor::SixtyFourth),
            _ => Err(format!("Unknown note divisor: {divisor}"))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NoteStyle {
    NonLegato,
    Legato,
    Staccato,
    Prolongate,
}

impl AsRef<str> for NoteStyle {
    fn as_ref(&self) -> &str {
        match self {
            NoteStyle::NonLegato => "",
            NoteStyle::Legato => "~",
            NoteStyle::Staccato => "!",
            NoteStyle::Prolongate => "."
        }
    }
}

impl FromStr for NoteStyle {
    type Err = String;

    fn from_str(style: &str) -> Result<Self, Self::Err> {
        match style {
            "" => Ok(NoteStyle::NonLegato),
            "~" => Ok(NoteStyle::Legato),
            "!" => Ok(NoteStyle::Staccato),
            "." => Ok(NoteStyle::Prolongate),
            _ => Err(format!("Unknown note style: {style}"))
        }
    }
}

impl NoteStyle {
    #[must_use]
    pub fn multiplier(self) -> f32 {
        match self {
            NoteStyle::NonLegato => 0.8_f32,
            NoteStyle::Legato => 1.0_f32,
            NoteStyle::Staccato => 0.25_f32,
            NoteStyle::Prolongate => 1.5_f32
        }
    }

    #[must_use]
    pub const fn is_note_style(sym: char) -> bool {
        matches!(sym, '~' | '!' | '.')
    }
}

#[derive(Debug, PartialEq)]
pub struct NoteRecord {
    note: Option<Note>,
    divisor: NoteDivisor,
    style: NoteStyle
}

impl Default for NoteRecord {
    fn default() -> Self {
        NoteRecord { note: None, divisor: NoteDivisor::Quarter, style: NoteStyle::NonLegato }
    }
}

impl NoteRecord {
    #[must_use]
    #[allow(dead_code)]
    pub const fn new(note: Option<Note>, divisor: NoteDivisor, style: NoteStyle) -> NoteRecord {
        NoteRecord { note, divisor, style }
    }

    #[must_use]
    pub fn freq(&self) -> Option<f32> {
        self.note.map(|note| note.freq())
    }

    #[must_use]
    pub fn divisor(&self) -> NoteDivisor {
        self.divisor
    }

    #[must_use]
    pub fn style(&self) -> NoteStyle {
        self.style
    }

    #[must_use]
    pub fn duration_nsec_unstyled(&self, bpm: u16) -> u64 {
        const NSEC_IN_MSEC: u64 = 1_000_000;
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let duration_msec = ((4_f32 * 60_000_f32) / (f32::from(bpm) * f32::from(u8::from(self.divisor())))) as u64;
        duration_msec * NSEC_IN_MSEC
    }

    #[must_use]
    pub fn duration_nsec(&self, bpm: u16) -> u64 {
        const NSEC_IN_MSEC: u64 = 1_000_000;
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let duration_msec = ((self.style().multiplier() * 4_f32 * 60_000_f32) / (f32::from(bpm) * f32::from(u8::from(self.divisor())))) as u64;
        duration_msec * NSEC_IN_MSEC
    }
}

impl FromStr for NoteRecord {
    type Err = String;

    fn from_str(record: &str) -> Result<Self, Self::Err> {
        if record.is_empty() {
            return Err(String::from("Empty note record"));
        }

        let delimiter = record.find(':').ok_or_else(|| format!("Missing note delimiter in the {record}"))?;
        
        if delimiter == 0 {
            return Err(format!("Missing note params in the {record}"));
        }

        if delimiter == record.len() - 1 {
            return Err(format!("Missing note in the {record}"));
        }

        let note_params = &record[0..delimiter];
        

        let Some(first_char) = note_params.chars().next() else {
            return Err(format!("Missing note params in the {record}"));
        };

        let (style, divisor) = if NoteStyle::is_note_style(first_char) {
            let note_style = NoteStyle::from_str(&note_params[0..1])?;
            let divisor = NoteDivisor::from_str(&note_params[1..delimiter])?;
            (note_style, divisor)
        } else {
            let divisor = NoteDivisor::from_str(&note_params[0..delimiter])?;
            (NoteStyle::NonLegato, divisor)
        };

        let note_token = &record[delimiter + 1..];
        if note_token == "0" {
            return Ok(NoteRecord { note: None, divisor, style });
        }

        let note = Note::from_str(note_token)?;
        Ok(NoteRecord { note: Some(note), divisor, style })
    }
}

impl std::fmt::Display for NoteRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{style}{divisor}:{note}",
            style = self.style.as_ref(),
            divisor = self.divisor.as_ref(),
            note = if let Some(note) = self.note {
                note.to_string()
            } else {
                String::from("0")
            }
        )
    }
}