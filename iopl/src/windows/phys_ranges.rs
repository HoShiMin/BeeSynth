use windows::Win32::System::Registry::{ RegOpenKeyW, RegQueryValueExW, HKEY, REG_VALUE_TYPE, HKEY_LOCAL_MACHINE };
use windows::w;


#[allow(clippy::struct_excessive_bools)]
pub struct PhysicalRegion {
    pub beginning: u64,
    pub size: u64,
    pub flags: u16,
    pub readwrite: bool,
    pub readonly: bool,
    pub writeonly: bool,
    pub prefetchable: bool,
    pub cacheable: bool,
    pub write_combined: bool
}


#[repr(C, packed(4))]
#[allow(non_camel_case_types)]
struct Memory {
    start: u64,
    size: u64
}

#[repr(u8)]
#[allow(dead_code, non_camel_case_types)]
#[allow(clippy::enum_variant_names)]
enum CM_RESOURCE_TYPE {
    CmResourceTypeNull               = 0,   // ResType_All or ResType_None (0x0000)
    CmResourceTypePort               = 1,   // ResType_IO (0x0002)
    CmResourceTypeInterrupt          = 2,   // ResType_IRQ (0x0004)
    CmResourceTypeMemory             = 3,   // ResType_Mem (0x0001)
    CmResourceTypeDma                = 4,   // ResType_DMA (0x0003)
    CmResourceTypeDeviceSpecific     = 5,   // ResType_ClassSpecific (0xFFFF)
    CmResourceTypeBusNumber          = 6,   // ResType_BusNumber (0x0006)
    CmResourceTypeMemoryLarge        = 7,   // ResType_MemLarge (0x0007)

    //CmResourceTypeNonArbitrated    = 128,   // Not arbitrated if 0x80 bit set
    CmResourceTypeConfigData       = 128,   // ResType_Reserved (0x8000)
    CmResourceTypeDevicePrivate    = 129,   // ResType_DevicePrivate (0x8001)
    CmResourceTypePcCardConfig     = 130,   // ResType_PcCardConfig (0x8002)
    CmResourceTypeMfCardConfig     = 131,   // ResType_MfCardConfig (0x8003)
    CmResourceTypeConnection       = 132    // ResType_Connection (0x8004)
}

