use std::arch::x86_64;

pub type PatternIndex = u8;

#[repr(C, packed(1))]
pub struct XmHeader {
    pub id_text: [u8; 17],
    pub module_name: [u8; 20],
    pub must_be_0x1a: u8, // Must be 0x1A
    pub tracker_name: [u8; 20],
    pub version_number: u16,
    pub header_size: u32,
    pub song_length: u16,
    pub restart_position: u16,
    pub number_of_channels: u16,
    pub number_of_patterns: u16,
    pub number_of_instruments: u16,
    pub flags: u16,
    pub default_tempo: u16,
    pub default_bpm: u16,
    pub pattern_order_table: [PatternIndex; 256]
}

impl XmHeader {
    #[must_use]
    pub fn from_raw(xm_file: &[u8]) -> &XmHeader {
        unsafe { &*std::ptr::addr_of!(xm_file[0]).cast() }
    }

    #[must_use]
    pub fn first_pattern_header(&self) -> *const PatternHeader {
        unsafe {
            std::ptr::addr_of!(*self).add(1).cast()
        }
    }

    #[must_use]
    pub fn pattern_headers(&self) -> Vec<&PatternHeader> {
        let mut headers = vec![];
        unsafe {
            let mut pattern_header = self.first_pattern_header();
            for _ in 0..self.number_of_patterns {
                headers.push(&*pattern_header);

                let pattern_size = (u32::from((*pattern_header).packed_pattern_data_size)
                    + (*pattern_header).pattern_header_length).try_into().expect("Fuck you asshole");

                pattern_header = pattern_header.cast::<u8>().add(pattern_size).cast();
            }
        }
        headers
    }

    #[must_use]
    pub fn is_xm(raw: &[u8]) -> bool {
        const EXTENDED_MODULE_SIG: &str = "Extended Module:";
        const TRACKER_NAME_SIG: &str = "FastTracker v2.00   ";

        if raw.len() < std::mem::size_of::<XmHeader>() {
            return false;
        }

        let header = XmHeader::from_raw(raw);

        let xm_sig_bytes = EXTENDED_MODULE_SIG.as_bytes();

        for sym in header.id_text.iter().zip(xm_sig_bytes) {
            if sym.0 != sym.1 {
                return false;
            }
        }

        let tracker_sig_bytes = TRACKER_NAME_SIG.as_bytes();
        for sym in header.tracker_name.iter().zip(tracker_sig_bytes) {
            if sym.0 != sym.1 {
                return false;
            }
        }

        true
    }
}



pub enum PatternType {
    Fixed,
    Packed
}

#[repr(C, packed(1))]
pub struct Pattern {
    pub note_or_flags: u8
}

#[derive(Default, Copy, Clone)]
pub struct PatternData {
    pub note_freq: Option<f32>,
    pub note: Option<u8>,
    pub instrument: u8,
    pub volume: u8,
    pub effect: u8,
    pub effect_param: u8
}

pub trait GenericPattern {
    #[must_use]
    fn next_pattern(&self) -> *const Pattern;
    
    #[must_use]
    fn pattern(&self) -> PatternData;
}

impl Pattern {
    const TYPE_MASK: u8 = 0b1000_0000;

    /// # Panics
    /// Will panic in case of invalid WAV format
    #[must_use]
    pub fn pattern_type(&self) -> PatternType {
        const TYPE_FIXED: u8 = 0;
        const TYPE_PACKED: u8 = 0b1000_0000;

        match self.note_or_flags & Self::TYPE_MASK {
            TYPE_FIXED => PatternType::Fixed,
            TYPE_PACKED => PatternType::Packed,
            _ => panic!("Unknown type of pattern")
        }
    }

    #[must_use]
    pub fn as_fixed(&self) -> *const FixedPattern {
        std::ptr::addr_of!(*self).cast()
    }

    #[must_use]
    pub fn as_packed(&self) -> *const PackedPattern {
        std::ptr::addr_of!(*self).cast()
    }
}

impl GenericPattern for Pattern {
    fn next_pattern(&self) -> *const Pattern {
        match self.pattern_type() {
            PatternType::Fixed => unsafe { (*self.as_fixed()).next_pattern() },
            PatternType::Packed => unsafe { (*self.as_packed()).next_pattern() }
        }
    }

