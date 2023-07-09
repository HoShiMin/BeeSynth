#[derive(Debug, Clone)]
pub enum Error {
    ThreadOpening(windows::core::Error),
    EnumPhysRegions(windows::core::Error),
    NoPhysicalRegions,
    GetContext(windows::core::Error),
    SetContext(windows::core::Error),
    WaitFailure(winapi::sync::WaitStatus),
    EflagsNotFound
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ThreadOpening(err) => write!(f, "Failed to open thread: {err}"),
            Error::EnumPhysRegions(err) => write!(f, "Failed to enum physical regions: {err}"),
            Error::NoPhysicalRegions => write!(f, "No physical regions found"),
            Error::GetContext(err) => write!(f, "Failed to get context: {err}"),
            Error::SetContext(err) => write!(f, "Failed to set context: {err}"),
            Error::WaitFailure(status) => write!(f, "Wait failure: {status:?}"),
            Error::EflagsNotFound => write!(f, "EFlags not found in physical memory")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ThreadOpening(err)
            | Error::EnumPhysRegions(err)
            | Error::GetContext(err)
            | Error::SetContext(err)
            | Error::WaitFailure(winapi::sync::WaitStatus::Failure(err)) => Some(err),
            Error::NoPhysicalRegions
            | Error::EflagsNotFound
            | Error::WaitFailure(_) => None
        }
    }
}