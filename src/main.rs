#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]


use std::sync::atomic::{AtomicBool, Ordering};

use audio_classifier::AudioType;
use beeper::{generic::Beeper, sound_emitter::SoundEmitter, port_accessor::PortAccessor, iopl_based::BeeperIopl};
use inpout::{Inpout, Interface, interface::PortByte};

use nano_sleep::NanoSleep;
use wave::{filter::{PositionRecord, FreqRecordFlt}, wav_header::WaveView};
use winapi::sched;

use crate::wave::filter::{self, HertzInt, Filter};

mod synth;
mod wave;
mod converter;
mod audio_classifier;
mod help;



static mut STOP_MACHINE: AtomicBool = AtomicBool::new(false);



struct InpoutDriver(Inpout);

impl InpoutDriver {
    pub fn new() -> Result<Self, inpout::error::Error> {
        Ok(Self(Inpout::new()?))
    }
}

impl PortAccessor for InpoutDriver {
    fn read_byte(&self, port: u16) -> Option<u8> {
        self.0.read_port::<PortByte>(port).ok()
    }

    fn write_byte(&self, port: u16, value: u8) -> bool {
        self.0.write_port::<PortByte>(port, value).is_ok()
    }
}

struct PhysMapping<'a>(inpout::interface::PhysMapping, &'a InpoutDriver);

impl<'a> iopl::windows::phys_mapper::Mapping for PhysMapping<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn size(&self) -> usize {
        self.0.size as usize
    }

    fn mapping(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.mapped.cast(), self.size()) }
    }

    fn mapping_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.0.mapped.cast(), self.size()) }
    }

    fn unmap(self) {
        let _unmap_result = self.1.0.unmap_physical_memory(self.0);
    }
}

impl<'a> iopl::windows::phys_mapper::Mapper<'a> for InpoutDriver {
    type Type = PhysMapping<'a>;

    fn map(&'a self, phys_addr: u64, size: u64) -> Option<Self::Type> {
        let mapping = self.0.map_physical_memory(phys_addr, size);
        match mapping {
            Ok(mapping) => Some(PhysMapping(mapping, self)),
            Err(_) => None
        }
    }
}

struct AmplitudePeeker<'a, Iter>
    where Iter: Iterator<Item = &'a PositionRecord>
{
    iter: Iter
}

impl<'a, Iter> AmplitudePeeker<'a, Iter>
    where Iter: Iterator<Item = &'a PositionRecord>
{
    pub fn new(iter: Iter) -> Self {
        Self { iter }
    }
}

impl<'a, Iter> wave::player::amplitudes::Peeker for AmplitudePeeker<'a, Iter>
    where Iter: Iterator<Item = &'a PositionRecord>
{
    fn peek(&mut self) -> Option<PositionRecord> {
        if unsafe { STOP_MACHINE.load(Ordering::Relaxed) } {
            return None;
        }

        self.iter.next().copied()
    }
}


struct FrequencyPeeker<'a, Iter>
    where Iter: Iterator<Item = &'a FreqRecordFlt>
{
    channels: Vec<Iter>
}

impl<'a, Iter> FrequencyPeeker<'a, Iter>
    where Iter: Iterator<Item = &'a FreqRecordFlt>
{
    pub fn new() -> Self {
        Self { channels: Vec::new() }
    }

    pub fn add(&mut self, iter: Iter) {
        self.channels.push(iter);
    }
}

impl<'a, Iter> wave::player::frequencies::Peeker for FrequencyPeeker<'a, Iter>
    where Iter: Iterator<Item = &'a FreqRecordFlt>
{
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn peek(&mut self, channel_number: usize) -> Option<filter::FreqRecord<HertzInt>> {
        if unsafe { STOP_MACHINE.load(Ordering::Relaxed) } {
            return None;
        }

        self.channels
            .get_mut(channel_number)
            .and_then(|channel| channel.next().map(|record|
                filter::FreqRecordInt {
                    freq: record.freq as HertzInt,
                    duration: record.duration
                }
            ))
    }

    fn channel_count(&self) -> usize {
        self.channels.len()
    }
}


enum BeeperType {
    Ioctl,
    Iopl
}