    fn pattern(&self) -> PatternData {
        match self.pattern_type() {
            PatternType::Fixed => unsafe { (*self.as_fixed()).pattern() },
            PatternType::Packed => unsafe { (*self.as_packed()).pattern() }
        }
    }
}



#[repr(C, packed(1))]
pub struct FixedPattern {
    pub note: u8,
    pub instrument: u8,
    pub volume_column_byte: u8,
    pub effect_type: u8,
    pub effect_parameter: u8
}



static NOTES: &[f32] = &[
    /* 00 */ 0.0, // No note

    /* 01 */ 32.70, // C1
    /* 02 */ 34.65, // C1s/D1b
    /* 03 */ 36.71, // D1
    /* 04 */ 38.89, // D1s/E1b
    /* 05 */ 41.20, // E1
    /* 06 */ 43.65, // F1
    /* 07 */ 46.25, // F1s/G1b
    /* 08 */ 49.00, // G1
    /* 09 */ 51.91, // G1s/A1b
    /* 0A */ 55.00, // A1
    /* 0B */ 58.27, // A1s/B1b
    /* 0C */ 61.74, // B1

    /* 0D */ 65.41, // C2
    /* 0E */ 69.30, // C2s/D2b
    /* 0F */ 73.42, // D2
    /* 10 */ 77.78, // D2s/E2b
    /* 11 */ 82.41, // E2
    /* 12 */ 87.31, // F2
    /* 13 */ 92.50, // F2s/G2b
    /* 14 */ 98.00, // G2
    /* 15 */ 103.83, // G2s/A2b
    /* 16 */ 110.00, // A2
    /* 17 */ 116.54, // A2s/B2b
    /* 18 */ 123.47, // B2

    /* 19 */ 130.81, // C3
    /* 1A */ 138.59, // C3s/D3b
    /* 1B */ 146.83, // D3
    /* 1C */ 155.56, // D3s/E3b
    /* 1D */ 164.81, // E3
    /* 1E */ 174.61, // F3
    /* 1F */ 185.00, // F3s/G3b
    /* 20 */ 196.00, // G3
    /* 21 */ 207.65, // G3s/A3b
    /* 22 */ 220.00, // A3
    /* 23 */ 233.08, // A3s/B3b
    /* 24 */ 246.94, // B3

    /* 25 */ 261.63, // C4
    /* 26 */ 277.18, // C4s/D4b
    /* 27 */ 293.66, // D4
    /* 28 */ 311.13, // D4s/E4b
    /* 29 */ 329.63, // E4
    /* 2A */ 349.23, // F4
    /* 2B */ 369.99, // F4s/G4b
    /* 2C */ 392.00, // G4
    /* 2D */ 415.30, // G4s/A4b
    /* 2E */ 440.00, // A4
    /* 2F */ 466.16, // A4s/B4b
    /* 30 */ 493.88, // B4

    /* 31 */ 523.25, // C5
    /* 32 */ 554.37, // C5s/D5b
    /* 33 */ 587.33, // D5
    /* 34 */ 622.25, // D5s/E5b
    /* 35 */ 659.25, // E5
    /* 36 */ 698.46, // F5
    /* 37 */ 739.99, // F5s/G5b
    /* 38 */ 783.99, // G5
    /* 39 */ 830.61, // G5s/A5b
    /* 3A */ 880.00, // A5
    /* 3B */ 932.33, // A5s/B5b
    /* 3C */ 987.77, // B5

    /* 3D */ 1046.50, // C6
    /* 3E */ 1108.73, // C6s/D6b
    /* 3F */ 1174.66, // D6
    /* 40 */ 1244.51, // D6s/E6b
    /* 41 */ 1318.51, // E6
    /* 42 */ 1396.91, // F6
    /* 43 */ 1479.98, // F6s/G6b
    /* 44 */ 1567.98, // G6
    /* 45 */ 1661.22, // G6s/A6b
    /* 46 */ 1760.00, // A6
    /* 47 */ 1864.66, // A6s/B6b
    /* 48 */ 1975.53, // B6

    /* 49 */ 2093.00, // C7
    /* 4A */ 2217.46, // C7s/D7b
    /* 4B */ 2349.32, // D7
    /* 4C */ 2489.02, // D7s/E7b
    /* 4D */ 2637.02, // E7
    /* 4E */ 2793.83, // F7
    /* 4F */ 2959.96, // F7s/G7b
    /* 50 */ 3135.96, // G7
    /* 51 */ 3322.44, // G7s/A7b
    /* 52 */ 3520.00, // A7
    /* 53 */ 3729.31, // A7s/B7b
    /* 54 */ 3951.07, // B7

    /* 55 */ 4186.01, // C8
    /* 56 */ 4434.92, // C8s/D8b
    /* 57 */ 4698.63, // D8
    /* 58 */ 4978.03, // D8s/E8b
    /* 59 */ 5274.04, // E8
    /* 5A */ 5587.65, // F8
    /* 5B */ 5919.91, // F8s/G8b
    /* 5C */ 6271.93, // G8
    /* 5D */ 6644.88, // G8s/A8b
    /* 5E */ 7040.00, // A8
    /* 5F */ 7458.62, // A8s/B8b
    /* 60 */ 7902.13  // B8
];

