use std::{
    fs,
    env,
    path::PathBuf,
    hash::Hasher,
    fmt::Debug
};

#[derive(Debug)]
pub enum Error {
    AbsentFFmpeg,
    GetExePath(std::io::Error),
    AbsentRootFolder,
    CreateCacheFolder(std::io::Error),
    Read(std::io::Error),
    RunFFmpeg(std::io::Error),
    Conversion(String, String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AbsentFFmpeg => write!(f, "FFmpeg executable not found in ./assets/ffmpeg/ffmpeg.exe"),
            Error::GetExePath(ref err) => write!(f, "Unable to get the current executable path: {err}"),
            Error::AbsentRootFolder => write!(f, "Absent root folder"),
            Error::CreateCacheFolder(ref err) => write!(f, "Unable to create the cache folder ./assets/cache/: {err}"),
            Error::Read(ref err) => write!(f, "Unable to read the given file: {err}"),
            Error::RunFFmpeg(ref err) => write!(f, "Unable to run FFmpeg: {err}"),
            Error::Conversion(ref stdout, ref stderr) => write!(f, "Conversion failure:\nStdOut:\n{stdout}\nStdErr:\n{stderr}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::GetExePath(ref err)        |
            Error::CreateCacheFolder(ref err) |
            Error::Read(ref err)              |
            Error::RunFFmpeg(ref err) => Some(err),
            _ => None
        }
    }
}



fn calc_hash(data: &[u8]) -> u64 {
    let mut hasher = wyhash::WyHash::with_seed(0x1ee7c0de);
    hasher.write(data);
    hasher.finish()
}



pub fn convert_to_wav(
    file_path: &std::path::Path,
    bitness: u8,
    sample_rate: u32) -> Result<PathBuf, Error>
{
    let assets_folder = env::current_exe().map_err(Error::GetExePath)?
        .parent().ok_or(Error::AbsentRootFolder)?
        .join("assets");

    if !assets_folder.exists() {
        return Err(Error::AbsentFFmpeg);
    }

    let cache_folder = assets_folder.join("cache");
    if !cache_folder.exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .create(&cache_folder)
            .map_err(Error::CreateCacheFolder)?;
    }

    let file = fs::read(file_path).map_err(Error::Read)?;
    let hash = calc_hash(&file);

    let cached_path = cache_folder.join(format!("{hash}_{bitness}_{sample_rate}.wav"));
    if cached_path.exists() {
        return Ok(cached_path);
    }

    let ffmpeg_path = assets_folder.join("ffmpeg").join("ffmpeg.exe");
    if !ffmpeg_path.exists() {
        return Err(Error::AbsentFFmpeg);
    }

    let encoder_name = match bitness {
        8  => "pcm_u8",
        16 => "pcm_s16le",
        24 => "pcm_s24le",
        32 => "pcm_s32le",
        _  => return Err(Error::Conversion(
            String::default(),
            String::from("Invalid bitness, the only supported are 8, 16, 24 and 32"))
        )
    };

    let ffmpeg_process = std::process::Command::new(ffmpeg_path)
        .arg("-i").arg(file_path)
        .arg("-acodec").arg(encoder_name)
        .arg("-ac").arg("1")
        .arg("-ar").arg(sample_rate.to_string())
        .arg(&cached_path)
        .output()
        .map_err(Error::RunFFmpeg)?
        ;

    if !cached_path.exists() {
        return Err(Error::Conversion(
            String::from_utf8_lossy(&ffmpeg_process.stdout).to_string(),
            String::from_utf8_lossy(&ffmpeg_process.stderr).to_string())
        );
    }

    Ok(cached_path)
}