struct PlayParams {
    beeper_type: BeeperType,
    switch_interval: u64,
    filters: Vec::<Box<dyn wave::filter::Filter>>
}

enum BeeperHolder<'a> {
    Ioctl(Beeper<'a, InpoutDriver>),
    Iopl(BeeperIopl)
}

fn play_data(mut samples: filter::Data, play_params: &PlayParams) -> Result<(), ()> {
    for filter in &play_params.filters {
        samples = if let Some(filtered) = filter.filter(samples) {
            filtered
        } else {
            eprintln!("Mismatched filter type and the filtered data.");
            return Err(());
        }
    }

    if let filter::Data::Amplitude(_) = &samples {
        let bakery = wave::bakery::Bakery::new(wave::bakery::Strategy::Differential(5));
        if let Some(baked) = bakery.filter(samples) {
            samples = baked;
        } else {
            eprintln!("Unable to bake samples.");
            return Err(());
        }
    }

    let inpout = match InpoutDriver::new() {
        Ok(inpout) => inpout,
        Err(err) => {
            eprintln!("Unable to initialize Inpout: {err}");
            return Err(());
        }
    };

    let mut beeper_holder = match play_params.beeper_type {
        BeeperType::Ioctl => BeeperHolder::Ioctl(Beeper::new(&inpout)),
        BeeperType::Iopl => {
            let patch_status = iopl::windows::Patcher::new(&inpout).patch(iopl::level::Level::Ring3);
            match patch_status {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Unable to patch iopl: {err}");
                    return Err(());
                }
            }
            BeeperHolder::Iopl(BeeperIopl::new())
        }
    };
    
    let waiter = NanoSleep::new(1000);

    match samples {
        filter::Data::Amplitude(_) => unreachable!(),
        filter::Data::Frequency(frequencies) => {
            let mut freq_peeker = FrequencyPeeker::new();
            for channel in &frequencies {
                freq_peeker.add(channel.iter());
            }

            match beeper_holder {
                BeeperHolder::Ioctl(ref mut beeper) => {
                    beeper.prepare();
                    wave::player::frequencies::play(beeper, &mut freq_peeker, &waiter, play_params.switch_interval);
                }
                BeeperHolder::Iopl(ref mut beeper) => {
                    beeper.prepare();
                    wave::player::frequencies::play(beeper, &mut freq_peeker, &waiter, play_params.switch_interval);
                }
            }
        },
        filter::Data::Position(positions) => {
            match beeper_holder {
                BeeperHolder::Ioctl(ref mut beeper) => {
                    beeper.prepare();
                    wave::player::amplitudes::play(beeper, &mut AmplitudePeeker::new(positions.iter()), &waiter);
                }
                BeeperHolder::Iopl(ref mut beeper) => {
                    beeper.prepare();
                    wave::player::amplitudes::play(beeper, &mut AmplitudePeeker::new(positions.iter()), &waiter);
                }
            }
        }
    }

    println!("Finished");
    Ok(())
}

fn play_wav(wav: WaveView, play_params: &PlayParams) -> Result<(), ()> {
    let samples: filter::Data = wav.into();
    if samples.is_empty() {
        eprintln!("There are no data to play.");
        return Err(());
    }

    play_data(samples, play_params)
}


fn parse_synth_params(params: Vec<Param>) -> Result<PlayParams, String> {
    let mut play_params = PlayParams {
        beeper_type: BeeperType::Ioctl,
        switch_interval: 20 * 1000 * 1000,
        filters: Vec::new()
    };

    for param in params {
        match param {
            Param::Iopl => play_params.beeper_type = BeeperType::Iopl,
            Param::SwitchInterval(interval) => play_params.switch_interval = interval,
            _ => return Err(format!("Inapplicable param {param:?}"))
        }
    }

    Ok(play_params)
}

