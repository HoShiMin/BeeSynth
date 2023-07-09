use windows::{
    Win32::System::Services::{
        OpenSCManagerW,
        SC_MANAGER_CREATE_SERVICE,
        CreateServiceW,
        SERVICE_ALL_ACCESS,
        SERVICE_KERNEL_DRIVER,
        SERVICE_DEMAND_START,
        SERVICE_ERROR_NORMAL,
        StartServiceW,
        ControlService,
        SERVICE_CONTROL_STOP,
        SERVICE_STATUS,
        OpenServiceW,
        DeleteService,
        SERVICE_STOPPED,
        SERVICE_START_PENDING,
        SERVICE_STOP_PENDING,
        SERVICE_RUNNING,
        SERVICE_CONTINUE_PENDING,
        SERVICE_PAUSE_PENDING,
        SERVICE_PAUSED,
        SERVICE_STATUS_CURRENT_STATE,
        QueryServiceStatus
    },
    core::{ PCWSTR, Error }
};

use crate::auto;

pub struct Scm {
    scm_handle: auto::ServiceHandle
}

impl Scm {
    /// # Errors
    /// 
    /// Returns an error in case of `OpenSCManagerW` failure.
    pub fn new() -> Result<Scm, Error> {
        let scm_handle = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CREATE_SERVICE) };
        scm_handle.map_or_else(
            |_| Err(Error::from_win32()),
            |handle| Ok(Scm { scm_handle: auto::ServiceHandle::new(handle) })
        )
    }

    /// # Errors
    /// 
    /// Returns an error in case of `CreateServiceW` failure.
    pub fn create_service(&self, name: &str, bin_path: &str) -> Result<Service, Error> {
        let name_utf16: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let bin_path_utf16: Vec<u16> = bin_path.encode_utf16().chain(std::iter::once(0)).collect();
        
        let service_handle = unsafe { CreateServiceW(
            self.scm_handle.get(),
            PCWSTR::from_raw(name_utf16.as_ptr()),
            PCWSTR::from_raw(name_utf16.as_ptr()),
            SERVICE_ALL_ACCESS,
            SERVICE_KERNEL_DRIVER,
            SERVICE_DEMAND_START,
            SERVICE_ERROR_NORMAL,
            PCWSTR::from_raw(bin_path_utf16.as_ptr()),
            PCWSTR::null(),
            None,
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null()
        ) };

        service_handle.map_or_else(
            |_| Err(Error::from_win32()),
            |handle| Ok(Service { service_handle: auto::ServiceHandle::new(handle) })
        )
    }

    /// # Errors
    /// 
    /// Returns an error in case of `OpenServiceW` failure.
    pub fn open_service(&self, name: &str) -> Result<Service, Error> {
        let name_utf16: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let service_handle = unsafe { OpenServiceW(self.scm_handle.get(), PCWSTR::from_raw(name_utf16.as_ptr()), SERVICE_ALL_ACCESS) };
        service_handle.map_or_else(
            |_| Err(Error::from_win32()),
            |handle| Ok(Service { service_handle: auto::ServiceHandle::new(handle) })
        )
    }
}


pub enum ServiceStatus {
    Unknown = 0,
    Stopped = 1,
    StartPending = 2,
    StopPending = 3,
    Running = 4,
    ContinuePending = 5,
    PausePending = 6,
    Paused = 7
}

impl ServiceStatus {
    #[must_use]
    pub fn from_raw_status(status: SERVICE_STATUS_CURRENT_STATE) -> ServiceStatus {
        match status {
            SERVICE_STOPPED => ServiceStatus::Stopped,
            SERVICE_START_PENDING => ServiceStatus::StartPending,
            SERVICE_STOP_PENDING => ServiceStatus::StopPending,
            SERVICE_RUNNING => ServiceStatus::Running,
            SERVICE_CONTINUE_PENDING => ServiceStatus::ContinuePending,
            SERVICE_PAUSE_PENDING => ServiceStatus::PausePending,
            SERVICE_PAUSED => ServiceStatus::Paused,
            _ => ServiceStatus::Unknown            
        }
    }
}

impl ToString for ServiceStatus {
    fn to_string(&self) -> String {
        match *self {
            ServiceStatus::Unknown => String::from("UNKNOWN"),
            ServiceStatus::Stopped => String::from("STOPPED"),
            ServiceStatus::StartPending => String::from("START_PENDING"),
            ServiceStatus::StopPending => String::from("STOP_PENDING"),
            ServiceStatus::Running => String::from("RUNNING"),
            ServiceStatus::ContinuePending => String::from("CONTINUE_PENDING"),
            ServiceStatus::PausePending => String::from("PAUSE_PENDING"),
            ServiceStatus::Paused => String::from("PAUSED"),
        }
    }
}

pub struct Service {
    service_handle: auto::ServiceHandle
}

impl Service {
    /// # Errors
    ///
    /// Returns an error in case of `StartServiceW` failure.
    pub fn start(&mut self) -> Result<(), Error> {
        let status = unsafe { StartServiceW(self.service_handle.get(), None) };
        if status.as_bool() {
            Ok(())
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `ControlService` failure.
    pub fn stop(&mut self) -> Result<(), Error> {
        let status = unsafe {
            let mut service_status = SERVICE_STATUS::default();
            ControlService(
                self.service_handle.get(),
                SERVICE_CONTROL_STOP,
                std::ptr::addr_of_mut!(service_status)
            )
        };

        if status.as_bool() {
            Ok(())
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `QueryServiceStatus` failure.
    pub fn query_status(&self) -> Result<ServiceStatus, Error> {
        let mut status_info = SERVICE_STATUS::default();
        let query_status = unsafe { QueryServiceStatus(self.service_handle.get(), std::ptr::addr_of_mut!(status_info)) };
        if query_status.as_bool() {
            Ok(ServiceStatus::from_raw_status(status_info.dwCurrentState))
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `DeleteService` failure.
    pub fn delete(&mut self) -> Result<(), Error> {
        let status = unsafe { DeleteService(self.service_handle.get()) };
        if status.as_bool() {
            Ok(())
        } else {
            Err(Error::from_win32())
        }
    }
}
