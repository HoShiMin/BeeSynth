use crate::wave::wav_header::WavHeader;

pub enum AudioType {
    Unknown,
    Mp3,
    Wav,
    //Xm, // Not supported yet
    Synth
}

fn is_mp3(buf: &[u8]) -> bool {
    ((buf[0] == 0x48) && (buf[1] == 0x44) && (buf[2] == 0x33)) // ID3 tag
    || ((buf[0] == 0xFF) && (buf[1] == 0xFB)) // LAME beginning
}

///
/// Check whether the file starts with the "#!/bin/beesynth".
/// 
fn is_synth(buf: &[u8]) -> bool {
    const TAG: &[u8] = b"#!/bin/beesynth";
    buf.len() >= TAG.len() && buf[..TAG.len()] == *TAG
}

impl AudioType {
    pub fn classify(buf: &[u8]) -> Self {
        if WavHeader::is_wav(buf) {
            AudioType::Wav
        // } else if XmHeader::is_xm(buf) {
        //     AudioType::Xm
        } else if is_mp3(buf) {
            AudioType::Mp3
        } else if is_synth(buf) {
            AudioType::Synth
        } else {
            AudioType::Unknown
        }
    }
}