fn parse_wave_params(params: Vec<Param>, sample_rate: u32) -> PlayParams {
    let mut play_params = PlayParams {
        beeper_type: BeeperType::Ioctl,
        switch_interval: 20 * 1000 * 1000,
        filters: Vec::new()
    };

    #[allow(clippy::cast_precision_loss)]
    for param in params {
        match param {
            Param::Iopl => play_params.beeper_type = BeeperType::Iopl,
            Param::SwitchInterval(interval) => play_params.switch_interval = interval,
            Param::LowPass(highest_freq) => play_params.filters.push(
                Box::new(wave::freq_filters::LowPass::new(sample_rate, highest_freq as f32))
            ),
            Param::HighPass(lowest_freq) => play_params.filters.push(
                Box::new(wave::freq_filters::LowPass::new(sample_rate, lowest_freq as f32))
            ),
            Param::BakeSimple => play_params.filters.push(
                Box::new(wave::bakery::Bakery::new(wave::bakery::Strategy::Simple))
            ),
            Param::BakeDifferential(percentage) => play_params.filters.push(
                Box::new(wave::bakery::Bakery::new(wave::bakery::Strategy::Differential(percentage)))
            ),
            Param::ExtractFreq(min, max, sampling_size, step_by, channel_count) => play_params.filters.push(Box::new(
                wave::freq_extractor::FreqExtractor::new(
                    min,
                    max,
                    sampling_size.unwrap_or(4096),
                    step_by.unwrap_or(32),
                    sample_rate, 
                    channel_count.unwrap_or(2)
                )
            )),
            Param::NoteMatcher => play_params.filters.push(Box::new(wave::note_matcher::NoteMatcher))
        }
    }

    play_params
}

