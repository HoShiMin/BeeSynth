use winapi::ioctl;
use winapi::arch::{self, Arch};
use winapi::ioctl::Device;
use windows::Win32::Foundation::{HANDLE, ERROR_INVALID_PARAMETER, ERROR_UNSUPPORTED_TYPE};
use windows::core::Error;

use crate::interface::{Func, Interface, PortTrait, PhysMapping, Mapping32, Mapping64};
use crate::slicer;

impl ioctl::Device for crate::Inpout {
    fn device_handle(&self) -> HANDLE {
        self.device_handle.get()
    }
}

impl Interface for crate::Inpout {
    fn read_port<Port: PortTrait>(&self, port_number: u16) -> Result<Port::DataType, windows::core::Error> {
        #[repr(C, packed)]
        struct Input {
            port_number: u16
        }

        #[repr(C, packed)]
        #[derive(Default)]
        struct Output<DataType: Default> {
            value: DataType
        }

        let input = Input { port_number };
        let mut output = Output::<Port::DataType>::default();

        let result = self.ioctl_inout(
            Self::make_ioctl(Port::READ_FN_NUMBER),
            slicer::from(&input), 
            slicer::from_mut(&mut output)
        );
        
        match result {
            Ok(_) => Ok(output.value),
            Err(err) => Err(err)
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of failed driver request.
    fn write_port<Port: PortTrait>(&self, port_number: u16, value: Port::DataType) -> Result<(), Error> {
        #[repr(C, packed)]
        #[derive(Default)]
        struct Input<DataType: Default> {
            port_number: u16,
            value: DataType
        }

        let input = Input { port_number, value };

        let result = self.ioctl_in(
            Self::make_ioctl(Port::WRITE_FN_NUMBER), 
            slicer::from(&input)
        );
        
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    /// # Errors
    /// 
    /// Returns an error in case of failed driver request or in case of unsupported platform.
    fn map_physical_memory(&self, phys_address: u64, size: u64) -> Result<PhysMapping, Error> {
        if cfg!(target_arch = "x86") {
            let os_arch = arch::os();
            return match os_arch {
                Arch::x64 => {
                    let mut mapped = Mapping64 { section: 0, size, phys_address, mapped: 0 };
                    let result = self.ioctl_inout_inplace(Self::make_ioctl(Func::MapPhysicalMemory), slicer::from_mut(&mut mapped));
                    result.map_or_else(Err, |_| Ok(mapped.into()))
                }
                Arch::i386 => {
                    if (phys_address > u64::from(u32::MAX)) || (size > u64::from(u32::MAX)) {
                        return Err(Error::from(ERROR_INVALID_PARAMETER));
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    let mut mapped = Mapping32 { section: 0, size: size as u32, phys_address: phys_address as u32, mapped: 0 };
                    let result = self.ioctl_inout_inplace(Self::make_ioctl(Func::MapPhysicalMemory), slicer::from_mut(&mut mapped));
                    result.map_or_else(Err, |_| Ok(mapped.into()))
                }
                Arch::Unknown => Err(Error::from(ERROR_UNSUPPORTED_TYPE)) // Unsupported platform
            }
        } else if cfg!(target_arch = "x86_64") {
            let mut mapped = Mapping64 { section: 0, size, phys_address, mapped: 0 };
            let result = self.ioctl_inout_inplace(Self::make_ioctl(Func::MapPhysicalMemory), slicer::from_mut(&mut mapped));
            return result.map_or_else(Err, |_| Ok(mapped.into()));
        }
        
        Err(Error::from(ERROR_UNSUPPORTED_TYPE)) // Unsupported platform
    }

    /// # Errors
    /// 
    /// Returns an error in case of invalid mapping info, in case of failed driver request or in case of unsupported platform.
    fn unmap_physical_memory(&self, mapped: PhysMapping) -> Result<(), Error> {
        let mapped = mapped;

        if cfg!(target_arch = "x86") {
            let os_arch = arch::os();
            return match os_arch {
                Arch::i386 => {
                    if (mapped.section > u64::from(u32::MAX))
                    || (mapped.size > u64::from(u32::MAX))
                    || (mapped.phys_address > u64::from(u32::MAX))
                    || ((mapped.mapped as u64) > u64::from(u32::MAX))
                    {
                        return Err(Error::from(ERROR_INVALID_PARAMETER));
                    }

                    #[allow(clippy::cast_possible_truncation)] // Values here are always less than or equal u32::MAX
                    let mapping_info = Mapping32 {
                        section: mapped.section as u32,
                        size: mapped.size as u32,
                        phys_address: mapped.phys_address as u32,
                        mapped: mapped.mapped as usize as u32
                    };
                    let result = self.ioctl_in(Self::make_ioctl(Func::UnmapPhysicalMemory), slicer::from(&mapping_info));
                    result.map_or_else(Err, |_| Ok(()))
                }
                Arch::x64 => {
                    let mapping_info = Mapping64 {
                        section: mapped.section,
                        size: mapped.size,
                        phys_address: mapped.phys_address,
                        mapped: mapped.mapped as usize as u64
                    };
                    let result = self.ioctl_in(Self::make_ioctl(Func::UnmapPhysicalMemory), slicer::from(&mapping_info));
                    result.map_or_else(Err, |_| Ok(()))
                }
                Arch::Unknown => Err(Error::from(ERROR_UNSUPPORTED_TYPE))
            }
        } else if cfg!(target_arch = "x86_64") {
            let mapping_info = Mapping64 {
                section: mapped.section,
                size: mapped.size,
                phys_address: mapped.phys_address,
                mapped: mapped.mapped as usize as u64
            };
            let result = self.ioctl_in(Self::make_ioctl(Func::UnmapPhysicalMemory), slicer::from(&mapping_info));
            return result.map_or_else(Err, |_| Ok(()))
        }

        Err(Error::from(ERROR_UNSUPPORTED_TYPE)) // Unsupported platform
    }
}