#[repr(C, packed(4))]
#[allow(non_camel_case_types)]
struct CM_PARTIAL_RESOURCE_DESCRIPTOR {
    type_: CM_RESOURCE_TYPE,
    share_disposition: u8,
    flags: u16,
    data: Memory
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct CM_PARTIAL_RESOURCE_LIST {
    version: u16,
    revision: u16,
    count: u32,
    partial_descriptors: [CM_PARTIAL_RESOURCE_DESCRIPTOR; 1]
}

#[repr(u32)]
#[allow(non_camel_case_types, dead_code)]
enum INTERFACE_TYPE {
    InterfaceTypeUndefined,
    Internal,
    Isa,
    Eisa,
    MicroChannel,
    TurboChannel,
    PCIBus,
    VMEBus,
    NuBus,
    PCMCIABus,
    CBus,
    MPIBus,
    MPSABus,
    ProcessorInternal,
    InternalPowerBus,
    PNPISABus,
    PNPBus,
    Vmcs,
    ACPIBus,
    MaximumInterfaceType
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct CM_FULL_RESOURCE_DESCRIPTOR {
    interface_type: INTERFACE_TYPE,
    bus_number: u32,
    partial_resource_list: CM_PARTIAL_RESOURCE_LIST
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct CM_RESOURCE_LIST {
    count: u32,
    list: [CM_FULL_RESOURCE_DESCRIPTOR; 1]
}

const CM_RESOURCE_MEMORY_READ_WRITE    : u16 = 0x0000;
const CM_RESOURCE_MEMORY_READ_ONLY     : u16 = 0x0001;
const CM_RESOURCE_MEMORY_WRITE_ONLY    : u16 = 0x0002;
const CM_RESOURCE_MEMORY_PREFETCHABLE  : u16 = 0x0004;
const CM_RESOURCE_MEMORY_COMBINEDWRITE : u16 = 0x0008;
const CM_RESOURCE_MEMORY_CACHEABLE     : u16 = 0x0020;

const CM_RESOURCE_MEMORY_LARGE_40 : u16 = 0x0200;
const CM_RESOURCE_MEMORY_LARGE_48 : u16 = 0x0400;
const CM_RESOURCE_MEMORY_LARGE_64 : u16 = 0x0800;


// "HKEY_LOCAL_MACHINE\HARDWARE\RESOURCEMAP\System Resources\Physical Memory"
// https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_cm_resource_list
pub fn get_physical_memory_ranges() -> Result<Vec<PhysicalRegion>, windows::core::Error> {
    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyW(
            HKEY_LOCAL_MACHINE,
            w!("HARDWARE\\RESOURCEMAP\\System Resources\\Physical Memory"),
            &mut key
        ).ok()?;
        let key_holder = winapi::auto::RegKey::new(key);

        let mut value_type = REG_VALUE_TYPE::default();
        let mut value_size: u32 = 0;
        RegQueryValueExW(
            key_holder.get(),
            w!(".Translated"),
            None,
            Some(std::ptr::addr_of_mut!(value_type)),
            None,
            Some(std::ptr::addr_of_mut!(value_size))
        ).ok()?;

        let mut buf = vec![0_u8; value_size as usize];
        RegQueryValueExW(
            key_holder.get(),
            w!(".Translated"),
            None,
            Some(std::ptr::addr_of_mut!(value_type)),
            Some(buf.as_mut_ptr()),
            Some(std::ptr::addr_of_mut!(value_size))
        ).ok()?;

        let mut result_list = Vec::<PhysicalRegion>::default();

        #[allow(clippy::cast_ptr_alignment)]
        let resource_list = buf.as_ptr().cast::<CM_RESOURCE_LIST>();
        
        let list = std::ptr::addr_of!((*resource_list).list).cast::<CM_FULL_RESOURCE_DESCRIPTOR>();
        for list_index in 0..(*resource_list).count {
            let list_entry = list.add(list_index as usize);
            let partial_list = std::ptr::addr_of!((*list_entry).partial_resource_list.partial_descriptors).cast::<CM_PARTIAL_RESOURCE_DESCRIPTOR>();
            for partial_index in 0..(*list_entry).partial_resource_list.count {
                let partial_entry = partial_list.add(partial_index as usize);
                result_list.push(PhysicalRegion {
                    beginning: (*partial_entry).data.start,
                    size: match (*partial_entry).type_ {
                        CM_RESOURCE_TYPE::CmResourceTypeMemory => (*partial_entry).data.size,
                        CM_RESOURCE_TYPE::CmResourceTypeMemoryLarge => {
                            if ((*partial_entry).flags & CM_RESOURCE_MEMORY_LARGE_40) != 0 {
                                (*partial_entry).data.size << 8
                            } else if ((*partial_entry).flags & CM_RESOURCE_MEMORY_LARGE_48) != 0 {
                                (*partial_entry).data.size << 16
                            } else if ((*partial_entry).flags & CM_RESOURCE_MEMORY_LARGE_64) != 0 {
                                (*partial_entry).data.size << 32
                            } else {
                                0
                            }
                        }
                        _ => 0
                    },
                    flags: (*partial_entry).flags,
                    readwrite: ((*partial_entry).flags & 0xFF) == CM_RESOURCE_MEMORY_READ_WRITE,
                    readonly: ((*partial_entry).flags & CM_RESOURCE_MEMORY_READ_ONLY) != 0,
                    writeonly: ((*partial_entry).flags & CM_RESOURCE_MEMORY_WRITE_ONLY) != 0,
                    prefetchable: ((*partial_entry).flags & CM_RESOURCE_MEMORY_PREFETCHABLE) != 0,
                    cacheable: ((*partial_entry).flags & CM_RESOURCE_MEMORY_CACHEABLE) != 0,
                    write_combined: ((*partial_entry).flags & CM_RESOURCE_MEMORY_COMBINEDWRITE) != 0
                });
            }
        }

        Ok(result_list)
    }
}
