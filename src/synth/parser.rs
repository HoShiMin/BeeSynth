///
/// Supported formats:
/// 
///     Comments:
///         ; Something
/// 
///     A standalone note format (square brackets are optional):
///         [Style]Duration:Note
/// 
///     Grouped notes format:
///         GroupFactor:[Note, Note, Note, ...]
/// 
///     Style (optional):
///         Default is non-legato (without a prefix)
///         ~ = Legato (a marked note with the next one)
///         ! = Staccato
///         . = Prolongated by half of the note
/// 
///     Duration (required):
///         W = Whole
///         H = Half (1/2 of Whole)
///         Q = Quarter (1/4 of Whole)
///         O = Eighth (1/8 of Whole)
///         S = Sixtinth (1/16 of Whole)
///         T = Thirty-second (1/32 of Whole)
///         X = Sixty-fourth (1/64 of Whole)
///
///     _____________________________________________
///     Examples:
///         !Q:E3 - Play the quarter note E3 using staccato
///         ~Q:E3 Q:F3 - Play the sequence using legato from E3 to F3
/// 
/// #!/bin/beesynth
/// @name Test
/// @bpm: 120
/// @channels: ch1
/// @ch1: !Q:E3 ~Q:E3 Q:F3
/// 

use std::{collections::{BTreeMap, BTreeSet}, str::FromStr};



use super::{channel::{Channel, Channels}, note_record::NoteRecord};



#[derive(Debug)]
pub struct ParseError(String);

impl ParseError {
    #[must_use]
    pub fn new(message: String) -> ParseError {
        ParseError(message)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}



pub struct Parser<'a> {
    listing: &'a str,
    channels: BTreeMap<String, Channel>,
    active_channels: BTreeSet<String>,
    current_channel: (String, Channel),
    name: String,
    bpm: Option<u16>,
}

impl<'a> Parser<'a> {
    pub fn new(listing: &'a str) -> Parser<'a> {
        Parser {
            listing,
            channels: BTreeMap::new(),
            active_channels: BTreeSet::new(),
            current_channel: (String::new(), Channel::new()),
            name: String::new(),
            bpm: None,
        }
    }

    fn append_note_line(&mut self, line: &str) -> Result<(), ParseError> {
        for note_token in line.split_whitespace() {
            let note = NoteRecord::from_str(note_token);
            match note {
                Ok(note) => self.current_channel.1.push(note),
                Err(err) => return Err(ParseError::new(format!("Unable to parse the note: {err}")))
            };
        }

        Ok(())
    }

    fn parse_meta(&mut self, name: &str, value: &str) -> Result<(), ParseError> {
        if name.len() < 2 {
            return Err(ParseError::new(format!("Invalid meta: {name}")));
        }

        // Remove the '@' prefix from the name:
        let trimmed_name = &name[1..];

        match trimmed_name {
            "bpm" => {
                let bpm = value.parse::<u16>().map_err(|_| ParseError::new(format!("Invalid BPM value: {value}")))?;
                if self.bpm.is_some() {
                    return Err(ParseError::new(String::from("BPM is already set")));
                }
                self.bpm = Some(bpm);
            },
            "channels" => {
                self.active_channels = value.split(' ').map(ToString::to_string).collect();
            },
            "name" => {
                self.name = value.to_string();
            },
            _ => {
                // Save the current channel:
                if (trimmed_name != self.current_channel.0) && !self.current_channel.1.is_empty() {
                    self.channels.insert(
                        std::mem::take(&mut self.current_channel.0),
                        std::mem::take(&mut self.current_channel.1)
                    );
                }

                self.current_channel.0 = trimmed_name.to_string();
                if let Some(channel) = self.channels.get_mut(trimmed_name) {
                    self.current_channel.1 = std::mem::take(channel);
                } else {
                    self.current_channel.1 = Channel::new();
                }

                self.append_note_line(value)?;
            }
        }

        Ok(())
    }

    pub fn parse(mut self) -> Result<Channels, ParseError> {
        for mut line in self.listing.lines() {
            if line.starts_with('#') {
                continue;
            }

            if let Some(comment_pos) = line.find(';') {
                line = &line[..comment_pos];
            }

            line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('@') {
                let mut meta_name: Option<&str> = None;
                let mut meta_value: Option<&str> = None;
                for (index, mut token) in line.splitn(2, ':').enumerate() {
                    token = token.trim();
                    if token.is_empty() {
                        return Err(ParseError::new(format!("Invalid meta: {line}")));
                    }

                    match index {
                        0 => meta_name = Some(token),
                        1 => meta_value = Some(token),
                        _ => unreachable!()
                    }
                }

                match (meta_name, meta_value) {
                    (Some(name), Some(value)) => self.parse_meta(name, value)?,
                    _ => return Err(ParseError::new(format!("Invalid meta: {line}")))
                }
            } else {
                self.append_note_line(line)?;
            }
        }
        
        // Save the current channel if exists:
        if !self.current_channel.1.is_empty() {
            self.channels.insert(
                std::mem::take(&mut self.current_channel.0),
                std::mem::take(&mut self.current_channel.1)
            );
        }

        let Some(bpm) = self.bpm else {
            return Err(ParseError::new(String::from("BPM is not set")));
        };

        let mut channels = Channels::new(bpm);
        for channel_name in self.active_channels {
            if let Some(channel) = self.channels.remove(&channel_name) {
                channels.push(channel);
            }
        }

        Ok(channels)
    }

}





#[test]
fn test() {
    use note::Note;
    use crate::synth::note_record::{NoteDivisor, NoteStyle};

    let str = r#"#!/bin/beesynth
        @name  :sample   
        @bpm: 120


        @channels   : ch1 ch2

@ch1  : !Q:E3 ~Q:E3 Q:F3
@ch2  : ~E:E4 ~E:0  H:E3
!Q:E3 ~Q:E3 Q:F3
@ch1  : !Q:E3 ~Q:E3 Q:F3
    "#;
    
    let channels = Parser::new(str).parse().unwrap();
    assert_eq!(channels.bpm(), 120);
    assert_eq!(channels.channels().len(), 2);

    let channel_list = channels.channels();

    assert_eq!(channel_list[0], vec![
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Staccato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Legato),
        NoteRecord::new(Some(Note::F(3)), NoteDivisor::Quarter, NoteStyle::NonLegato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Staccato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Legato),
        NoteRecord::new(Some(Note::F(3)), NoteDivisor::Quarter, NoteStyle::NonLegato)
    ]);

    assert_eq!(channel_list[1], vec![
        NoteRecord::new(Some(Note::E(4)), NoteDivisor::Eighth, NoteStyle::Legato),
        NoteRecord::new(None, NoteDivisor::Eighth, NoteStyle::Legato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Half, NoteStyle::NonLegato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Staccato),
        NoteRecord::new(Some(Note::E(3)), NoteDivisor::Quarter, NoteStyle::Legato),
        NoteRecord::new(Some(Note::F(3)), NoteDivisor::Quarter, NoteStyle::NonLegato)
    ]);
}