fn play_generic(path: &std::path::Path, params: Vec<Param>) -> Result<(), ()> {
    sched::set_affinity(sched::Affinity::Exact(sched::get_cpu_count() - 1));
    sched::set_process_priority(sched::Priority::Realtime);
    sched::set_thread_priority(sched::Priority::Realtime);
    
    let mut data = match std::fs::read(path) {
        Ok(buf) => buf,
        Err(err) => {
            eprintln!("Unable to read the given file {}: {err}", path.to_str().unwrap_or("<???>"));
            return Err(());
        }
    };

    let audio_type = AudioType::classify(&data);
    if let AudioType::Synth = audio_type {
        let listing = unsafe { std::str::from_utf8_unchecked(&data) };
        let channels = match synth::parser::Parser::new(listing).parse() {
            Ok(channels) => channels,
            Err(err) => {
                eprintln!("Unable to parse the given synth-file: {err}");
                return Err(());
            }
        };

        let play_params = match parse_synth_params(params) {
            Ok(params) => params,
            Err(err) => {
                eprintln!("{err}");
                return Err(());
            }
        };

        play_data(channels.into(), &play_params)
    } else {
        let wav_header = match audio_type {
            AudioType::Wav => match wave::wav_header::WaveView::try_from(data.as_slice()) {
                Ok(header) => header,
                Err(err) => {
                    eprintln!("Unable to parse the given wav-file: {err}");
                    return Err(());
                }
            },
            AudioType::Mp3 | AudioType::Unknown => {
                let converted_file = converter::convert_to_wav(path, 16, 22050);
                match converted_file {
                    Ok(path) => match std::fs::read(&path) {
                        Ok(buf) => {
                            data = buf;
                            match wave::wav_header::WaveView::try_from(data.as_slice()) {
                                Ok(header) => header,
                                Err(err) => {
                                    eprintln!("Unable to parse the given wav-file: {err}");
                                    return Err(());
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Unable to read the converted file {}: {err}", path.to_str().unwrap_or("<???>"));
                            return Err(());
                        }
                    }
                    Err(err) => {
                        eprintln!("Unable to convert the given file to WAV: {err}");
                        return Err(());
                    }
                }
            },
            AudioType::Synth => unreachable!() // Handled above
        };

        let play_params = parse_wave_params(params, wav_header.header().sample_rate);
        play_wav(wav_header, &play_params)
    }
}

fn mute() -> Result<(), ()> {
    let inpout = match InpoutDriver::new() {
        Ok(inpout) => inpout,
        Err(err) => {
            eprintln!("Unable to initialize Inpout: {err}");
            return Err(());
        }
    };

    let mut beeper = Beeper::new(&inpout);
    beeper.prepare();
    beeper.mute();

    Ok(())
}

#[derive(Debug)]
enum Param {
    Iopl,                                  // --iopl
    SwitchInterval(u64 /* Msec */),        // --switch-interval=msec
    LowPass(u32 /* Hz */),                 // --low-pass=hz
    HighPass(u32 /* Hz */),                // --high-pass=hz
    BakeSimple,                            // --bake-simple
    BakeDifferential(u8 /* Percentage */), // --bake-diff=percentage
    ExtractFreq(                           // --extract-freq=[...]
        Option<u32> /* Min Hz */,              // min=N
        Option<u32> /* Max Hz */,              // max=N
        Option<u32> /* Sampling size */,       // sampling=N
        Option<u32> /* Step by */,             // step=N
        Option<u8>  /* Number of channels */,  // channels=N
    ),
    NoteMatcher                            // --note-matcher
}

#[derive(Debug)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}



fn parse_params(params: &[String]) -> Result<(std::path::PathBuf, Vec<Param>), ParseError> {
    let mut path = std::path::PathBuf::new();
    let mut result = Vec::<Param>::new();
    for param in params {
        if param.starts_with("--") {
            let mut parts = param.splitn(2, '=');
            let name = parts.next().unwrap();
            let value = parts.next();
            match name {
                "--iopl" => {
                    result.push(Param::Iopl);
                }
                "--switch-interval" => {
                    let value = value.ok_or(ParseError(format!("Missing value for {name}")))?;
                    let value = value.parse::<u64>().map_err(|err| ParseError(format!("Unable to parse {value} as u64: {err}")))?;
                    result.push(Param::SwitchInterval(value));
                }
                "--low-pass" => {
                    let value = value.ok_or(ParseError(format!("Missing value for {name}")))?;
                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                    result.push(Param::LowPass(value));
                }
                "--high-pass" => {
                    let value = value.ok_or(ParseError(format!("Missing value for {name}")))?;
                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                    result.push(Param::HighPass(value));
                }
                "--bake-simple" => {
                    result.push(Param::BakeSimple);
                }
                "--bake-diff" => {
                    let value = value.ok_or(ParseError(format!("Missing value for {name}")))?;
                    let value = value.parse::<u8>().map_err(|err| ParseError(format!("Unable to parse {value} as u8: {err}")))?;
                    result.push(Param::BakeDifferential(value));
                }
                "--note-matcher" => {
                    result.push(Param::NoteMatcher);
                }
                "--extract-freq" => {
                    let mut min = None;
                    let mut max = None;
                    let mut sampling = None;
                    let mut step = None;
                    let mut channels = None;
                    if let Some(value) = value {
                        let parts = value.split(',');
                        for part in parts {
                            let mut subparts = part.split('=');
                            let name = subparts.next().unwrap();
                            let value = subparts.next().ok_or(ParseError(format!("Missing value for {name}")))?;
                            match name {
                                "min" => {
                                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                                    min = Some(value);
                                }
                                "max" => {
                                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                                    max = Some(value);
                                }
                                "sampling" => {
                                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                                    sampling = Some(value);
                                }
                                "step" => {
                                    let value = value.parse::<u32>().map_err(|err| ParseError(format!("Unable to parse {value} as u32: {err}")))?;
                                    step = Some(value);
                                }
                                "channels" => {
                                    let value = value.parse::<u8>().map_err(|err| ParseError(format!("Unable to parse {value} as u8: {err}")))?;
                                    channels = Some(value);
                                }
                                _ => {
                                    return Err(ParseError(format!("Unknown parameter {name}")));
                                }
                            }
                        }
                    }
                    result.push(Param::ExtractFreq(min, max, sampling, step, channels));
                }
                "--help" => {
                    help::print_help();
                    std::process::exit(0);
                }
                _ => {
                    return Err(ParseError(format!("Unknown parameter {name}")));
                }
            }
        } else {
            path = std::path::PathBuf::from(param);
        }
    }

    Ok((path, result))
}

fn main() -> Result<(), ()> {
    ctrlc::set_handler(move || {
        unsafe { STOP_MACHINE.store(true, Ordering::Relaxed) };
        let _result = mute();
        std::process::exit(0);
    }).expect("Unable to set the Ctrl-C handler");

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        return mute();
    }

    let (path, filter_params) = parse_params(&args[1..]).map_err(|err| {
        eprintln!("{err}");
    })?;

    play_generic(&path, filter_params)?;

    Ok(())
}