impl GenericPattern for FixedPattern {
    fn next_pattern(&self) -> *const Pattern {
        unsafe { std::ptr::addr_of!(*self).add(1).cast() }
    }

    fn pattern(&self) -> PatternData {
        PatternData {
            note_freq: if (self.note as usize) < NOTES.len() {
                Some(NOTES[self.note as usize])
            } else {
                None
            },
            note: if (self.note as usize) < NOTES.len() {
                Some(self.note)
            } else {
                None
            },
            instrument: self.instrument,
            volume: self.volume_column_byte,
            effect: self.effect_type,
            effect_param: self.effect_parameter
        }
    }
}



#[repr(C, packed(1))]
pub struct PackedPattern {
    pub mask: u8
}

impl PackedPattern {
    const HAS_NOTE_MASK             : u8 = 0b0000_0001;
    const HAS_INSTRUMENT_MASK       : u8 = 0b0000_0010;
    const HAS_VOLUME_MASK           : u8 = 0b0000_0100;
    const HAS_EFFECT_TYPE_MASK      : u8 = 0b0000_1000;
    const HAS_EFFECT_PARAMETER_MASK : u8 = 0b0001_0000;

    #[must_use]
    pub fn len(&self) -> usize {
        #[allow(clippy::cast_sign_loss)] // It's safe as popcnt is always positive
        unsafe { x86_64::_popcnt32(i32::from(self.mask)) as usize }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        (self.mask & !Pattern::TYPE_MASK) == 0
    }
}

impl GenericPattern for PackedPattern {
    fn next_pattern(&self) -> *const Pattern {
        unsafe { std::ptr::addr_of!(*self).cast::<u8>().add(self.len()).cast() }
    }

    fn pattern(&self) -> PatternData {
        let note = {
            if (self.mask & Self::HAS_NOTE_MASK) == 0 {
                Some(0_u8)
            } else {
                let note_index = unsafe { *std::ptr::addr_of!(self.mask).add(1) };
                if usize::from(note_index) < NOTES.len() {
                    Some(note_index)
                } else {
                    None
                }
            }
        };

        PatternData {
            note_freq: note.map(|note| NOTES[note as usize]),
            note,
            instrument: if (self.mask & Self::HAS_INSTRUMENT_MASK) == 0 {
                0
            } else {
                unsafe { *std::ptr::addr_of!(self.mask).add(2) }
            },
            volume: if (self.mask & Self::HAS_VOLUME_MASK) == 0 {
                0
            } else {
                unsafe { *std::ptr::addr_of!(self.mask).add(3) }
            },
            effect: if (self.mask & Self::HAS_EFFECT_TYPE_MASK) == 0 {
                0
            } else {
                unsafe { *std::ptr::addr_of!(self.mask).add(4) }
            },
            effect_param: if (self.mask & Self::HAS_EFFECT_PARAMETER_MASK) == 0 {
                0
            } else {
                unsafe { *std::ptr::addr_of!(self.mask).add(5) }
            }
        }
    }
}





#[repr(C, packed(1))]
pub struct PatternHeader {
    pub pattern_header_length: u32,
    pub packing_type: u8,
    pub number_of_rows_in_pattern: u16,
    pub packed_pattern_data_size: u16,
    //pub patterns: [Pattern]
}

impl PatternHeader {
    #[must_use]
    pub fn first_pattern(&self) -> *const Pattern {
        unsafe {
            std::ptr::addr_of!(*self).cast::<u8>().add(self.pattern_header_length as usize).cast()
        }
    }
}