use windows::{
    Win32::Storage::FileSystem::{
        CreateFileW,
        FILE_ALL_ACCESS,
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ,
        FILE_SHARE_WRITE,
        OPEN_EXISTING
    },
    w
};

use winapi::{auto, scm::{self, Service}};


use crate::{
    Inpout,
    error::Error as InpoutError
};

impl crate::Inpout {
    /// # Errors
    /// 
    /// Returns an error in case of device opening failure.
    fn open_device() -> Result<Inpout, windows::core::Error> {
        #[cfg(target_arch = "x86_64")]
        let device_name = w!("\\\\.\\inpoutx64");

        #[cfg(target_arch = "x86")]
        let device_name = w!("\\\\.\\inpout");

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        compile_error!("Unsupported platform");

        let open_result = unsafe { CreateFileW(
            device_name,
            FILE_ALL_ACCESS.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None
        ) };

        open_result.map(|handle| Inpout { device_handle: auto::FileHandle::new(handle) })
    }
        
    /// # Errors
    /// 
    /// Returns an error in case of driver unpacking failure.
    fn extract_driver() -> Result<std::path::PathBuf, std::io::Error> {
        const INPOUT_OUTPUT_PATH: &str = "inpout.sys";

        let driver_path = std::env::current_dir()?
            .parent().ok_or(std::io::ErrorKind::NotFound)?
            .join(INPOUT_OUTPUT_PATH)
            ;
        
        if !driver_path.exists() {
            #[cfg(target_arch = "x86_64")] {
                let driver_bin = include_bytes!("./bin/inpoutx64.sys");
                std::fs::write(&driver_path, driver_bin)?;
            }

            #[cfg(target_arch = "x86")] {
                let driver_bin = include_bytes!("./bin/inpout32.sys");
                std::fs::write(&driver_path, driver_bin)?;
            }

            #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
            compile_error!("Unsupported platform, the only supported platforms are i386 and amd64.");
        }

        Ok(driver_path)
    }

    fn load() -> Result<Inpout, InpoutError> {
        const SERVICE_NAME: &str = "inpout";

        enum State {
            ExtractFile,
            CreateService(std::path::PathBuf /* driver path */),
            OpenService,
            StartService(Service),
            OpenDevice,
            Success(Inpout),
            DeleteService(Service),
            Failure(InpoutError)
        }

        let scm = match scm::Scm::new() {
            Ok(scm) => scm,
            Err(err) => return Err(InpoutError::OpenScm(err))
        };

        let mut state = State::OpenDevice;

        loop {
            state = match state {
                State::ExtractFile => match Self::extract_driver() {
                    Ok(driver_path) => State::CreateService(driver_path),
                    Err(err) => State::Failure(InpoutError::Unpack(err))
                },
                State::CreateService(driver_path) => if let Some(path) = driver_path.to_str() {
                    match scm.create_service(SERVICE_NAME, path) {
                        Ok(svc) => State::StartService(svc),
                        Err(err) => State::Failure(InpoutError::CreateService(err))
                    }
                } else {
                    State::Failure(InpoutError::InvalidPath)
                },
                State::OpenService => match scm.open_service(SERVICE_NAME) {
                    Ok(svc) => State::StartService(svc),
                    Err(_) => State::ExtractFile
                },
                State::StartService(mut svc) => match svc.start() {
                    Ok(()) => State::OpenDevice,
                    Err(_) => State::DeleteService(svc) // Attempt to delete and reinstall the service
                },
                State::OpenDevice => match Self::open_device() {
                    Ok(inpout) => State::Success(inpout),
                    Err(_) => State::OpenService,
                },
                State::DeleteService(mut svc) => match svc.delete() {
                    Ok(()) => State::ExtractFile,
                    Err(err) => State::Failure(InpoutError::DeleteService(err))
                }
                State::Success(inpout) => return Ok(inpout),
                State::Failure(err) => return Err(err)
            }
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of driver loading or opening failure.
    pub fn new() -> Result<crate::Inpout, InpoutError> {
        Self::load()
    }
}