#[derive(Debug)]
pub enum Error {
    OpenScm(windows::core::Error),
    Unpack(std::io::Error),
    OpenDevice(windows::core::Error),
    CreateService(windows::core::Error),
    InvalidPath,
    DeleteService(windows::core::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OpenScm(err) => write!(f, "Unable to open service manager: {err}"),
            Error::Unpack(err) => write!(f, "Unable to unpack the driver: {err}"),
            Error::OpenDevice(err) => write!(f, "Unable to open the device: {err}"),
            Error::CreateService(err) => write!(f, "Unable to create the service: {err}"),
            Error::InvalidPath => write!(f, "Invalid path."),
            Error::DeleteService(err) => write!(f, "Unable to delete the service: {err}")
        }
    }
} 

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::OpenScm(err)
            | Error::OpenDevice(err)
            | Error::CreateService(err)
            | Error::DeleteService(err) => Some(err),
            Error::Unpack(err) => Some(err),
            Error::InvalidPath => None
        }
    }
}