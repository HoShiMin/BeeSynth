use std::ops::{Deref, DerefMut};

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{CONTEXT, SetThreadContext};
use windows::Win32::System::Threading::{SuspendThread, ResumeThread};



#[derive(Default, Clone, Copy)]
pub(crate) struct ThreadState {
    rip: u64,
    rsp: u64
}

impl From<CONTEXT> for ThreadState {
    fn from(context: CONTEXT) -> Self {
        Self {
            rip: context.Rip,
            rsp: context.Rsp
        }
    }
}

#[allow(clippy::similar_names)]
impl ThreadState {
    pub fn rip(&self) -> u64 {
        self.rip
    }

    pub fn rsp(&self) -> u64 {
        self.rsp
    }
}



#[repr(C, align(16))]
#[derive(Default, Clone, Copy)]
pub(crate) struct AlignedContext(CONTEXT);

impl Deref for AlignedContext {
    type Target = CONTEXT;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}



pub(crate) struct Suspender {
    thread_handle: HANDLE
}

impl Suspender {
    pub fn new(thread_handle: HANDLE) -> Self {
        unsafe { SuspendThread(thread_handle) };
        Self { thread_handle }
    }
}

impl Drop for Suspender {
    fn drop(&mut self) {
        unsafe { ResumeThread(self.thread_handle) };
    }
}



pub(crate) struct ContextKeeper {
    thread_handle: HANDLE,
    context: AlignedContext
}

impl ContextKeeper {
    pub fn new(thread_handle: HANDLE, context: &AlignedContext) -> Self {
        Self { thread_handle, context: *context }
    }
}

impl Deref for ContextKeeper {
    type Target = CONTEXT;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl Drop for ContextKeeper {
    fn drop(&mut self) {
        unsafe { SetThreadContext(self.thread_handle, &*self.context) };
    }
}
