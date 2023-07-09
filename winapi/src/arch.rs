#[cfg(target_arch = "x86")]
use windows::Win32::{
    System::Threading::{
        IsWow64Process,
        GetCurrentProcess
    },
    Foundation::BOOL
};



#[allow(non_camel_case_types)]
pub enum Arch {
    Unknown,
    i386,
    x64
}

#[must_use]
pub fn os() -> Arch {
    #[cfg(target_arch = "x86_64")]
    {
        Arch::x64
    }

    #[cfg(target_arch = "x86")]
    {
        let mut is_wow64 = BOOL::default();
        if unsafe { IsWow64Process(GetCurrentProcess(), std::ptr::addr_of_mut!(is_wow64)) }.as_bool() {
            if is_wow64.as_bool() {
                Arch::x64
            } else {
                Arch::i386
            }
        } else {
            Arch::Unknown
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        Arch::Unknown
    }
}

#[must_use]
pub const fn current() -> Arch {
    #[cfg(target_arch = "x86_64")]
    {
        Arch::x64
    }
    
    #[cfg(target_arch = "x86")]
    {
        Arch::i386
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        Arch::Unknown
    }
}