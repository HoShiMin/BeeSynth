use windows::core::Error;
use winapi::ioctl;

#[repr(C)]
pub enum Func {
    ReadPortByte = 1,
    WritePortByte = 2,
    ReadPortWord = 3,
    WritePortWord = 4,
    ReadPortDword = 5,
    WritePortDword = 6,
    MapPhysicalMemory = 7,
    UnmapPhysicalMemory = 8
}

impl From<Func> for u32 {
    fn from(val: Func) -> Self {
        val as u32
    }
}



pub trait PortTrait {
    type DataType: Default + Sized;
    const READ_FN_NUMBER: Func;
    const WRITE_FN_NUMBER: Func;
}

pub struct PortByte;
pub struct PortWord;
pub struct PortDword;

impl PortTrait for PortByte {
    type DataType = u8;
    const READ_FN_NUMBER: Func = Func::ReadPortByte;
    const WRITE_FN_NUMBER: Func = Func::WritePortByte;
}

impl PortTrait for PortWord {
    type DataType = u16;
    const READ_FN_NUMBER: Func = Func::ReadPortWord;
    const WRITE_FN_NUMBER: Func = Func::WritePortWord;
}

impl PortTrait for PortDword {
    type DataType = u32;
    const READ_FN_NUMBER: Func = Func::ReadPortDword;
    const WRITE_FN_NUMBER: Func = Func::WritePortDword;
}



// Arch-independent physical mapping representation:
pub struct PhysMapping {
    pub section: u64, // It's an arch-independent HANDLE placeholder
    pub size: u64,
    pub phys_address: u64,
    pub mapped: *mut std::ffi::c_void
}

impl Default for PhysMapping {
    fn default() -> Self {
        Self { 
            section: 0,
            size: 0,
            phys_address: 0,
            mapped: std::ptr::null_mut()
        }
    }
}

impl PhysMapping {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.section != 0
        && (self.size > 0)
        && !self.mapped.is_null()
    }

    /// # Safety
    /// 
    /// You should check validity of the mapping, otherwise you will get the `ACCESS_VIOLATION` exception.
    #[must_use]
    #[inline]
    pub unsafe fn read<T: Copy>(&self, byte_offset: usize) -> T {
        let ptr = self.mapped.cast::<u8>().add(byte_offset).cast::<T>();
        std::ptr::read_volatile(ptr)
    }

    /// # Safety
    /// 
    /// You should check validity of the mapping, otherwise you will get the `ACCESS_VIOLATION` exception.
    #[inline]
    pub unsafe fn write<T: Copy>(&self, byte_offset: usize, value: T) {
        let ptr = self.mapped.cast::<u8>().add(byte_offset).cast::<T>();
        std::ptr::write_volatile(ptr, value);
    }
}

// Physical mapping introduced by 64-bit Inpout driver:
#[repr(C, packed)]
#[derive(Default)]
pub struct Mapping64 {
    pub section: u64,
    pub size: u64,
    pub phys_address: u64,
    pub mapped: u64
}

impl From<Mapping64> for PhysMapping {
    fn from(val: Mapping64) -> Self {
        PhysMapping {
            section: val.section,
            size: val.size,
            phys_address: val.phys_address,
            mapped: val.mapped as *mut std::ffi::c_void
        }
    }
}

// Physical mapping introduced by 32-bit Inpout driver:
#[repr(C, packed)]
#[derive(Default)]
pub struct Mapping32 {
    pub section: u32,
    pub size: u32,
    pub phys_address: u32,
    pub mapped: u32
}

impl From<Mapping32> for PhysMapping {
    fn from(val: Mapping32) -> Self {
        PhysMapping {
            section: u64::from(val.section),
            size: u64::from(val.size),
            phys_address: u64::from(val.phys_address),
            mapped: val.mapped as *mut std::ffi::c_void
        }
    }
}



pub trait Interface {
    #[must_use]
    fn make_ioctl(func: Func) -> ioctl::Ioctl {
        const INPOUT_DEVICE_TYPE: u32 = 40_000; // Just a magic value instead of DEVICE_TYPE_UNKNOWN introduced by the inpout developer.
        ioctl::Ioctl::new(ioctl::Method::Buffered, ioctl::DeviceAccess::Any, INPOUT_DEVICE_TYPE, func.into())
    }

    /// # Errors
    /// 
    /// Returns an error in case of driver request failure.
    fn read_port<Port: PortTrait>(&self, port_number: u16) -> Result<Port::DataType, Error>;
    
    /// # Errors
    /// 
    /// Returns an error in case of driver request failure.
    fn write_port<Port: PortTrait>(&self, port_number: u16, value: Port::DataType) -> Result<(), Error>;

    /// # Errors
    /// 
    /// Returns an error in case of driver request failure or if the requsted physical memory is inaccessible.
    fn map_physical_memory(&self, phys_address: u64, size: u64) -> Result<PhysMapping, Error>;
    
    /// # Errors
    /// 
    /// Returns an error in case of driver request failure or if the mapping info is invalid.
    fn unmap_physical_memory(&self, mapped: PhysMapping) -> Result<(), Error>;
}
