use proc_bitfield::bitfield;

use windows::{
    Win32::{
        System::{
            Ioctl as WinIoctl,
            IO::DeviceIoControl
        },
        Foundation::HANDLE
    },
    core::Error
};

pub trait Device {
    fn device_handle(&self) -> HANDLE;

    /// # Errors
    /// 
    /// Returns an error in case of `DeviceIoControl` failure.
    fn ioctl_inout_inplace(&self, ctl: Ioctl, inout: &mut [u8]) -> Result<u32, Error> {
        let mut returned = 0;
        let status = unsafe {
            #[allow(clippy::cast_possible_truncation)]
            DeviceIoControl(
                self.device_handle(),
                ctl.raw(),
                Some(inout.as_ptr().cast()),
                inout.len() as u32,
                Some(inout.as_mut_ptr().cast()),
                inout.len() as u32,
                Some(std::ptr::addr_of_mut!(returned)),
                None
            )
        };

        if status.as_bool() {
            Ok(returned)
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `DeviceIoControl` failure.
    fn ioctl_inout(&self, ctl: Ioctl, input: &[u8], output: &mut [u8]) -> Result<u32, Error> {
        let mut returned = 0;
        let status = unsafe {
            #[allow(clippy::cast_possible_truncation)]
            DeviceIoControl(
                self.device_handle(),
                ctl.raw(),
                Some(input.as_ptr().cast()),
                input.len() as u32,
                Some(output.as_mut_ptr().cast()),
                output.len() as u32,
                Some(std::ptr::addr_of_mut!(returned)),
                None
            )
        };

        if status.as_bool() {
            Ok(returned)
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `DeviceIoControl` failure.
    fn ioctl_in(&self, ctl: Ioctl, input: &[u8]) -> Result<u32, Error> {
        let mut returned = 0;
        let status = unsafe {
            #[allow(clippy::cast_possible_truncation)]
            DeviceIoControl(
                self.device_handle(),
                ctl.raw(),
                Some(input.as_ptr().cast()),
                input.len() as u32,
                None,
                0,
                Some(std::ptr::addr_of_mut!(returned)),
                None
            )
        };

        if status.as_bool() {
            Ok(returned)
        } else {
            Err(Error::from_win32())
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of `DeviceIoControl` failure.
    fn ioctl_out(&self, ctl: Ioctl, input: &[u8]) -> Result<u32, Error> {
        let mut returned = 0;
        let status = unsafe {
            #[allow(clippy::cast_possible_truncation)]
            DeviceIoControl(
                self.device_handle(),
                ctl.raw(),
                Some(input.as_ptr().cast()),
                input.len() as u32,
                None,
                0,
                Some(std::ptr::addr_of_mut!(returned)),
                None
            )
        };

        if status.as_bool() {
            Ok(returned)
        } else {
            Err(Error::from_win32())
        }
    }
}

#[repr(u32)]
pub enum Method {
    Buffered = WinIoctl::METHOD_BUFFERED,
    InDirect = WinIoctl::METHOD_IN_DIRECT,
    OutDirect = WinIoctl::METHOD_OUT_DIRECT,
    Neither = WinIoctl::METHOD_NEITHER
}

#[repr(u32)]
pub enum DeviceAccess {
    Any = WinIoctl::FILE_ANY_ACCESS,
    Read = WinIoctl::FILE_READ_ACCESS,
    Write = WinIoctl::FILE_WRITE_ACCESS
}

bitfield! {
    #[repr(C, packed)]
    #[derive(Default)]
    pub struct Ioctl(u32) {
        transfer_type: u32 @ 0..=1, // IoctlMethod
        function_code: u32 @ 2..=12,
        custom: bool @ 13,
        required_access: u32 @ 14..=15,
        device_type: u32 @ 16..=30,
        common: bool @ 31
    }
}

impl Ioctl {
    #[must_use]
    pub fn new(method: Method, access: DeviceAccess, device_type: u32, function_code: u32) -> Ioctl {
        /*
            1 001110001000000 00 1 00000000001 00
            ^ ^               ^  ^ ^           ^- Transfer Type (METHOD_***)
            | |               |  | +- Function code (1, 2, 3, ...)
            | |               |  +- Custom
            | |               +- Required Access
            | +- Device Type (FILE_DEVICE_***)
            +- Common
        */
        Ioctl::default()
            .with_transfer_type(method as u32)
            .with_function_code(function_code)
            .with_custom(true)
            .with_required_access(access as u32)
            .with_device_type(device_type)
            .with_common(true)
    }

    #[must_use]
    pub fn make_buffered(function_code: u32) -> Ioctl {
        Self::new(Method::Buffered, DeviceAccess::Any, WinIoctl::FILE_DEVICE_UNKNOWN, function_code)
    }

    #[must_use]
    pub fn make_in_direct(function_code: u32) -> Ioctl {
        Self::new(Method::InDirect, DeviceAccess::Any, WinIoctl::FILE_DEVICE_UNKNOWN, function_code)
    }

    #[must_use]
    pub fn make_out_direct(function_code: u32) -> Ioctl {
        Self::new(Method::OutDirect, DeviceAccess::Any, WinIoctl::FILE_DEVICE_UNKNOWN, function_code)
    }

    #[must_use]
    pub fn make_neither(function_code: u32) -> Ioctl {
        Self::new(Method::Neither, DeviceAccess::Any, WinIoctl::FILE_DEVICE_UNKNOWN, function_code)
    }

    #[must_use]
    pub fn raw(&self) -> u32 {
        self.0
    }
}
