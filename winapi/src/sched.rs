use windows::Win32::System::{
    Threading,
    SystemInformation::{
        GetSystemInfo,
        SYSTEM_INFO
    }
};

#[derive(Clone, Copy)]
pub enum Priority {
    Idle,
    Lowest,
    Lower,
    Normal,
    Higher,
    Highest,
    Realtime
}

impl Priority {
    fn get_process_priority_class(self) -> Threading::PROCESS_CREATION_FLAGS {
        match self {
            Priority::Idle => Threading::IDLE_PRIORITY_CLASS,
            Priority::Lowest | Priority::Lower => Threading::BELOW_NORMAL_PRIORITY_CLASS,
            Priority::Normal => Threading::NORMAL_PRIORITY_CLASS,
            Priority::Higher => Threading::ABOVE_NORMAL_PRIORITY_CLASS,
            Priority::Highest => Threading::HIGH_PRIORITY_CLASS,
            Priority::Realtime => Threading::REALTIME_PRIORITY_CLASS
        }
    }

    fn get_thread_priority(self) -> Threading::THREAD_PRIORITY {
        match self {
            Priority::Idle => Threading::THREAD_PRIORITY_IDLE,
            Priority::Lowest => Threading::THREAD_PRIORITY_LOWEST,
            Priority::Lower => Threading::THREAD_PRIORITY_BELOW_NORMAL,
            Priority::Normal => Threading::THREAD_PRIORITY_NORMAL,
            Priority::Higher => Threading::THREAD_PRIORITY_ABOVE_NORMAL,
            Priority::Highest => Threading::THREAD_PRIORITY_HIGHEST,
            Priority::Realtime => Threading::THREAD_PRIORITY_TIME_CRITICAL
        }
    }
}

#[allow(clippy::must_use_candidate)]
pub fn set_process_priority(priority: Priority) -> bool {
    unsafe {
        Threading::SetPriorityClass(Threading::GetCurrentProcess(), priority.get_process_priority_class()).as_bool()
    }
}

#[allow(clippy::must_use_candidate)]
pub fn set_thread_priority(priority: Priority) -> bool {
    unsafe {
        Threading::SetThreadPriority(Threading::GetCurrentThread(), priority.get_thread_priority()).as_bool()
    }
}

#[derive(Clone, Copy)]
pub enum Affinity {
    All,
    Exact(usize),
    Mask(usize)
}

#[allow(clippy::must_use_candidate)]
pub fn set_affinity(affinity: Affinity) -> Affinity {
    let mask: usize = match affinity {
        Affinity::All => 0,
        Affinity::Exact(cpu_number) => 1 << cpu_number,
        Affinity::Mask(cpu_mask) => cpu_mask
    };

    unsafe { Affinity::Mask(Threading::SetThreadAffinityMask(Threading::GetCurrentThread(), mask)) }
}

#[must_use]
pub fn get_cpu_count() -> usize {
    unsafe {
        let mut system_info = SYSTEM_INFO::default();
        GetSystemInfo(&mut system_info);
        system_info.dwNumberOfProcessors as usize
    }
}