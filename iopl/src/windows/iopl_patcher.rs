use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{
    OpenThread,
    THREAD_SUSPEND_RESUME,
    THREAD_GET_CONTEXT,
    THREAD_SET_CONTEXT,
    GetCurrentThreadId, INFINITE
};
use winapi::auto::HandleWrapper;
use winapi::sync;

use crate::level::Level;

use super::error::Error as IoplError;
use super::ktrap_frame::TrapFrame;
use super::phys_mapper::{Mapper, Mapping};
use super::{types, consts, phys_ranges, patcher_impl};


pub struct Patcher<'a, PhysMapper>
    where PhysMapper: Mapper<'a>
{
    mapper: &'a PhysMapper
}

impl<'a, PhysMapper: Mapper<'a>> Patcher<'a, PhysMapper> {
    pub fn new(mapper: &'a PhysMapper) -> Self {
        Self { mapper }
    }

    /// # Errors
    /// 
    /// Returns `IoplError` in case of patch failure.
    pub fn patch(&self, level: Level) -> Result<(), IoplError> {
        let thread_handle = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT,
                false,
                GetCurrentThreadId()
            )
        }.map_err(IoplError::ThreadOpening)?.wrap_handle();
    
        let mut status = Err(IoplError::EflagsNotFound);
    
        let thread_is_ready_event = sync::Event::new_manual(false);
        let scan_has_finished_event = sync::Event::new_manual(false);

        std::thread::scope(|scope| {
            scope.spawn(|| {
                let thread_ready_status = thread_is_ready_event.wait();
                if !thread_ready_status.is_signaled() {
                    status = Err(IoplError::WaitFailure(thread_ready_status));
                    return;
                }
    
                status = self.patch_worker(thread_handle.get(), level);
                scan_has_finished_event.set_event();
            });
    
            let _wait_status = sync::Event::signal_and_wait(&thread_is_ready_event, &scan_has_finished_event, INFINITE);
        });
    
        status
    }
    


    fn patch_worker(&self, thread_handle: HANDLE, level: Level) -> Result<(), IoplError> {
        let physical_regions = {
            let mut regions = phys_ranges::get_physical_memory_ranges().map_err(IoplError::EnumPhysRegions)?;
            if regions.is_empty() {
                return Err(IoplError::NoPhysicalRegions);
            }
    
            // Sort in descending order:
            regions.sort_by(|a, b| b.size.cmp(&a.size));
            
            regions
        };
    
        types::Suspender::new(thread_handle);
    
        let context_keeper = unsafe { patcher_impl::prepare_context(thread_handle) }?;
        let thread_state: types::ThreadState = (*context_keeper).into();

        let mut is_found = false;
        for physical_region in physical_regions {
            if physical_region.size < std::mem::size_of::<TrapFrame>() as u64 {
                continue;
            }
    
            if !physical_region.readwrite {
                continue;
            }
    
            let Some(mapped) = self.mapper.map(physical_region.beginning, physical_region.size) else {
                continue;
            };
    
            unsafe {
                #[allow(clippy::cast_possible_truncation)]
                let found = patcher_impl::find_eflags(patcher_impl::convert_slice_to_u32(mapped.mapping()), &thread_state);
                if !found.is_null() {
                    drop(context_keeper); // Restore the original context

                    std::ptr::write_volatile(found, std::ptr::read_volatile(found) | ((level as u32) << consts::EFLAGS_IOPL_OFFSET));
                    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    
                    is_found = true;
                    mapped.unmap();
                    break;
                }
            }
    
            mapped.unmap();
        }

        if is_found {
            Ok(())
        } else {
            Err(IoplError::EflagsNotFound)
        }
    }
    
}
