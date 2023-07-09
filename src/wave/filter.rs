#[derive(Default, Clone, Copy, PartialOrd, PartialEq)]
pub enum Position {
    #[default] Down,
    Up
}

pub enum Type {
    Amplitude,
    Frequency
}

pub type Nsec = u64; // Nanoseconds
pub type Ticks = u64; // TSC tick count
pub type HertzFlt = f32;
pub type HertzInt = u32;

#[derive(Default)]
pub struct WaveData {
    pub samples: Vec<f32>,
    pub sample_rate: u16
}

#[derive(Default, Clone, Copy)]
pub struct FreqRecord<HzType> {
    pub freq: HzType, // Set to zero to perform pause
    pub duration: Nsec
}
pub type FreqRecordFlt = FreqRecord<HertzFlt>;
pub type FreqRecordInt = FreqRecord<HertzInt>;

pub type FreqChannel<HzType> = Vec<FreqRecord<HzType>>;
pub type FreqData<HzType> = Vec<FreqChannel<HzType>>;


#[derive(Default, Clone, Copy)]
pub struct PositionRecord {
    pub position: Position,
    pub duration: Nsec
}

pub type PositionData = Vec<PositionRecord>;

pub enum Data {
    Amplitude(WaveData),
    Frequency(FreqData<HertzFlt>),
    Position(PositionData)
}

impl Data {
    pub fn is_empty(&self) -> bool {
        match self {
            Data::Amplitude(wave) => wave.samples.is_empty(),
            Data::Frequency(freq) => freq.is_empty(),
            Data::Position(pos) => pos.is_empty()
        }
    }
}

pub trait Filter {
    fn filter_type(&self) -> Type;
    fn filter(&self, data: Data) -> Option<Data>;
}
