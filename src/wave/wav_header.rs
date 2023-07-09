#[repr(C, packed(1))]
pub struct WavHeader {
    pub chunk_id: u32, // 'RIFF' in ASCII
    pub chunk_size: u32,
    pub format: u32, // 'WAVE'
    pub first_subchunk_header: SubchunkHeader, // 'fmt' subchunk
    pub audio_format: u16, // PCM = 1, other values mean that some compression is present
    pub num_channels: u16,
    pub sample_rate: u32, // In Hertz
    pub byte_rate: u32, // Bytes per second
    pub block_align: u16, // Byte count for one sample including all channels
    pub bits_per_sample: u16,
    // subchunk_id
    // subchunk_size
    // data
    // ...
}

impl WavHeader {
    const RIFF_SIGNATURE: u32 = 0x464_64952; // 'R', 'I', 'F', 'F'
    const WAVE_SIGNATURE: u32 = 0x455_64157; // 'W', 'A', 'V', 'E'

    pub fn is_wav(buf: &[u8]) -> bool {
        if buf.len() < std::mem::size_of::<WavHeader>() {
            return false;
        }

        let wav_header: &WavHeader = unsafe { &*(std::ptr::addr_of!(buf[0]).cast()) };
        if (wav_header.chunk_id != WavHeader::RIFF_SIGNATURE) || (wav_header.format != WavHeader::WAVE_SIGNATURE) {
            return false;
        }

        true
    }
}



#[repr(C, packed(1))]
pub struct SubchunkHeader {
    pub subchunk_id: u32,   // 'fmt' in BigEndian for the first chunk and custom id for the next chunks
    pub subchunk_size: u32, // Size of raw data not including this header
    // ... raw data with size of 'subchunk_size' field ...
} 

impl SubchunkHeader {
    pub const FMT_SIGNATURE: u32 = 0x2074_6D66;  // 'f', 'm', 't', ' '
    pub const DATA_SIGNATURE: u32 = 0x6174_6164; // 'd', 'a', 't', 'a'

    #[must_use]
    pub fn next(&self) -> *const SubchunkHeader {
        unsafe {
            std::ptr::addr_of!(*self)
                .cast::<u8>()
                .add(std::mem::size_of::<SubchunkHeader>() + self.subchunk_size as usize)
                .cast()
        }
    }

    #[must_use]
    pub fn is_fmt(&self) -> bool {
        self.subchunk_id == Self::FMT_SIGNATURE
    }

    #[must_use]
    pub fn is_data(&self) -> bool {
        self.subchunk_id == Self::DATA_SIGNATURE
    }
}



pub enum Wave<'a> {
    Unknown,
    Wave8(&'a [u8]),
    Wave16(&'a [i16]),
    Wave32(&'a [i32])
}

impl From<Wave<'_>> for Vec<i16> {
    ///
    /// Converts waves to 16-bit signed samples with a tiny amplitude loss.
    ///
    #[allow(clippy::cast_possible_truncation)]
    fn from(wave: Wave) -> Self {
        match wave {
            Wave::Wave8(data) => data.iter().map(|&sample| (i16::from(sample) - i16::from(i8::MAX)) * (i16::from(u8::MAX) + 1)).collect(),
            Wave::Wave16(data) => data.to_vec(),
            Wave::Wave32(data) => data.iter().map(|&sample| (sample / (i32::from(u16::MAX) + 1)) as i16).collect(),
            Wave::Unknown => Vec::new()
        }
    }
}

pub struct WaveView<'a> {
    header: &'a WavHeader
}

impl<'a> TryFrom<&'a [u8]> for WaveView<'a> {
    type Error = &'static str;

    fn try_from(data: &'a [u8]) -> Result<WaveView, Self::Error> {
        if !WavHeader::is_wav(data) {
            return Err("It's not a known wav-file.");
        }

        let wav_header: &WavHeader = unsafe { &*(std::ptr::addr_of!(data[0]).cast()) };

        Ok(Self { header: wav_header })
    }
}

impl<'a> WaveView<'a> {
    #[must_use]
    pub fn header(&self) -> &WavHeader {
        self.header
    }

    #[must_use]
    pub fn lookup_samples(&self) -> Wave<'a> {
        unsafe {
            let mut subchunk = std::ptr::addr_of!(self.header.first_subchunk_header);
            let end_of_file = std::ptr::addr_of!(*self.header).cast::<u8>().add(self.header.chunk_size as usize - 8);
            
            while !(*subchunk).is_data() {
                subchunk = (*subchunk).next();
                if subchunk as usize >= end_of_file as usize {
                    return Wave::Unknown;
                }
            }

            match self.header().bits_per_sample {
                8 => Wave::Wave8(std::slice::from_raw_parts(
                    subchunk.cast::<u8>().add(std::mem::size_of::<SubchunkHeader>()).cast(),
                    (*subchunk).subchunk_size as usize
                )),
                16 => Wave::Wave16(std::slice::from_raw_parts(
                    subchunk.cast::<u8>().add(std::mem::size_of::<SubchunkHeader>()).cast(),
                    (*subchunk).subchunk_size as usize / 2
                )),
                32 => Wave::Wave32(std::slice::from_raw_parts(
                    subchunk.cast::<u8>().add(std::mem::size_of::<SubchunkHeader>()).cast(),
                    (*subchunk).subchunk_size as usize / 4
                )),
                _ => Wave::Unknown
            }
        }
    }
}
